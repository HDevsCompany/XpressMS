// rutas_gestion.rs (acceso/personal): las "puertas" con las que el Administrador administra al personal.
// Solo reciben JSON, comprueban el rol, llaman a gestion.rs y devuelven JSON. No contienen SQL ni reglas.
// TODAS exigen sesión y rol de Administrador: el backend decide, no el navegador.
//
//   GET    /panel/api/personal                              lista del personal
//   POST   /panel/api/personal                              da de alta   -> { usuario, clave_temporal }
//   PUT    /panel/api/personal/{codigo}                     edita datos y roles
//   POST   /panel/api/personal/{codigo}/restablecer-clave   clave nueva  -> { usuario, clave_temporal }
//   DELETE /panel/api/personal/{codigo}                     da de baja
// {codigo} es el código del usuario (el que trae cada fila de la lista).

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use sqlx::PgPool;

use super::gestion::{self, Gestion};
use super::modelo::{EntradaAlta, EntradaEdicion, UsuarioActual};
use super::repositorio_gestion;
use super::roles::ROL_ADMIN;
use super::sesion::{exigir_rol, PersonalAutenticado};

pub fn rutas() -> Router<PgPool> {
    Router::new()
        .route("/panel/api/personal", get(listar).post(crear))
        .route("/panel/api/personal/{codigo}", axum::routing::put(actualizar).delete(eliminar))
        .route("/panel/api/personal/{codigo}/restablecer-clave", post(restablecer_clave))
}

// Texto de error que el navegador muestra tal cual.
#[derive(Serialize)]
struct Mensaje {
    mensaje: String,
}

fn con_mensaje(codigo: StatusCode, texto: String) -> Response {
    (codigo, Json(Mensaje { mensaje: texto })).into_response()
}

fn error_interno(error: sqlx::Error) -> Response {
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

// Los casos de error son los mismos para todas las rutas.
fn error_de<T>(gestion: Gestion<T>) -> Response {
    match gestion {
        Gestion::Invalido(mensaje) => con_mensaje(StatusCode::BAD_REQUEST, mensaje),
        Gestion::Repetido(mensaje) => con_mensaje(StatusCode::CONFLICT, mensaje),
        _ => con_mensaje(
            StatusCode::NOT_FOUND,
            "Esa persona ya no existe. Recarga la página.".to_string(),
        ),
    }
}

// Respuesta de las operaciones que no devuelven datos (editar, eliminar): 204 si salió bien.
fn responder_vacio(resultado: Result<Gestion<()>, sqlx::Error>) -> Response {
    match resultado {
        Err(error) => error_interno(error),
        Ok(Gestion::Hecho(())) => StatusCode::NO_CONTENT.into_response(),
        Ok(otro) => error_de(otro),
    }
}

// Respuesta de las operaciones que entregan una clave temporal (alta, restablecer).
// "no-store" evita que el navegador guarde en caché una respuesta con una clave.
fn responder_clave(
    resultado: Result<Gestion<super::modelo::ClaveTemporal>, sqlx::Error>,
    exito: StatusCode,
) -> Response {
    match resultado {
        Err(error) => error_interno(error),
        Ok(Gestion::Hecho(clave)) => (exito, [(header::CACHE_CONTROL, "no-store")], Json(clave)).into_response(),
        Ok(otro) => error_de(otro),
    }
}

async fn listar(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    match repositorio_gestion::listar(&pool).await {
        Ok(lista) => ([(header::CACHE_CONTROL, "no-store")], Json(lista)).into_response(),
        Err(error) => error_interno(error),
    }
}

async fn crear(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Json(entrada): Json<EntradaAlta>,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    responder_clave(gestion::crear(&pool, entrada).await, StatusCode::CREATED)
}

async fn actualizar(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
    Json(entrada): Json<EntradaEdicion>,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    responder_vacio(gestion::actualizar(&pool, &usuario, &codigo, entrada).await)
}

async fn restablecer_clave(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    responder_clave(
        gestion::restablecer_clave(&pool, &usuario, &codigo).await,
        StatusCode::OK,
    )
}

async fn eliminar(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    responder_vacio(gestion::eliminar(&pool, &usuario, &codigo).await)
}
