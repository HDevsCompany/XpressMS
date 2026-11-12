// rutas.rs (agente): las "puertas" del chat. Solo reciben JSON, llaman al repositorio y al flujo,
// y devuelven JSON. No contienen SQL ni textos del agente.
//
// Hay dos puertas:
//   POST /agente/iniciar  -> al abrir el chat: crea la conversación (o recupera la anterior) y
//                            devuelve el token y los mensajes que ya existían.
//   POST /agente/sesion   -> el lienzo pregunta si ese token ya es de un cliente reconocido y cómo se llama
//                            (para saludarlo con su nombre y mostrar su avatar).
//   POST /agente/nueva    -> «Nueva conversación» de un cliente reconocido: empieza una conversación limpia
//                            sin perder su identificación.
//   POST /agente/mensaje  -> cada vez que la persona escribe: guarda su mensaje, responde y guarda la respuesta.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::identidad::{self, Agente};
use super::modelo::{Componente, MensajeGuardado};
use super::identificacion::{self, Resultado};
use super::{flujo, flujo_carrito, flujo_solicitud, repositorio};
use crate::modulos::acceso::repositorio as acceso_repo;
use crate::modulos::comun::formato;

// Lo que envía el navegador al abrir la página. "token" es Option porque la primera vez no existe.
#[derive(Deserialize)]
struct IniciarEntrada {
    token: Option<String>,
    // Por dónde llegó: "registro" (pulsó «Regístrate con Juno») o nada / "mirar". Decide el saludo de Juno.
    motivo: Option<String>,
}

// Lo que respondemos al abrir: el token (el navegador lo guarda), el historial y los botones.
#[derive(Serialize)]
struct IniciarSalida {
    token: String,
    agente: Agente, // nombre e icono con los que se presenta el agente
    mensajes: Vec<MensajeGuardado>,
    opciones: Vec<String>,
    componente: Option<Componente>, // elemento visual pendiente (la casilla de términos, si quedó en ese paso)
    carrito: Option<Componente>,    // el carrito guardado de la tienda, para dibujarlo al recargar la página
    solicitud: Option<Componente>,  // la solicitud a medias de /servicios, para dibujar «Tu solicitud» al recargar la página
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
    solicitud: Option<Componente>,  // el estado de «Tu solicitud» (solo en la ventana de Juno de /servicios)
}

// La página principal pregunta si el navegador ya tiene una conversación con un cliente reconocido.
#[derive(Deserialize)]
struct SesionEntrada {
    token: Option<String>,
}

#[derive(Serialize)]
struct SesionSalida {
    reconocido: bool,
    // Cómo saludarlo ("Pedro"). Solo viaja si el cliente está reconocido; sirve para el «Hola, Pedro» y la inicial del avatar.
    #[serde(skip_serializing_if = "Option::is_none")]
    nombre: Option<String>,
}

// «Nueva conversación» de un cliente reconocido: el navegador manda el token de su conversación actual.
#[derive(Deserialize)]
struct NuevaEntrada {
    token: Option<String>,
}

#[derive(Serialize)]
struct NuevaSalida {
    token: String,
}

// Lo que envía la página de ingreso cuando un cliente quiere que lo reconozcan.
#[derive(Deserialize)]
struct IdentificarEntrada {
    token: Option<String>, // la conversación que ya tenía el navegador (si la hay)
    tipo: String,          // código del tipo de documento: "1" DNI, "6" RUC...
    numero: String,
    ultimos: String, // últimos 3 dígitos del celular registrado
}

// Si se reconoce al cliente, se devuelve el token de su conversación; el navegador lo guarda.
#[derive(Serialize)]
struct IdentificarSalida {
    token: String,
}

// Un texto para mostrarle a la persona cuando algo no salió bien.
#[derive(Serialize)]
struct Aviso {
    mensaje: String,
}

// Agrupa las rutas del agente; main.rs las suma con ".merge(...)".
pub fn rutas() -> Router<PgPool> {
    Router::new()
        .route("/agente/iniciar", post(iniciar))
        .route("/agente/mensaje", post(mensaje))
        .route("/agente/sesion", post(sesion))
        .route("/agente/nueva", post(nueva))
        .route("/agente/identificar", post(identificar))
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

    // 2. Usamos la conversación existente o creamos una nueva. En ambos casos Juno deja el saludo que
    //    corresponde a cómo llegó la persona (ver flujo::dar_bienvenida).
    let entrada_chat = flujo::Entrada::desde_texto(entrada.motivo.as_deref());
    let (mut conversacion, es_nueva) = match existente {
        Some(conversacion) => (conversacion, false),
        None => (repositorio::crear_conversacion(&pool).await.map_err(error_interno)?, true),
    };
    // La ventana de Juno en la tienda (motivo «catalogo») no recibe saludo: es una conversación aparte, solo para la vitrina.
    let en_tienda = entrada.motivo.as_deref() == Some("catalogo");
    // Igual la ventana de Juno de /servicios (motivo «servicios»): conversación aparte, sin saludo, con «Tu solicitud».
    let en_servicios = entrada.motivo.as_deref() == Some("servicios");
    if en_tienda {
        flujo::preparar_catalogo(&pool, &mut conversacion).await.map_err(error_interno)?;
    } else if en_servicios {
        flujo::preparar_servicios(&pool, &mut conversacion).await.map_err(error_interno)?;
    } else {
        flujo::dar_bienvenida(&pool, &mut conversacion, entrada_chat, es_nueva)
            .await
            .map_err(error_interno)?;
    }

    // 3. Traemos el historial para que la persona vea su conversación aunque recargue la página.
    let mensajes = repositorio::listar_mensajes(&pool, &conversacion.codigo)
        .await
        .map_err(error_interno)?;

    // 4. Si quedó en medio del registro, se vuelve a dibujar la casilla de términos; si la conversación
    //    recién empieza, se ofrecen los botones del menú.
    let componente = flujo::componente_de_reanudacion(&conversacion.paso);
    let opciones = if en_tienda || en_servicios { Vec::new() } else { flujo::opciones_de_reanudacion(&conversacion.paso, mensajes.len()) };

    // En la tienda, el carrito que Juno tenía guardado vuelve a dibujarse (si la persona recarga o regresa otro día).
    let carrito = if en_tienda {
        flujo_carrito::reanudar(&pool, &conversacion.datos).await.map_err(error_interno)?
    } else {
        None
    };

    // En /servicios, la solicitud que quedó a medias vuelve a dibujarse (con el día y la hora que ya se habían elegido).
    let solicitud = if en_servicios {
        flujo_solicitud::reanudar(&pool, &conversacion).await.map_err(error_interno)?
    } else {
        None
    };

    Ok(Json(IniciarSalida {
        token: conversacion.token,
        agente: identidad::actual(),
        mensajes,
        opciones,
        componente,
        carrito,
        solicitud,
    }))
}

// POST /agente/sesion
// Solo responde si el token pertenece a un cliente ya reconocido; no revela nada más que su nombre de pila
// (el mismo que la persona ya ve al entrar: «Hola, Pedro»).
async fn sesion(
    State(pool): State<PgPool>,
    Json(entrada): Json<SesionEntrada>,
) -> Result<Json<SesionSalida>, StatusCode> {
    let cliente = match entrada.token {
        Some(token) => repositorio::buscar_por_token(&pool, &token)
            .await
            .map_err(error_interno)?
            .and_then(|conversacion| conversacion.cliente),
        None => None,
    };

    let nombre = match cliente.as_deref() {
        Some(codigo) => acceso_repo::obtener_cliente(&pool, codigo)
            .await
            .map_err(error_interno)?
            .map(|datos| nombre_de_pila(datos.nombres.as_deref(), datos.razon_social.as_deref())),
        None => None,
    };

    Ok(Json(SesionSalida { reconocido: cliente.is_some(), nombre }))
}

// El nombre con el que Juno saluda: la primera palabra de los nombres ("PEDRO ANTONIO" -> "Pedro").
// Una empresa no tiene nombres de pila: se usa su razón social completa ("COMERCIAL ABC SAC" -> "Comercial Abc Sac").
fn nombre_de_pila(nombres: Option<&str>, razon_social: Option<&str>) -> String {
    let primero = nombres
        .and_then(|texto| texto.split_whitespace().next())
        .map(formato::a_titulo);
    match primero {
        Some(nombre) if !nombre.is_empty() => nombre,
        _ => formato::a_titulo(razon_social.unwrap_or("")),
    }
}

// POST /agente/nueva
// «Nueva conversación» de un cliente reconocido: se crea otra conversación ya vinculada a él (no tiene que volver a
// identificarse) y se devuelve su token. Si el token no es de un cliente reconocido, no se hace nada: un visitante
// simplemente olvida su conversación en el navegador.
async fn nueva(
    State(pool): State<PgPool>,
    Json(entrada): Json<NuevaEntrada>,
) -> Result<Json<NuevaSalida>, StatusCode> {
    let cliente = match entrada.token {
        Some(token) => repositorio::buscar_por_token(&pool, &token)
            .await
            .map_err(error_interno)?
            .and_then(|conversacion| conversacion.cliente),
        None => None,
    };
    let Some(cliente) = cliente else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let token = identificacion::entrar_al_chat(&pool, None, &cliente)
        .await
        .map_err(error_interno)?;
    Ok(Json(NuevaSalida { token }))
}

// POST /agente/mensaje
async fn mensaje(
    State(pool): State<PgPool>,
    cabeceras: HeaderMap,
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
    //    La red de origen sirve para limitar cuántos códigos SMS puede pedir una misma conexión.
    let red = red_de_origen(&cabeceras);
    let respuesta = flujo::responder(&pool, &conversacion, texto, entrada.accion.as_deref(), &red)
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
        solicitud: respuesta.solicitud,
    }))
}

// Dirección de internet de quien hace la petición, para limitar los intentos repetidos.
// Detrás del servidor de Render, la última dirección de "x-forwarded-for" es la que agrega Render
// (la real de la persona); las anteriores las puede escribir cualquiera, por eso no se usan.
fn red_de_origen(cabeceras: &HeaderMap) -> String {
    cabeceras
        .get("x-forwarded-for")
        .and_then(|valor| valor.to_str().ok())
        .and_then(|lista| lista.rsplit(',').next())
        .map(|direccion| direccion.trim().to_string())
        .filter(|direccion| !direccion.is_empty())
        .unwrap_or_else(|| "local".to_string())
}

// POST /agente/identificar
// La página de ingreso envía documento + últimos 3 dígitos; si coinciden, la conversación queda vinculada
// al cliente y la persona entra al chat ya reconocida.
async fn identificar(
    State(pool): State<PgPool>,
    cabeceras: HeaderMap,
    Json(entrada): Json<IdentificarEntrada>,
) -> Response {
    let red = red_de_origen(&cabeceras);
    let resultado =
        identificacion::identificar(&pool, &entrada.tipo, &entrada.numero, &entrada.ultimos, &red).await;

    match resultado {
        Err(error) => error_interno(error).into_response(),
        Ok(Resultado::Invalido(mensaje)) => {
            (StatusCode::BAD_REQUEST, Json(Aviso { mensaje })).into_response()
        }
        // Siempre el mismo texto: no se revela si el documento está registrado o no.
        Ok(Resultado::NoVerificado) => (
            StatusCode::UNAUTHORIZED,
            Json(Aviso {
                mensaje: "No pudimos verificar tus datos. Revisa el documento y los últimos 3 dígitos de tu celular."
                    .to_string(),
            }),
        )
            .into_response(),
        Ok(Resultado::Bloqueado) => (
            StatusCode::TOO_MANY_REQUESTS,
            Json(Aviso {
                mensaje: "Demasiados intentos. Inténtalo de nuevo en unos minutos.".to_string(),
            }),
        )
            .into_response(),
        Ok(Resultado::Reconocido(cliente)) => {
            match identificacion::entrar_al_chat(&pool, entrada.token.as_deref(), &cliente).await {
                Ok(token) => Json(IdentificarSalida { token }).into_response(),
                Err(error) => error_interno(error).into_response(),
            }
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn saluda_con_el_primer_nombre() {
        assert_eq!(nombre_de_pila(Some("PEDRO ANTONIO"), None), "Pedro");
        assert_eq!(nombre_de_pila(Some("  maría  josé "), None), "María");
    }

    #[test]
    fn una_empresa_usa_su_razon_social() {
        assert_eq!(nombre_de_pila(None, Some("COMERCIAL ABC SAC")), "Comercial Abc Sac");
        assert_eq!(nombre_de_pila(Some(""), Some("TECNO SUR EIRL")), "Tecno Sur Eirl");
    }

    #[test]
    fn sin_datos_no_inventa_un_nombre() {
        assert_eq!(nombre_de_pila(None, None), "");
    }
}
