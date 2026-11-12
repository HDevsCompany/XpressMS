// rutas.rs (catalogo): las "puertas" con las que el panel administra productos, categorías y apartados.
// Solo reciben JSON, comprueban el rol, llaman a gestion.rs o apartados.rs y devuelven JSON. Sin SQL ni reglas.
// TODAS exigen sesión y rol de Vendedor o Administrador: el backend decide, no el navegador.
//
//   GET    /panel/api/categorias               lista de categorías (con cuántos productos tiene cada una)
//   POST   /panel/api/categorias               crea una          { nombre }
//   PUT    /panel/api/categorias/{codigo}      cambia su nombre  { nombre }
//   DELETE /panel/api/categorias/{codigo}      la elimina (solo si no tiene productos)
//   GET    /panel/api/productos                lista de productos
//   POST   /panel/api/productos                crea uno          { categoria, nombre, descripcion, precio, stock }
//   PUT    /panel/api/productos/{codigo}       lo edita (mismo cuerpo)
//   DELETE /panel/api/productos/{codigo}       lo elimina (solo si no tiene apartados pendientes)
//   GET    /panel/api/apartados                lista de apartados (los vencidos se actualizan al consultar)
//   POST   /panel/api/apartados/{codigo}/entregar   el cliente recogió y pagó
//   POST   /panel/api/apartados/{codigo}/cancelar   se cancela y el stock vuelve

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use sqlx::PgPool;

use super::modelo::{EntradaCategoria, EntradaProducto};
use super::{apartados, gestion, repositorio, repositorio_gestion};
use crate::modulos::acceso::personal::modelo::UsuarioActual;
use crate::modulos::acceso::personal::roles::{ROL_ADMIN, ROL_VENDEDOR};
use crate::modulos::acceso::personal::sesion::{exigir_rol, PersonalAutenticado};
use crate::modulos::servicio_tecnico::rutas::{error_interno, responder};

pub fn rutas() -> Router<PgPool> {
    Router::new()
        .route("/panel/api/categorias", get(listar_categorias).post(crear_categoria))
        .route(
            "/panel/api/categorias/{codigo}",
            put(renombrar_categoria).delete(eliminar_categoria),
        )
        .route("/panel/api/productos", get(listar_productos).post(crear_producto))
        .route(
            "/panel/api/productos/{codigo}",
            put(actualizar_producto).delete(eliminar_producto),
        )
        .route("/panel/api/apartados", get(listar_apartados))
        .route("/panel/api/apartados/{codigo}/entregar", post(entregar_apartado))
        .route("/panel/api/apartados/{codigo}/cancelar", post(cancelar_apartado))
}

// Solo el Vendedor o el Administrador (con su clave ya cambiada) pueden usar estas rutas.
fn solo_ventas(usuario: &UsuarioActual) -> Result<(), Response> {
    exigir_rol(usuario, &[ROL_ADMIN, ROL_VENDEDOR]).map_err(|codigo| codigo.into_response())
}

// ---------- Categorías ----------

async fn listar_categorias(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
) -> Response {
    if let Err(rechazo) = solo_ventas(&usuario) {
        return rechazo;
    }
    match repositorio_gestion::listar_categorias(&pool).await {
        Ok(lista) => ([(header::CACHE_CONTROL, "no-store")], Json(lista)).into_response(),
        Err(error) => error_interno(error),
    }
}

async fn crear_categoria(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Json(entrada): Json<EntradaCategoria>,
) -> Response {
    if let Err(rechazo) = solo_ventas(&usuario) {
        return rechazo;
    }
    responder(gestion::crear_categoria(&pool, entrada).await, StatusCode::CREATED)
}

async fn renombrar_categoria(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
    Json(entrada): Json<EntradaCategoria>,
) -> Response {
    if let Err(rechazo) = solo_ventas(&usuario) {
        return rechazo;
    }
    responder(
        gestion::renombrar_categoria(&pool, &codigo, entrada).await,
        StatusCode::NO_CONTENT,
    )
}

async fn eliminar_categoria(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
) -> Response {
    if let Err(rechazo) = solo_ventas(&usuario) {
        return rechazo;
    }
    responder(gestion::eliminar_categoria(&pool, &codigo).await, StatusCode::NO_CONTENT)
}

// ---------- Productos ----------

async fn listar_productos(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
) -> Response {
    if let Err(rechazo) = solo_ventas(&usuario) {
        return rechazo;
    }
    match repositorio::listar_productos(&pool).await {
        Ok(lista) => ([(header::CACHE_CONTROL, "no-store")], Json(lista)).into_response(),
        Err(error) => error_interno(error),
    }
}

async fn crear_producto(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Json(entrada): Json<EntradaProducto>,
) -> Response {
    if let Err(rechazo) = solo_ventas(&usuario) {
        return rechazo;
    }
    responder(gestion::crear_producto(&pool, entrada).await, StatusCode::CREATED)
}

async fn actualizar_producto(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
    Json(entrada): Json<EntradaProducto>,
) -> Response {
    if let Err(rechazo) = solo_ventas(&usuario) {
        return rechazo;
    }
    responder(
        gestion::actualizar_producto(&pool, &codigo, entrada).await,
        StatusCode::NO_CONTENT,
    )
}

async fn eliminar_producto(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
) -> Response {
    if let Err(rechazo) = solo_ventas(&usuario) {
        return rechazo;
    }
    responder(gestion::eliminar_producto(&pool, &codigo).await, StatusCode::NO_CONTENT)
}

// ---------- Apartados ----------

async fn listar_apartados(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
) -> Response {
    if let Err(rechazo) = solo_ventas(&usuario) {
        return rechazo;
    }
    match apartados::listar_para_panel(&pool).await {
        Ok(lista) => ([(header::CACHE_CONTROL, "no-store")], Json(lista)).into_response(),
        Err(error) => error_interno(error),
    }
}

async fn entregar_apartado(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
) -> Response {
    if let Err(rechazo) = solo_ventas(&usuario) {
        return rechazo;
    }
    responder(
        apartados::entregar(&pool, &usuario.codigo, &codigo).await,
        StatusCode::NO_CONTENT,
    )
}

async fn cancelar_apartado(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
) -> Response {
    if let Err(rechazo) = solo_ventas(&usuario) {
        return rechazo;
    }
    responder(
        apartados::cancelar(&pool, &usuario.codigo, &codigo).await,
        StatusCode::NO_CONTENT,
    )
}
