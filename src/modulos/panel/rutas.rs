// rutas.rs (panel): las páginas del panel interno. Todas exigen sesión: sin ella, redirigen a /ingresar.
//
//   GET /panel            -> página de inicio del panel (templates/panel/inicio.html)
//   GET /panel/servicios  -> servicios y tipos de equipo (solo Administrador)
//   GET /panel/personal   -> empleados y usuarios (solo Administrador)
//   GET /panel/ordenes    -> órdenes de servicio (Administrador y Técnico)
//   GET /panel/productos  -> productos y categorías (Administrador y Vendedor)
//   GET /panel/apartados  -> productos apartados por clientes (Administrador y Vendedor)
//
// La página es la misma para todos; sus datos (nombre, roles) los pide el navegador a /acceso/personal/yo,
// y el backend sigue siendo quien decide qué puede hacer cada rol en cada ruta de datos (/panel/api/...).

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use sqlx::PgPool;

use crate::modulos::acceso::personal::roles::{ROL_ADMIN, ROL_TECNICO, ROL_VENDEDOR};
use crate::modulos::acceso::personal::sesion::{self, exigir_rol};

pub fn rutas() -> Router<PgPool> {
    Router::new()
        .route("/panel", get(pagina_inicio))
        .route("/panel/servicios", get(pagina_servicios))
        .route("/panel/personal", get(pagina_personal))
        .route("/panel/ordenes", get(pagina_ordenes))
        .route("/panel/productos", get(pagina_productos))
        .route("/panel/apartados", get(pagina_apartados))
}

fn error_interno(error: sqlx::Error) -> Response {
    eprintln!("Error de base de datos: {}", error);
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

// Entrega una página HTML (sin guardarla en caché del navegador).
fn pagina(html: &'static str) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], Html(html)).into_response()
}

// GET /panel
async fn pagina_inicio(State(pool): State<PgPool>, cabeceras: HeaderMap) -> Response {
    match sesion::usuario_actual(&pool, &cabeceras).await {
        Ok(Some(_)) => pagina(include_str!("../../../templates/panel/inicio.html")),
        Ok(None) => Redirect::to("/ingresar").into_response(), // sin sesión: a ingresar
        Err(error) => error_interno(error),
    }
}

// Páginas restringidas a ciertos roles. Sin sesión va a /ingresar. Con sesión pero sin permiso (otro rol,
// o clave temporal sin cambiar) vuelve al inicio del panel. El permiso real lo vuelve a comprobar cada
// ruta de datos.
async fn pagina_de_roles(
    pool: &PgPool,
    cabeceras: &HeaderMap,
    roles: &[&str],
    html: &'static str,
) -> Response {
    match sesion::usuario_actual(pool, cabeceras).await {
        Ok(Some(usuario)) if exigir_rol(&usuario, roles).is_ok() => pagina(html),
        Ok(Some(_)) => Redirect::to("/panel").into_response(),
        Ok(None) => Redirect::to("/ingresar").into_response(),
        Err(error) => error_interno(error),
    }
}

// GET /panel/servicios
async fn pagina_servicios(State(pool): State<PgPool>, cabeceras: HeaderMap) -> Response {
    pagina_de_roles(&pool, &cabeceras, &[ROL_ADMIN], include_str!("../../../templates/panel/servicios.html")).await
}

// GET /panel/personal
async fn pagina_personal(State(pool): State<PgPool>, cabeceras: HeaderMap) -> Response {
    pagina_de_roles(&pool, &cabeceras, &[ROL_ADMIN], include_str!("../../../templates/panel/personal.html")).await
}

// GET /panel/ordenes (el Administrador atiende todas; el Técnico, las suyas)
async fn pagina_ordenes(State(pool): State<PgPool>, cabeceras: HeaderMap) -> Response {
    pagina_de_roles(
        &pool,
        &cabeceras,
        &[ROL_ADMIN, ROL_TECNICO],
        include_str!("../../../templates/panel/ordenes.html"),
    )
    .await
}

// GET /panel/productos (el Administrador y el Vendedor administran el catálogo)
async fn pagina_productos(State(pool): State<PgPool>, cabeceras: HeaderMap) -> Response {
    pagina_de_roles(
        &pool,
        &cabeceras,
        &[ROL_ADMIN, ROL_VENDEDOR],
        include_str!("../../../templates/panel/productos.html"),
    )
    .await
}

// GET /panel/apartados (el Administrador y el Vendedor entregan o cancelan lo apartado)
async fn pagina_apartados(State(pool): State<PgPool>, cabeceras: HeaderMap) -> Response {
    pagina_de_roles(
        &pool,
        &cabeceras,
        &[ROL_ADMIN, ROL_VENDEDOR],
        include_str!("../../../templates/panel/apartados.html"),
    )
    .await
}
