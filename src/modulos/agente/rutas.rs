// rutas.rs (agente): las "puertas" del chat. Solo reciben JSON, llaman al repositorio y al flujo,
// y devuelven JSON. No contienen SQL ni textos del agente.
//
// Hay dos puertas:
//   POST /agente/iniciar  -> al abrir la página: crea la conversación (o recupera la anterior) y
//                            devuelve el token y los mensajes que ya existían.
//   POST /agente/mensaje  -> cada vez que la persona escribe: guarda su mensaje, responde y guarda la respuesta.

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::modelo::{Componente, MensajeGuardado};
use super::{flujo, repositorio};

// Lo que envía el navegador al abrir la página. "token" es Option porque la primera vez no existe.
#[derive(Deserialize)]
struct IniciarEntrada {
    token: Option<String>,
}

// Lo que respondemos al abrir: el token (el navegador lo guarda), el historial y los botones.
#[derive(Serialize)]
struct IniciarSalida {
    token: String,
    mensajes: Vec<MensajeGuardado>,
    opciones: Vec<String>,
    componente: Option<Componente>, // elemento visual pendiente (por ejemplo la casilla de términos)
}

// Lo que envía el navegador cuando la persona escribe.
#[derive(Deserialize)]
struct MensajeEntrada {
    token: String,
    texto: String,
    // Lo envían los elementos visuales (por ejemplo "aceptar_terminos"). Si la persona solo escribe, no viene.
    accion: Option<String>,
}

// La respuesta del agente a un mensaje.
#[derive(Serialize)]
struct MensajeSalida {
    respuesta: String,
    opciones: Vec<String>,
    componente: Option<Componente>, // casilla de términos, tarjeta de datos...
    espera: Option<String>,         // texto de espera para el próximo mensaje ("Consultando tus datos…")
}

// Agrupa las rutas del agente; main.rs las suma con ".merge(...)".
pub fn rutas() -> Router<PgPool> {
    Router::new()
        .route("/agente/iniciar", post(iniciar))
        .route("/agente/mensaje", post(mensaje))
}

// Convierte un error de base de datos en un "500" para el navegador y deja el detalle en la terminal.
fn error_interno(error: sqlx::Error) -> StatusCode {
    eprintln!("Error de base de datos: {}", error);
    StatusCode::INTERNAL_SERVER_ERROR
}

// POST /agente/iniciar
async fn iniciar(
    State(pool): State<PgPool>,
    Json(entrada): Json<IniciarEntrada>,
) -> Result<Json<IniciarSalida>, StatusCode> {
    // 1. Si el navegador trajo un token, buscamos esa conversación; si no, no hay ninguna.
    let existente = match entrada.token {
        Some(token) => repositorio::buscar_por_token(&pool, &token).await.map_err(error_interno)?,
        None => None,
    };

    // 2. Usamos la conversación existente o creamos una nueva (con el saludo guardado como primer mensaje).
    let conversacion = match existente {
        Some(conversacion) => conversacion,
        None => {
            let nueva = repositorio::crear_conversacion(&pool).await.map_err(error_interno)?;
            let saludo = flujo::saludo_inicial();
            repositorio::guardar_mensaje(&pool, &nueva.codigo, "A", &saludo.texto)
                .await
                .map_err(error_interno)?;
            nueva
        }
    };

    // 3. Traemos el historial para que la persona vea su conversación aunque recargue la página.
    let mensajes = repositorio::listar_mensajes(&pool, &conversacion.codigo)
        .await
        .map_err(error_interno)?;

    // 4. Mostramos los botones del saludo solo si todavía no hubo más conversación.
    let opciones = if mensajes.len() <= 1 {
        flujo::saludo_inicial().opciones
    } else {
        Vec::new()
    };

    // 5. Si la conversación quedó en medio de un paso con elemento visual, lo volvemos a dibujar.
    let componente = flujo::componente_de_reanudacion(&conversacion.paso);

    Ok(Json(IniciarSalida {
        token: conversacion.token,
        mensajes,
        opciones,
        componente,
    }))
}

// POST /agente/mensaje
async fn mensaje(
    State(pool): State<PgPool>,
    Json(entrada): Json<MensajeEntrada>,
) -> Result<Json<MensajeSalida>, StatusCode> {
    // 1. Validamos el texto: no vacío y no demasiado largo (400 = "petición inválida").
    let texto = entrada.texto.trim();
    if texto.is_empty() || texto.chars().count() > 1000 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 2. Buscamos la conversación por su token; si no existe, 404 ("no encontrada").
    let conversacion = repositorio::buscar_por_token(&pool, &entrada.token)
        .await
        .map_err(error_interno)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // 3. Guardamos lo que escribió la persona ('P').
    repositorio::guardar_mensaje(&pool, &conversacion.codigo, "P", texto)
        .await
        .map_err(error_interno)?;

    // 4. El flujo decide qué responder (según el paso en que va la conversación) y guardamos
    //    la respuesta del agente ('A').
    let respuesta = flujo::responder(&pool, &conversacion, texto, entrada.accion.as_deref())
        .await
        .map_err(error_interno)?;
    repositorio::guardar_mensaje(&pool, &conversacion.codigo, "A", &respuesta.texto)
        .await
        .map_err(error_interno)?;

    Ok(Json(MensajeSalida {
        respuesta: respuesta.texto,
        opciones: respuesta.opciones,
        componente: respuesta.componente,
        espera: respuesta.espera,
    }))
}
