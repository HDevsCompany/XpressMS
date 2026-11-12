// sms.rs: envío de mensajes SMS. Lo usan todos los módulos que necesiten avisar por celular
// (hoy: el código de verificación del registro, en acceso/verificacion_telefono.rs).
//
// El envío se hace con Textbee (https://textbee.dev): un celular Android con una SIM peruana, con su app instalada,
// que actúa de "pasarela". Nuestro servidor le pide a la API de Textbee el envío y el celular manda el SMS.
// Como el SMS sale de ese teléfono, su saldo/paquete es el que se gasta, y si el teléfono está apagado, sin
// señal o sin datos, el SMS se demora o no sale (la API solo confirma que lo recibió para enviarlo).
//
// Configuración (archivo .env, o variables de entorno en Render):
//   TEXTBEE_API_KEY    la clave de la API de Textbee (SECRETA: nunca en el código ni en Git).
//   TEXTBEE_DEVICE_ID  el identificador de tu teléfono en Textbee. Opcional: si falta, Textbee usa el predeterminado.
//   TEXTBEE_API_URL    opcional; solo si Textbee cambia la dirección. Por defecto: URL_PREDETERMINADA.
//   SMS_MODO           "textbee" (predeterminado) o "simulado": en modo simulado NO se envía nada y el mensaje
//                      (con el código) sale en la terminal. Sirve para desarrollar sin gastar SMS. No usar en producción.
//
// Requiere la librería "reqwest" (cliente HTTP):  cargo add reqwest --features json

use std::fmt;
use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{json, Value};

// La dirección oficial para enviar un SMS (documentación: https://textbee.dev/docs/api-reference).
// El teléfono que envía se elige con "deviceId" dentro del cuerpo de la petición.
pub const URL_PREDETERMINADA: &str = "https://api.textbee.dev/api/v1/gateway/send-sms";

// Cuánto esperamos a Textbee antes de rendirnos (para que el chat no se quede colgado).
const ESPERA_CONEXION: Duration = Duration::from_secs(5);
const ESPERA_TOTAL: Duration = Duration::from_secs(10);

// Por qué no se pudo enviar.
#[derive(Debug, PartialEq, Clone)]
pub enum ErrorSms {
    NoConfigurado,          // falta TEXTBEE_API_KEY
    Red(String),            // no se pudo ni conectar con Textbee: el mensaje seguro NO salió
    SinRespuesta(String),   // la petición salió pero Textbee no contestó a tiempo: el mensaje PUEDE salir igual (respuesta diferida)
    Rechazado(u16, String), // Textbee respondió con un error (clave inválida, sin dispositivo, límite del plan...)
}

impl fmt::Display for ErrorSms {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSms::NoConfigurado => write!(f, "falta TEXTBEE_API_KEY en la configuración"),
            ErrorSms::Red(detalle) => write!(f, "sin conexión con Textbee: {}", detalle),
            ErrorSms::SinRespuesta(detalle) => write!(f, "Textbee no respondió a tiempo: {}", detalle),
            ErrorSms::Rechazado(estado, detalle) => write!(f, "Textbee rechazó el envío ({}): {}", estado, detalle),
        }
    }
}

enum Modo {
    Textbee,
    Simulado,
}

struct Configuracion {
    modo: Modo,
    url: String,
    clave: Option<String>,
    dispositivo: Option<String>,
}

// Lee la configuración una sola vez (el .env ya se cargó al arrancar, en config.rs).
fn configuracion() -> &'static Configuracion {
    static CONFIGURACION: OnceLock<Configuracion> = OnceLock::new();
    CONFIGURACION.get_or_init(|| {
        let leer = |nombre: &str| {
            std::env::var(nombre)
                .ok()
                .map(|valor| valor.trim().to_string())
                .filter(|valor| !valor.is_empty())
        };
        // En "cargo test" nunca se envía nada de verdad.
        let simulado = cfg!(test) || leer("SMS_MODO").is_some_and(|modo| modo.eq_ignore_ascii_case("simulado"));
        Configuracion {
            modo: if simulado { Modo::Simulado } else { Modo::Textbee },
            url: leer("TEXTBEE_API_URL").unwrap_or_else(|| URL_PREDETERMINADA.to_string()),
            clave: leer("TEXTBEE_API_KEY"),
            dispositivo: leer("TEXTBEE_DEVICE_ID"),
        }
    })
}

// Al arrancar el servidor: avisa en la terminal en qué modo está el envío de SMS y si falta algo.
// (Nunca muestra la clave.)
pub fn informar_estado() {
    let config = configuracion();
    match config.modo {
        Modo::Simulado => println!("SMS: modo SIMULADO (no se envía nada; los códigos salen aquí, en la terminal)."),
        Modo::Textbee if config.clave.is_none() => {
            println!("⚠ SMS: falta TEXTBEE_API_KEY en el .env; no se podrán enviar códigos de verificación.")
        }
        Modo::Textbee => match &config.dispositivo {
            Some(dispositivo) => println!("SMS: Textbee listo (teléfono {}).", dispositivo),
            None => println!("SMS: Textbee listo (teléfono predeterminado; puedes fijarlo con TEXTBEE_DEVICE_ID)."),
        },
    }
}

// El cliente HTTP se crea una sola vez y se reutiliza (así se reaprovechan las conexiones).
fn cliente() -> &'static reqwest::Client {
    static CLIENTE: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENTE.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(ESPERA_CONEXION)
            .timeout(ESPERA_TOTAL)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

// Envía un SMS. "telefono" debe venir en formato internacional E.164 ("+51987654321").
// Devuelve Ok cuando Textbee aceptó el mensaje (no garantiza que ya llegó: el celular pasarela lo envía enseguida).
pub async fn enviar(telefono: &str, mensaje: &str) -> Result<(), ErrorSms> {
    #[cfg(test)]
    {
        pruebas_envio::registrar(telefono, mensaje)?;
    }

    let config = configuracion();
    match config.modo {
        Modo::Simulado => {
            println!("[SMS simulado] a {}: {}", telefono, mensaje);
            Ok(())
        }
        Modo::Textbee => {
            let clave = config.clave.as_deref().ok_or(ErrorSms::NoConfigurado)?;
            let cuerpo = armar_cuerpo(telefono, mensaje, config.dispositivo.as_deref());

            let respuesta = cliente()
                .post(config.url.as_str())
                .header("x-api-key", clave)
                .json(&cuerpo)
                .send()
                .await
                .map_err(|error| {
                    // Si ni siquiera conectó, el SMS seguro no salió. Si conectó y no hubo respuesta a tiempo
                    // (o se cortó), Textbee pudo haberlo recibido igual: es una respuesta diferida.
                    if error.is_connect() {
                        ErrorSms::Red(error.to_string())
                    } else {
                        ErrorSms::SinRespuesta(error.to_string())
                    }
                })?;
            let estado = respuesta.status().as_u16();
            let texto = respuesta
                .text()
                .await
                .map_err(|error| ErrorSms::SinRespuesta(error.to_string()))?;

            // Un aviso de Textbee (por ejemplo, falta un permiso en el teléfono) no impide el envío, pero se anota.
            if let Some(aviso) = interpretar_respuesta(estado, &texto)? {
                eprintln!("Aviso de Textbee: {}", aviso);
            }
            Ok(())
        }
    }
}

// El cuerpo JSON de la petición: el mensaje, los destinatarios y (si se configuró) el teléfono que envía.
// Está separada de enviar() para poder probarla sin internet.
pub fn armar_cuerpo(telefono: &str, mensaje: &str, dispositivo: Option<&str>) -> Value {
    let mut cuerpo = json!({
        "message": mensaje,
        "recipients": [telefono],
    });
    if let Some(dispositivo) = dispositivo {
        cuerpo["deviceId"] = json!(dispositivo);
    }
    cuerpo
}

// Entiende la respuesta de Textbee. Devuelve Ok(None) si todo salió bien, Ok(Some(aviso)) si Textbee aceptó el
// mensaje pero avisó de algo, y Err si no se pudo enviar. También está separada para probarla sin internet.
pub fn interpretar_respuesta(estado: u16, cuerpo: &str) -> Result<Option<String>, ErrorSms> {
    let json: Option<Value> = serde_json::from_str(cuerpo).ok();
    // El texto de Textbee para explicar el error (si lo trae); si no, un trozo de la respuesta.
    let detalle = || -> String {
        json.as_ref()
            .and_then(|valor| valor.get("message").or_else(|| valor.pointer("/data/message")))
            .and_then(|valor| valor.as_str().map(|texto| texto.to_string()))
            .unwrap_or_else(|| cuerpo.chars().take(200).collect())
    };

    if !(200..300).contains(&estado) {
        let motivo = match estado {
            401 => "la clave de la API no es válida o fue revocada".to_string(),
            403 => "la cuenta no puede enviar mensajes".to_string(),
            429 => format!("se agotó el límite del plan ({})", detalle()),
            _ => detalle(),
        };
        return Err(ErrorSms::Rechazado(estado, motivo));
    }

    // Respuesta 2xx: Textbee la devuelve dentro de "data". Si trajera otra forma, no lo tratamos como error
    // (un 2xx significa que lo aceptó), pero dejamos un aviso en la terminal para revisarlo.
    let Some(datos) = json.as_ref().and_then(|valor| valor.get("data")) else {
        let resumen: String = cuerpo.chars().take(200).collect();
        return Ok(Some(format!("respuesta sin «data»: {}", resumen)));
    };
    if datos.get("success").and_then(|valor| valor.as_bool()) == Some(false) {
        return Err(ErrorSms::Rechazado(estado, detalle()));
    }
    // Si respondió sin encolar, trae cuántos se enviaron y cuántos fallaron.
    let exitos = datos.get("successCount").and_then(|valor| valor.as_u64());
    let fallos = datos.get("failureCount").and_then(|valor| valor.as_u64());
    if exitos == Some(0) && fallos.is_some_and(|f| f > 0) {
        return Err(ErrorSms::Rechazado(estado, "el teléfono no pudo enviar el mensaje".to_string()));
    }

    let aviso = datos.get("warning").map(|aviso| {
        let codigo = aviso.get("code").and_then(|v| v.as_str()).unwrap_or("?");
        let texto = aviso.get("message").and_then(|v| v.as_str()).unwrap_or("");
        format!("{} {}", codigo, texto).trim().to_string()
    });
    Ok(aviso)
}

// Ayudas solo para las pruebas: anotan lo que se "envió" y permiten simular un fallo.
#[cfg(test)]
pub mod pruebas_envio {
    use super::ErrorSms;
    use std::cell::RefCell;

    thread_local! {
        static ENVIADOS: RefCell<Vec<(String, String)>> = const { RefCell::new(Vec::new()) };
        static FALLAR: RefCell<Option<ErrorSms>> = const { RefCell::new(None) };
    }

    pub(super) fn registrar(telefono: &str, mensaje: &str) -> Result<(), ErrorSms> {
        if let Some(error) = FALLAR.with(|f| f.borrow().clone()) {
            return Err(error);
        }
        ENVIADOS.with(|e| e.borrow_mut().push((telefono.to_string(), mensaje.to_string())));
        Ok(())
    }

    // Lo enviado hasta ahora: (teléfono, mensaje).
    pub fn enviados() -> Vec<(String, String)> {
        ENVIADOS.with(|e| e.borrow().clone())
    }

    pub fn limpiar() {
        ENVIADOS.with(|e| e.borrow_mut().clear());
        FALLAR.with(|f| *f.borrow_mut() = None);
    }

    // Desde ahora cada envío falla con este error (None = vuelve a funcionar).
    pub fn fallar_con(error: Option<ErrorSms>) {
        FALLAR.with(|f| *f.borrow_mut() = error);
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_cuerpo_lleva_destinatarios_y_mensaje() {
        let cuerpo = armar_cuerpo("+51987654321", "Hola", None);
        assert_eq!(cuerpo["message"], "Hola");
        assert_eq!(cuerpo["recipients"], json!(["+51987654321"]));
        assert!(cuerpo.get("deviceId").is_none());
    }

    #[test]
    fn el_cuerpo_lleva_el_telefono_pasarela_si_se_configuro() {
        let cuerpo = armar_cuerpo("+51987654321", "Hola", Some("6aeb88fa14743c91efcee8f9"));
        assert_eq!(cuerpo["deviceId"], "6aeb88fa14743c91efcee8f9");
    }

    #[test]
    fn un_envio_encolado_es_correcto() {
        let cuerpo = r#"{"data":{"success":true,"message":"SMS added to queue","smsBatchId":"abc","recipientCount":1}}"#;
        assert_eq!(interpretar_respuesta(201, cuerpo), Ok(None));
    }

    #[test]
    fn un_envio_inmediato_es_correcto() {
        let cuerpo = r#"{"data":{"successCount":1,"failureCount":0}}"#;
        assert_eq!(interpretar_respuesta(200, cuerpo), Ok(None));
    }

    #[test]
    fn un_aviso_no_impide_el_envio() {
        let cuerpo = r#"{"data":{"success":true,"warning":{"code":"SMS_PERMISSION_MISSING","message":"Falta el permiso"}}}"#;
        assert_eq!(
            interpretar_respuesta(200, cuerpo),
            Ok(Some("SMS_PERMISSION_MISSING Falta el permiso".to_string()))
        );
    }

    #[test]
    fn success_en_falso_es_un_error() {
        let cuerpo = r#"{"data":{"success":false,"message":"Device offline"}}"#;
        assert_eq!(
            interpretar_respuesta(200, cuerpo),
            Err(ErrorSms::Rechazado(200, "Device offline".to_string()))
        );
    }

    #[test]
    fn si_todos_fallaron_es_un_error() {
        let cuerpo = r#"{"data":{"successCount":0,"failureCount":1}}"#;
        assert!(matches!(interpretar_respuesta(200, cuerpo), Err(ErrorSms::Rechazado(200, _))));
    }

    #[test]
    fn los_errores_http_se_explican() {
        assert!(matches!(interpretar_respuesta(401, r#"{"message":"Unauthorized"}"#), Err(ErrorSms::Rechazado(401, m)) if m.contains("clave")));
        assert!(matches!(interpretar_respuesta(403, ""), Err(ErrorSms::Rechazado(403, _))));
        assert!(matches!(interpretar_respuesta(429, r#"{"message":"Daily limit reached"}"#), Err(ErrorSms::Rechazado(429, m)) if m.contains("Daily limit reached")));
        assert!(matches!(interpretar_respuesta(400, r#"{"message":"No enabled device"}"#), Err(ErrorSms::Rechazado(400, m)) if m == "No enabled device"));
        assert!(matches!(interpretar_respuesta(500, "<html>boom</html>"), Err(ErrorSms::Rechazado(500, m)) if m.contains("boom")));
    }

    #[test]
    fn una_respuesta_2xx_con_otra_forma_se_acepta_con_aviso() {
        assert!(matches!(interpretar_respuesta(200, "no es json"), Ok(Some(aviso)) if aviso.contains("sin «data»")));
        assert!(matches!(interpretar_respuesta(201, r#"{"otra":1}"#), Ok(Some(_))));
    }

    #[test]
    fn el_mensaje_de_error_no_contiene_la_clave() {
        let error = ErrorSms::Rechazado(401, "la clave de la API no es válida o fue revocada".to_string());
        assert!(!error.to_string().contains("sk_"));
    }
}
