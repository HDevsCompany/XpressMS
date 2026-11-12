// gestion.rs (servicio_tecnico): las reglas de crear, editar y eliminar servicios y tipos de equipo.
// Aquí se decide: se revisa lo que llegó, se genera el código y se llama al repositorio. Sin SQL.
// Quién puede hacerlo (solo el Administrador) lo decide rutas.rs antes de llegar aquí.

use sqlx::PgPool;

use super::modelo::{EntradaEquipo, EntradaServicio, Resultado};
use super::repositorio_gestion as repositorio;
use super::validaciones::{validar_imagen, validar_nombre_equipo, validar_servicio};
use crate::modulos::agente::flujo_solicitud::CODIGO_DIAGNOSTICO;
use crate::modulos::comun::codigos::generar_codigo;

// El resultado que entiende rutas.rs: cada caso tiene su código HTTP y su mensaje.
pub enum Gestion {
    Hecho,
    Invalido(String),  // lo enviado no cumple las reglas (400)
    Repetido(String),  // ya existe uno con ese nombre (409)
    NoEncontrado,      // no existe o ya fue eliminado (404)
}

// Convierte el resultado del repositorio en el de la gestión, con el mensaje para la persona.
fn traducir(resultado: Resultado, que: &str) -> Gestion {
    match resultado {
        Resultado::Hecho => Gestion::Hecho,
        Resultado::NombreRepetido => Gestion::Repetido(format!("Ya existe {} con ese nombre.", que)),
        Resultado::EquipoInvalido => Gestion::Invalido(
            "Alguno de los tipos de equipo elegidos ya no existe. Recarga la página e inténtalo de nuevo.".to_string(),
        ),
        Resultado::NoEncontrado => Gestion::NoEncontrado,
    }
}

fn desde_bool(existia: bool) -> Gestion {
    if existia {
        Gestion::Hecho
    } else {
        Gestion::NoEncontrado
    }
}

// ---------- Servicios ----------

pub async fn crear_servicio(pool: &PgPool, entrada: EntradaServicio) -> Result<Gestion, sqlx::Error> {
    Ok(crear_servicio_con_codigo(pool, entrada).await?.0)
}

// Igual que crear_servicio, pero además devuelve el código del servicio recién creado (Juno lo necesita para subirle su imagen).
pub async fn crear_servicio_con_codigo(
    pool: &PgPool,
    entrada: EntradaServicio,
) -> Result<(Gestion, Option<String>), sqlx::Error> {
    let datos = match revisar_servicio(&entrada) {
        Ok(datos) => datos,
        Err(mensaje) => return Ok((Gestion::Invalido(mensaje), None)),
    };
    let codigo = generar_codigo();
    let resultado = repositorio::crear_servicio(pool, &codigo, &datos).await?;
    let creado = matches!(resultado, Resultado::Hecho);
    Ok((traducir(resultado, "un servicio"), if creado { Some(codigo) } else { None }))
}

pub async fn actualizar_servicio(
    pool: &PgPool,
    codigo: &str,
    entrada: EntradaServicio,
) -> Result<Gestion, sqlx::Error> {
    let datos = match revisar_servicio(&entrada) {
        Ok(datos) => datos,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };
    let resultado = repositorio::actualizar_servicio(pool, codigo, &datos).await?;
    Ok(traducir(resultado, "un servicio"))
}

pub async fn eliminar_servicio(pool: &PgPool, codigo: &str) -> Result<Gestion, sqlx::Error> {
    // El servicio «Diagnóstico de equipo» (código fijo) lo usa la página /servicios para agendar la cita de diagnóstico: no se elimina.
    if codigo == CODIGO_DIAGNOSTICO {
        return Ok(Gestion::Invalido(
            "El servicio «Diagnóstico de equipo» no se puede eliminar: la página de servicios lo usa para agendar citas de diagnóstico."
                .to_string(),
        ));
    }
    Ok(desde_bool(repositorio::eliminar_servicio(pool, codigo).await?))
}

// Guarda la imagen de un servicio (los bytes ya vienen reducidos por el navegador; aquí se revisa que de verdad sea una imagen).
pub async fn guardar_imagen_servicio(pool: &PgPool, codigo: &str, bytes: &[u8]) -> Result<Gestion, sqlx::Error> {
    let tipo = match validar_imagen(bytes) {
        Ok(tipo) => tipo,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };
    Ok(desde_bool(repositorio::guardar_imagen(pool, codigo, tipo, bytes).await?))
}

pub async fn quitar_imagen_servicio(pool: &PgPool, codigo: &str) -> Result<Gestion, sqlx::Error> {
    Ok(desde_bool(repositorio::quitar_imagen(pool, codigo).await?))
}

fn revisar_servicio(
    entrada: &EntradaServicio,
) -> Result<super::validaciones::DatosServicio, String> {
    let mut datos = validar_servicio(
        &entrada.nombre,
        &entrada.descripcion,
        entrada.precio_base.as_deref(),
        entrada.tiempo_estimado.as_deref(),
        &entrada.equipos,
    )?;
    datos.aviso_precio = entrada.aviso_precio;
    Ok(datos)
}

// ---------- Tipos de equipo ----------

pub async fn crear_tipo_equipo(pool: &PgPool, entrada: EntradaEquipo) -> Result<Gestion, sqlx::Error> {
    let nombre = match validar_nombre_equipo(&entrada.nombre) {
        Ok(nombre) => nombre,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };
    let resultado = repositorio::crear_tipo_equipo(pool, &generar_codigo(), &nombre).await?;
    Ok(traducir(resultado, "un tipo de equipo"))
}

pub async fn renombrar_tipo_equipo(
    pool: &PgPool,
    codigo: &str,
    entrada: EntradaEquipo,
) -> Result<Gestion, sqlx::Error> {
    let nombre = match validar_nombre_equipo(&entrada.nombre) {
        Ok(nombre) => nombre,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };
    let resultado = repositorio::renombrar_tipo_equipo(pool, codigo, &nombre).await?;
    Ok(traducir(resultado, "un tipo de equipo"))
}

pub async fn eliminar_tipo_equipo(pool: &PgPool, codigo: &str) -> Result<Gestion, sqlx::Error> {
    Ok(desde_bool(repositorio::eliminar_tipo_equipo(pool, codigo).await?))
}
