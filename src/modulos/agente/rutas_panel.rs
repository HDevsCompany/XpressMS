// rutas_panel.rs (agente): la «puerta» de Juno en el panel de servicios. Solo para el Administrador.
//
//   POST /panel/api/juno/servicios   { texto, accion?, estado }  ->  { respuesta, opciones, estado, borrador, hecho, foco }
//
// Aquí se junta todo: se leen los servicios y equipos de la base, panel_servicios.rs (que no toca la base) decide qué responde Juno y,
// solo si la persona ya confirmó, se EJECUTA la acción (crear, guardar cambios o eliminar) con las mismas reglas de gestion.rs que usa
// el resto del panel. La imagen del servicio la sube el navegador después, con PUT /panel/api/servicios/{codigo}/imagen.

use axum::{
    extract::State,
    http::header,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use sqlx::PgPool;

use super::flujo_solicitud::CODIGO_DIAGNOSTICO;
use super::panel_servicios::{self as juno, Accion, Catalogo, Entrada, EquipoCat, Estado, Resultado, ServicioCat};
use crate::modulos::acceso::personal::roles::ROL_ADMIN;
use crate::modulos::acceso::personal::sesion::{exigir_rol, PersonalAutenticado};
use crate::modulos::servicio_tecnico::gestion::{self, Gestion};
use crate::modulos::servicio_tecnico::modelo::EntradaServicio;
use crate::modulos::servicio_tecnico::repositorio_gestion as repositorio;
use crate::modulos::servicio_tecnico::rutas::error_interno;

pub fn rutas() -> Router<PgPool> {
    Router::new().route("/panel/api/juno/servicios", post(conversar))
}

async fn conversar(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Json(entrada): Json<Entrada>,
) -> Response {
    if let Err(codigo) = exigir_rol(&usuario, &[ROL_ADMIN]) {
        return codigo.into_response();
    }
    let catalogo = match cargar_catalogo(&pool).await {
        Ok(catalogo) => catalogo,
        Err(error) => return error_interno(error),
    };

    let turno = juno::responder(&catalogo, &usuario.nombre, &entrada);
    let salida = match turno.accion {
        None => turno.salida,
        Some(accion) => {
            let estado = turno.salida.estado.clone();
            match ejecutar(&pool, &estado, &accion).await {
                Ok(resultado) => juno::tras_ejecutar(&catalogo, &estado, &accion, resultado),
                Err(error) => return error_interno(error),
            }
        }
    };
    ([(header::CACHE_CONTROL, "no-store")], Json(salida)).into_response()
}

// Los servicios y tipos de equipo tal como los ve Juno.
async fn cargar_catalogo(pool: &PgPool) -> Result<Catalogo, sqlx::Error> {
    let equipos = repositorio::listar_equipos(pool).await?;
    let servicios = repositorio::listar_servicios(pool).await?;

    let equipos: Vec<EquipoCat> = equipos.into_iter().map(|q| EquipoCat { codigo: q.codigo, nombre: q.nombre }).collect();
    let servicios: Vec<ServicioCat> = servicios
        .into_iter()
        .map(|s| {
            // Los equipos de cada servicio en el mismo orden que el catálogo (así se comparan bien al editar).
            let codigos: Vec<String> = equipos
                .iter()
                .filter(|q| s.equipos.iter().any(|t| t.codigo == q.codigo))
                .map(|q| q.codigo.clone())
                .collect();
            ServicioCat {
                protegido: s.servicio.codigo == CODIGO_DIAGNOSTICO,
                codigo: s.servicio.codigo,
                nombre: s.servicio.nombre,
                descripcion: s.servicio.descripcion,
                precio: s.servicio.precio_base,
                tiempo: s.servicio.tiempo_estimado,
                equipos: codigos,
                aviso_precio: s.servicio.aviso_precio,
                imagen: s.imagen,
            }
        })
        .collect();
    Ok(Catalogo { servicios, equipos })
}

// Convierte lo que decidió gestion.rs en lo que Juno entiende.
fn traducir(gestion: Gestion, codigo: String) -> Resultado {
    match gestion {
        Gestion::Hecho => Resultado::Hecho(codigo),
        Gestion::Invalido(mensaje) | Gestion::Repetido(mensaje) => Resultado::Rechazo(mensaje),
        Gestion::NoEncontrado => Resultado::Rechazo("Ese servicio ya no existe. Recarga la página.".to_string()),
    }
}

// Hace de verdad lo que la persona confirmó.
async fn ejecutar(pool: &PgPool, estado: &Estado, accion: &Accion) -> Result<Resultado, sqlx::Error> {
    let ficha = &estado.ficha;
    let entrada = || EntradaServicio {
        nombre: ficha.nombre.clone(),
        descripcion: ficha.descripcion.clone(),
        precio_base: ficha.precio.valor.clone(),
        tiempo_estimado: ficha.tiempo.valor.clone(),
        equipos: ficha.equipos.clone().unwrap_or_default(),
        aviso_precio: Some(ficha.aviso_precio),
    };

    match accion {
        Accion::Crear => {
            let (resultado, codigo) = gestion::crear_servicio_con_codigo(pool, entrada()).await?;
            Ok(traducir(resultado, codigo.unwrap_or_default()))
        }
        Accion::Editar => {
            let resultado = gestion::actualizar_servicio(pool, &estado.codigo, entrada()).await?;
            if matches!(resultado, Gestion::Hecho) && ficha.imagen == "quitar" {
                let quitada = gestion::quitar_imagen_servicio(pool, &estado.codigo).await?;
                if !matches!(quitada, Gestion::Hecho) {
                    return Ok(traducir(quitada, estado.codigo.clone()));
                }
            }
            Ok(traducir(resultado, estado.codigo.clone()))
        }
        Accion::Eliminar => {
            let resultado = gestion::eliminar_servicio(pool, &estado.codigo).await?;
            Ok(traducir(resultado, estado.codigo.clone()))
        }
    }
}
