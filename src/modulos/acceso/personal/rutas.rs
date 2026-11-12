// rutas.rs (acceso/personal): las "puertas" del ingreso del personal. Solo reciben JSON, llaman a la
// lógica y devuelven JSON. No contienen SQL ni reglas.
//
//   GET  /ingresar                       -> la página de ingreso (si ya hay sesión, va al panel)
//   POST /acceso/personal/entrar         -> usuario y clave; si son correctos, entrega la cookie de sesión
//   POST /acceso/personal/salir          -> cierra la sesión y borra la cookie
//   GET  /acceso/personal/yo             -> quién está en sesión (nombre, roles)
//   POST /acceso/personal/cambiar-clave  -> clave actual + clave nueva

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::autenticacion::{self, ResultadoCambio, ResultadoEntrada};
use super::modelo::UsuarioActual;
use super::sesion::{self, PersonalAutenticado};
use super::{claves, cookies, repositorio};

pub fn rutas() -> Router<PgPool> {
    Router::new()
        .route("/ingresar", get(pagina_ingresar))
        .route("/acceso/personal/entrar", post(entrar))
        .route("/acceso/personal/salir", post(salir))
        .route("/acceso/personal/yo", get(yo))
        .route("/acceso/personal/cambiar-clave", post(cambiar_clave))
}

#[derive(Deserialize)]
struct EntrarEntrada {
    usuario: String,
    clave: String,
}

#[derive(Serialize)]
struct EntrarSalida {
    debe_cambiar_clave: bool,
}

#[derive(Deserialize)]
struct CambiarClaveEntrada {
    actual: String,
    nueva: String,
}

// Respuesta de error con un texto que el navegador puede mostrar tal cual.
#[derive(Serialize)]
struct Mensaje {
    mensaje: String,
}

fn con_mensaje(codigo: StatusCode, texto: &str) -> Response {
    (codigo, Json(Mensaje { mensaje: texto.to_string() })).into_response()
}

fn error_interno(error: sqlx::Error) -> Response {
    eprintln!("Error de base de datos: {}", error);
    con_mensaje(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Ocurrió un problema. Inténtalo de nuevo en un momento.",
    )
}

// En Render la conexión llega por HTTPS y el proxy lo avisa con este encabezado; en localhost no.
// Solo con HTTPS se marca la cookie como "Secure".
fn es_https(cabeceras: &HeaderMap) -> bool {
    cabeceras
        .get("x-forwarded-proto")
        .and_then(|valor| valor.to_str().ok())
        .is_some_and(|valor| valor == "https")
}

// GET /ingresar
async fn pagina_ingresar(State(pool): State<PgPool>, cabeceras: HeaderMap) -> Response {
    match sesion::usuario_actual(&pool, &cabeceras).await {
        Ok(Some(_)) => Redirect::to("/panel").into_response(), // ya ingresó: directo al panel
        Ok(None) => (
            [(header::CACHE_CONTROL, "no-store")],
            Html(include_str!("../../../../templates/acceso/ingresar.html")),
        )
            .into_response(),
        Err(error) => error_interno(error),
    }
}

// POST /acceso/personal/entrar
async fn entrar(
    State(pool): State<PgPool>,
    cabeceras: HeaderMap,
    Json(entrada): Json<EntrarEntrada>,
) -> Response {
    match autenticacion::entrar(&pool, &entrada.usuario, &entrada.clave).await {
        Err(error) => error_interno(error),
        // 401 = "no identificado"
        Ok(ResultadoEntrada::Incorrecto) => {
            con_mensaje(StatusCode::UNAUTHORIZED, "Usuario o contraseña incorrectos.")
        }
        // 429 = "demasiados intentos"
        Ok(ResultadoEntrada::Bloqueado) => con_mensaje(
            StatusCode::TOO_MANY_REQUESTS,
            &format!(
                "Demasiados intentos. Espera {} minutos e inténtalo de nuevo.",
                autenticacion::MINUTOS_BLOQUEO
            ),
        ),
        Ok(ResultadoEntrada::Correcto { token, debe_cambiar_clave }) => (
            [(header::SET_COOKIE, cookies::de_sesion(&token, es_https(&cabeceras)))],
            Json(EntrarSalida { debe_cambiar_clave }),
        )
            .into_response(),
    }
}

// POST /acceso/personal/salir
async fn salir(State(pool): State<PgPool>, cabeceras: HeaderMap) -> Response {
    if let Some(token) = sesion::token_de(&cabeceras) {
        if let Err(error) = repositorio::cerrar_sesion(&pool, &claves::huella_token(&token)).await {
            return error_interno(error);
        }
    }
    (
        [(header::SET_COOKIE, cookies::de_cierre(es_https(&cabeceras)))],
        StatusCode::NO_CONTENT,
    )
        .into_response()
}

// GET /acceso/personal/yo
async fn yo(PersonalAutenticado(usuario): PersonalAutenticado) -> impl IntoResponse {
    ([(header::CACHE_CONTROL, "no-store")], Json::<UsuarioActual>(usuario))
}

// POST /acceso/personal/cambiar-clave
async fn cambiar_clave(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    cabeceras: HeaderMap,
    Json(entrada): Json<CambiarClaveEntrada>,
) -> Response {
    // La huella de la sesión actual: esa sesión NO se cierra al cambiar la clave.
    let Some(token) = sesion::token_de(&cabeceras) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let huella = claves::huella_token(&token);

    match autenticacion::cambiar_clave(&pool, &usuario, &huella, &entrada.actual, &entrada.nueva).await {
        Err(error) => error_interno(error),
        Ok(ResultadoCambio::Hecho) => StatusCode::NO_CONTENT.into_response(),
        Ok(ResultadoCambio::ClaveActualIncorrecta) => {
            con_mensaje(StatusCode::BAD_REQUEST, "La clave actual no es correcta.")
        }
        Ok(ResultadoCambio::ClaveNuevaInvalida(mensaje)) => {
            con_mensaje(StatusCode::BAD_REQUEST, &mensaje)
        }
    }
}
