// rutas.rs (publico): los datos de la página principal (/), que puede ver cualquier visitante sin iniciar sesión.
//
//   GET /publico/inicio   -> { categorias, destacados, servicios }
//   GET /publico/catalogo -> la vitrina completa de /catalogo: { tipo: "catalogo", filtros, total, productos }
//   GET /publico/servicios -> lo que muestra /servicios: { servicios, diagnostico }
//
// Solo se muestra información que ya es pública: nombre, precio y disponibilidad de los productos activos, y los
// servicios activos. No hay datos de clientes. La página (templates/publico/inicio.html) arma las tarjetas con esto.
// Esta ruta no escribe SQL: reutiliza las consultas de lectura de los módulos catalogo y servicio_tecnico.

use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use sqlx::PgPool;

use crate::modulos::agente::{flujo_catalogo, flujo_solicitud::CODIGO_DIAGNOSTICO, modelo::Componente};
use crate::modulos::catalogo::modelo::{Categoria, Producto};
use crate::modulos::catalogo::repositorio as catalogo_repo;
use crate::modulos::servicio_tecnico::modelo::{Servicio, ServicioPanel};
use crate::modulos::servicio_tecnico::repositorio as servicios_repo;
use crate::modulos::servicio_tecnico::repositorio_gestion as servicios_gestion;

// Cuántos productos y servicios se muestran en la página principal.
const MAX_DESTACADOS: usize = 4;
const MAX_SERVICIOS: usize = 3;

// Lo que recibe el navegador.
#[derive(Serialize)]
struct Inicio {
    categorias: Vec<Categoria>,
    destacados: Vec<Producto>,
    servicios: Vec<Servicio>,
}

// Un servicio tal como lo dibuja la página /servicios (sin códigos internos: Juno entiende el nombre).
#[derive(Serialize)]
struct ServicioPublico {
    nombre: String,
    descripcion: String,
    precio: Option<String>, // "150.00"; None = «A cotizar»
    tiempo: Option<String>, // "1 a 2 días"
    equipos: Vec<String>,   // «Laptop», «PC»
    imagen: Option<String>, // la dirección de su imagen; None = se dibuja la ilustración de siempre
    aviso_precio: bool,     // true = la ficha avisa que el precio puede cambiar
}

// Lo que recibe /servicios. «diagnostico» es el servicio «Diagnóstico de equipo»: no sale en la lista de servicios (tiene su propia
// tarjeta), y si el personal lo eliminó la tarjeta se muestra como no disponible.
#[derive(Serialize)]
struct PaginaServicios {
    servicios: Vec<ServicioPublico>,
    diagnostico: Option<ServicioPublico>,
}

pub fn rutas() -> Router<PgPool> {
    Router::new()
        .route("/publico/inicio", get(inicio))
        .route("/publico/catalogo", get(catalogo))
        .route("/publico/servicios", get(servicios))
        .route("/publico/servicios/{codigo}/imagen", get(imagen_de_servicio))
}

// GET /publico/servicios/{codigo}/imagen: la imagen que el Administrador le puso a un servicio (la dirección lleva "?v=" con un
// número que cambia cuando se cambia la imagen, por eso el navegador puede guardarla un día sin quedarse con una vieja).
async fn imagen_de_servicio(State(pool): State<PgPool>, Path(codigo): Path<String>) -> Response {
    match servicios_gestion::leer_imagen(&pool, &codigo).await {
        Ok(Some((tipo, bytes))) => {
            let mut respuesta = bytes.into_response();
            let cabeceras = respuesta.headers_mut();
            if let Ok(valor) = HeaderValue::from_str(&tipo) {
                cabeceras.insert(header::CONTENT_TYPE, valor);
            }
            cabeceras.insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=86400"));
            respuesta
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => error_interno(error).into_response(),
    }
}

// GET /publico/servicios: los servicios activos con sus equipos, y aparte el diagnóstico. Es información pública (la misma que Juno
// cuenta en el chat); no hay datos de clientes.
async fn servicios(State(pool): State<PgPool>) -> Result<Json<PaginaServicios>, StatusCode> {
    let lista = servicios_gestion::listar_servicios(&pool).await.map_err(error_interno)?;
    Ok(Json(separar_diagnostico(lista)))
}

// Reparte los servicios: el diagnóstico (por su código fijo) va aparte; los demás, en el orden en que llegan.
fn separar_diagnostico(lista: Vec<ServicioPanel>) -> PaginaServicios {
    let mut servicios = Vec::new();
    let mut diagnostico = None;
    for item in lista {
        let es_diagnostico = item.servicio.codigo == CODIGO_DIAGNOSTICO;
        let publico = ServicioPublico {
            nombre: item.servicio.nombre,
            descripcion: item.servicio.descripcion,
            precio: item.servicio.precio_base,
            tiempo: item.servicio.tiempo_estimado,
            equipos: item.equipos.into_iter().map(|e| e.nombre).collect(),
            imagen: item.imagen,
            aviso_precio: item.servicio.aviso_precio,
        };
        if es_diagnostico {
            diagnostico = Some(publico);
        } else {
            servicios.push(publico);
        }
    }
    PaginaServicios { servicios, diagnostico }
}

// GET /publico/catalogo: la vitrina completa de la página /catalogo, visible para cualquiera. Después, quien la filtra es Juno
// (la ventana de la página le pide el filtro al agente y recibe el mismo formato).
async fn catalogo(State(pool): State<PgPool>) -> Result<Json<Componente>, StatusCode> {
    flujo_catalogo::vitrina_completa(&pool).await.map(Json).map_err(error_interno)
}

// GET /publico/inicio
async fn inicio(State(pool): State<PgPool>) -> Result<Json<Inicio>, StatusCode> {
    let categorias = catalogo_repo::listar_categorias(&pool).await.map_err(error_interno)?;
    let productos = catalogo_repo::listar_productos(&pool).await.map_err(error_interno)?;
    let servicios: Vec<Servicio> = servicios_repo::listar_servicios(&pool)
        .await
        .map_err(error_interno)?
        .into_iter()
        .take(MAX_SERVICIOS)
        .collect();

    Ok(Json(Inicio {
        categorias,
        destacados: elegir_destacados(productos),
        servicios,
    }))
}

// Convierte un error de base de datos en un "500" para el navegador y deja el detalle en la terminal.
fn error_interno(error: sqlx::Error) -> StatusCode {
    eprintln!("Error de base de datos: {}", error);
    StatusCode::INTERNAL_SERVER_ERROR
}

// Elige los productos que se muestran primero: solo los que tienen stock, y de variada categoría.
// Primero toma uno de cada categoría (en el orden en que vienen) y, si todavía faltan, completa con los siguientes.
fn elegir_destacados(productos: Vec<Producto>) -> Vec<Producto> {
    let con_stock: Vec<Producto> = productos.into_iter().filter(|p| p.stock > 0).collect();
    let mut elegidos: Vec<Producto> = Vec::new();

    for producto in &con_stock {
        if elegidos.len() >= MAX_DESTACADOS {
            break;
        }
        if !elegidos.iter().any(|e| e.categoria_codigo == producto.categoria_codigo) {
            elegidos.push(producto.clone());
        }
    }
    for producto in &con_stock {
        if elegidos.len() >= MAX_DESTACADOS {
            break;
        }
        if !elegidos.iter().any(|e| e.codigo == producto.codigo) {
            elegidos.push(producto.clone());
        }
    }
    elegidos
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::modulos::servicio_tecnico::modelo::TipoEquipo;

    fn servicio_panel(codigo: &str, nombre: &str) -> ServicioPanel {
        ServicioPanel {
            servicio: Servicio {
                codigo: codigo.to_string(),
                nombre: nombre.to_string(),
                descripcion: "Descripción".to_string(),
                precio_base: Some("60.00".to_string()),
                tiempo_estimado: None,
                aviso_precio: true,
            },
            equipos: vec![TipoEquipo { codigo: "E1".to_string(), nombre: "Laptop".to_string() }],
            imagen: None,
        }
    }

    #[test]
    fn el_diagnostico_va_aparte_de_los_servicios() {
        let pagina = separar_diagnostico(vec![
            servicio_panel("AAAAAAAAA", "Formateo"),
            servicio_panel(CODIGO_DIAGNOSTICO, "Diagnóstico de equipo"),
            servicio_panel("BBBBBBBBB", "Cambio de pantalla"),
        ]);
        let nombres: Vec<&str> = pagina.servicios.iter().map(|s| s.nombre.as_str()).collect();
        assert_eq!(nombres, vec!["Formateo", "Cambio de pantalla"]);
        assert_eq!(pagina.diagnostico.map(|d| d.nombre), Some("Diagnóstico de equipo".to_string()));
    }

    #[test]
    fn sin_diagnostico_la_tarjeta_queda_vacia() {
        let pagina = separar_diagnostico(vec![servicio_panel("AAAAAAAAA", "Formateo")]);
        assert!(pagina.diagnostico.is_none());
        assert_eq!(pagina.servicios[0].equipos, vec!["Laptop".to_string()]);
    }

    fn producto(codigo: &str, categoria: &str, stock: i32) -> Producto {
        Producto {
            codigo: codigo.to_string(),
            categoria_codigo: categoria.to_string(),
            categoria_nombre: categoria.to_string(),
            nombre: format!("Producto {}", codigo),
            descripcion: String::new(),
            precio: "10.00".to_string(),
            stock,
        }
    }

    fn codigos(lista: &[Producto]) -> Vec<&str> {
        lista.iter().map(|p| p.codigo.as_str()).collect()
    }

    #[test]
    fn los_agotados_no_se_destacan() {
        let lista = vec![producto("a", "x", 0), producto("b", "x", 3)];
        assert_eq!(codigos(&elegir_destacados(lista)), vec!["b"]);
    }

    #[test]
    fn primero_uno_por_categoria() {
        let lista = vec![
            producto("a1", "A", 5),
            producto("a2", "A", 5),
            producto("b1", "B", 5),
            producto("c1", "C", 5),
        ];
        assert_eq!(codigos(&elegir_destacados(lista)), vec!["a1", "b1", "c1", "a2"]);
    }

    #[test]
    fn nunca_pasa_del_maximo_ni_repite() {
        let lista = (0..10).map(|i| producto(&format!("p{}", i), "A", 1)).collect();
        let elegidos = elegir_destacados(lista);
        assert_eq!(elegidos.len(), MAX_DESTACADOS);
        assert_eq!(codigos(&elegidos), vec!["p0", "p1", "p2", "p3"]);
    }

    #[test]
    fn sin_productos_no_hay_destacados() {
        assert!(elegir_destacados(Vec::new()).is_empty());
    }
}
