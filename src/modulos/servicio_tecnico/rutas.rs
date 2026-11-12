// rutas.rs (servicio_tecnico): las "puertas" con las que el panel administra servicios y tipos de equipo.
// Solo reciben JSON, comprueban el rol, llaman a gestion.rs y devuelven JSON. No contienen SQL ni reglas.
// TODAS exigen sesión y rol de Administrador: el backend decide, no el navegador.
//
//   GET    /panel/api/equipos            lista de tipos de equipo
//   POST   /panel/api/equipos            crea uno          { nombre }
//   PUT    /panel/api/equipos/{codigo}   cambia su nombre  { nombre }
//   DELETE /panel/api/equipos/{codigo}   lo elimina
//   GET    /panel/api/servicios          lista de servicios con sus equipos
//   POST   /panel/api/servicios          crea uno          { nombre, descripcion, precio_base, tiempo_estimado, equipos }
//   PUT    /panel/api/servicios/{codigo} lo edita (mismo cuerpo)
//   DELETE /panel/api/servicios/{codigo} lo elimina
//   PUT    /panel/api/servicios/{codigo}/imagen   le pone imagen (el cuerpo es la imagen misma: JPG, PNG o WEBP, hasta 1.5 MB)
//   DELETE /panel/api/servicios/{codigo}/imagen   le quita la imagen

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use serde::Serialize;
use sqlx::PgPool;

use super::gestion::{self, Gestion};
use super::modelo::{EntradaEquipo, EntradaServicio};
use super::repositorio_gestion as repositorio;
use crate::modulos::acceso::personal::modelo::UsuarioActual;
use crate::modulos::acceso::personal::roles::ROL_ADMIN;
use crate::modulos::acceso::personal::sesion::{exigir_rol, PersonalAutenticado};

pub fn rutas() -> Router<PgPool> {
    Router::new()
        .route("/panel/api/equipos", get(listar_equipos).post(crear_equipo))
        .route(
            "/panel/api/equipos/{codigo}",
            put(renombrar_equipo).delete(eliminar_equipo),
        )
        .route("/panel/api/servicios", get(listar_servicios).post(crear_servicio))
        .route(
            "/panel/api/servicios/{codigo}",
            put(actualizar_servicio).delete(eliminar_servicio),
        )
        .route(
            "/panel/api/servicios/{codigo}/imagen",
            put(subir_imagen).delete(quitar_imagen),
        )
}

// Texto de error que el navegador muestra tal cual.
#[derive(Serialize)]
struct Mensaje {
    mensaje: String,
}

pub(crate) fn con_mensaje(codigo: StatusCode, texto: String) -> Response {
    (codigo, Json(Mensaje { mensaje: texto })).into_response()
}

pub(crate) fn error_interno(error: sqlx::Error) -> Response {
    eprintln!("Error de base de datos: {}", error);
    con_mensaje(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Ocurrió un problema. Inténtalo de nuevo en un momento.".to_string(),
    )
}

// Solo el Administrador (con su clave ya cambiada) puede usar estas rutas.
fn solo_admin(usuario: &UsuarioActual) -> Result<(), Response> {
    exigir_rol(usuario, &[ROL_ADMIN]).map_err(|codigo| codigo.into_response())
}

// Convierte el resultado de gestion.rs en la respuesta HTTP. "exito" es el código si todo salió bien.
pub(crate) fn responder(resultado: Result<Gestion, sqlx::Error>, exito: StatusCode) -> Response {
    match resultado {
        Err(error) => error_interno(error),
        Ok(Gestion::Hecho) => exito.into_response(),
        Ok(Gestion::Invalido(mensaje)) => con_mensaje(StatusCode::BAD_REQUEST, mensaje),
        Ok(Gestion::Repetido(mensaje)) => con_mensaje(StatusCode::CONFLICT, mensaje),
        Ok(Gestion::NoEncontrado) => con_mensaje(
            StatusCode::NOT_FOUND,
            "Ese registro ya no existe. Recarga la página.".to_string(),
        ),
    }
}

// ---------- Tipos de equipo ----------

async fn listar_equipos(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    match repositorio::listar_equipos(&pool).await {
        Ok(lista) => ([(header::CACHE_CONTROL, "no-store")], Json(lista)).into_response(),
        Err(error) => error_interno(error),
    }
}

async fn crear_equipo(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Json(entrada): Json<EntradaEquipo>,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    responder(gestion::crear_tipo_equipo(&pool, entrada).await, StatusCode::CREATED)
}

async fn renombrar_equipo(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
    Json(entrada): Json<EntradaEquipo>,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    responder(
        gestion::renombrar_tipo_equipo(&pool, &codigo, entrada).await,
        StatusCode::NO_CONTENT,
    )
}

async fn eliminar_equipo(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    responder(gestion::eliminar_tipo_equipo(&pool, &codigo).await, StatusCode::NO_CONTENT)
}

// ---------- Servicios ----------

async fn listar_servicios(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    match repositorio::listar_servicios(&pool).await {
        Ok(lista) => ([(header::CACHE_CONTROL, "no-store")], Json(lista)).into_response(),
        Err(error) => error_interno(error),
    }
}

async fn crear_servicio(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Json(entrada): Json<EntradaServicio>,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    responder(gestion::crear_servicio(&pool, entrada).await, StatusCode::CREATED)
}

async fn actualizar_servicio(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
    Json(entrada): Json<EntradaServicio>,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    responder(
        gestion::actualizar_servicio(&pool, &codigo, entrada).await,
        StatusCode::NO_CONTENT,
    )
}

async fn eliminar_servicio(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    responder(gestion::eliminar_servicio(&pool, &codigo).await, StatusCode::NO_CONTENT)
}

// ---------- Imagen del servicio ----------

// El cuerpo de la petición ES la imagen (sin JSON). gestion.rs revisa que de verdad lo sea.
async fn subir_imagen(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
    cuerpo: Bytes,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    responder(
        gestion::guardar_imagen_servicio(&pool, &codigo, &cuerpo).await,
        StatusCode::NO_CONTENT,
    )
}

async fn quitar_imagen(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    responder(gestion::quitar_imagen_servicio(&pool, &codigo).await, StatusCode::NO_CONTENT)
}
