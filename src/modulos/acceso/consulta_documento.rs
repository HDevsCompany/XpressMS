// consulta_documento.rs: obtiene los datos de una persona o empresa a partir de su DNI o RUC.
//
// La consulta real se hace con la API de Factiliza (https://docs.factiliza.com, sección «API Consulta»):
//   GET https://api.factiliza.com/v1/dni/info/{dni}    -> nombres, apellidos, dirección, fecha de nacimiento, sexo
//   GET https://api.factiliza.com/v1/ruc/info/{ruc}    -> razón social y dirección
// Se envía el token en la cabecera "Authorization: Bearer <token>". El plan gratuito tiene un número limitado
// de consultas, por eso solo se consulta cuando alguien escribe un documento NUEVO en el registro.
//
// Configuración (archivo .env, o variables de entorno en Render):
//   FACTILIZA_TOKEN     el token de la API (SECRETO: nunca en el código ni en Git).
//   FACTILIZA_API_URL   opcional; solo si Factiliza cambia su dirección. Por defecto: URL_BASE.
//   CONSULTA_MODO       "factiliza" (predeterminado) o "simulado": datos inventados, sin gastar consultas.
//                       Sirve solo para desarrollar. NO usar con clientes reales: guardaría datos falsos.
//
// Si la consulta falla (sin token, sin internet, documento no encontrado, consultas agotadas...), devuelve None y el
// chat sigue pidiéndole los datos a la persona a mano: el registro nunca se queda trabado por esta consulta.
// El resto del sistema no conoce a Factiliza: siempre recibe una "PersonaConsultada".
//
// Requiere la librería "reqwest" (la misma que usa el envío de SMS).

use std::sync::OnceLock;
use std::time::Duration;

use serde_json::Value;

use crate::modulos::comun::validaciones::{normalizar_fecha, TIPO_DNI, TIPO_RUC};

// La dirección base de la API de Factiliza.
pub const URL_BASE: &str = "https://api.factiliza.com/v1";

// Cuánto esperamos a Factiliza antes de rendirnos (para que el chat no se quede colgado).
const ESPERA_CONEXION: Duration = Duration::from_secs(5);
const ESPERA_TOTAL: Duration = Duration::from_secs(10);

// Los datos que devuelve la consulta. Todo es opcional porque la fuente puede no traerlo.
pub struct PersonaConsultada {
    pub nombres: Option<String>,
    pub paterno: Option<String>,
    pub materno: Option<String>,
    pub razon_social: Option<String>,
    pub direccion: Option<String>,
    pub sexo: Option<String>,             // 'M' o 'F'
    pub fecha_nacimiento: Option<String>, // AAAA-MM-DD
}

// Por qué no se obtuvieron los datos (solo sirve para dejar el motivo en la terminal del servidor).
#[derive(Debug, PartialEq)]
pub enum MotivoFallo {
    NoEncontrado,  // ese documento no existe (la persona escribe sus datos a mano)
    TokenInvalido, // falta el token, es incorrecto o fue revocado
    LimiteAgotado, // se acabaron las consultas del plan
    Otro(String),  // cualquier otra cosa (error del servidor de Factiliza, respuesta rara...)
}

enum Modo {
    Factiliza,
    Simulado,
}

struct Configuracion {
    modo: Modo,
    url_base: String,
    token: Option<String>,
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
        // En "cargo test" nunca se consulta nada de verdad.
        let simulado = cfg!(test) || leer("CONSULTA_MODO").is_some_and(|modo| modo.eq_ignore_ascii_case("simulado"));
        Configuracion {
            modo: if simulado { Modo::Simulado } else { Modo::Factiliza },
            url_base: leer("FACTILIZA_API_URL").unwrap_or_else(|| URL_BASE.to_string()),
            token: leer("FACTILIZA_TOKEN"),
        }
    })
}

// Al arrancar el servidor: avisa en la terminal cómo se consultarán los documentos. (Nunca muestra el token.)
pub fn informar_estado() {
    let config = configuracion();
    match config.modo {
        Modo::Simulado => println!("Consulta de DNI/RUC: modo SIMULADO (datos inventados; no usar con clientes reales)."),
        Modo::Factiliza if config.token.is_none() => {
            println!("⚠ Consulta de DNI/RUC: falta FACTILIZA_TOKEN en el .env; los datos se pedirán a mano.")
        }
        Modo::Factiliza => println!("Consulta de DNI/RUC: Factiliza listo."),
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

// Consulta un documento. Devuelve None si no se encontró o no se pudo consultar (la persona escribe sus datos a mano).
pub async fn consultar(tipo: &str, numero: &str) -> Option<PersonaConsultada> {
    let config = configuracion();
    match config.modo {
        Modo::Simulado => simulada(tipo, numero),
        Modo::Factiliza => match consultar_factiliza(config, tipo, numero).await {
            Ok(persona) => Some(persona),
            Err(MotivoFallo::NoEncontrado) => None, // lo normal: el documento no existe. No es un problema del sistema.
            Err(motivo) => {
                eprintln!("[Consulta DNI/RUC] No se pudo consultar: {}", describir(&motivo));
                None
            }
        },
    }
}

fn describir(motivo: &MotivoFallo) -> String {
    match motivo {
        MotivoFallo::NoEncontrado => "documento no encontrado".to_string(),
        MotivoFallo::TokenInvalido => {
            "el token de Factiliza falta, es inválido o fue revocado (revisa FACTILIZA_TOKEN)".to_string()
        }
        MotivoFallo::LimiteAgotado => "se agotaron las consultas del plan de Factiliza".to_string(),
        MotivoFallo::Otro(detalle) => detalle.clone(),
    }
}

async fn consultar_factiliza(config: &Configuracion, tipo: &str, numero: &str) -> Result<PersonaConsultada, MotivoFallo> {
    let ruta = if tipo == TIPO_DNI {
        "dni"
    } else if tipo == TIPO_RUC {
        "ruc"
    } else {
        return Err(MotivoFallo::NoEncontrado); // carnet de extranjería y pasaporte no tienen consulta pública
    };
    // El número va dentro de la dirección web: solo se aceptan dígitos (así nadie puede colar otra ruta).
    if numero.is_empty() || !numero.chars().all(|c| c.is_ascii_digit()) {
        return Err(MotivoFallo::NoEncontrado);
    }
    let token = config.token.as_deref().ok_or(MotivoFallo::TokenInvalido)?;

    let url = format!("{}/{}/info/{}", config.url_base.trim_end_matches('/'), ruta, numero);
    let respuesta = cliente()
        .get(url.as_str())
        .bearer_auth(token)
        .send()
        .await
        .map_err(|error| MotivoFallo::Otro(format!("sin conexión con Factiliza: {}", error)))?;
    let estado = respuesta.status().as_u16();
    let cuerpo = respuesta
        .text()
        .await
        .map_err(|error| MotivoFallo::Otro(format!("Factiliza no terminó de responder: {}", error)))?;

    interpretar_respuesta(tipo, estado, &cuerpo)
}

// ---------- Entender la respuesta (funciones puras: se prueban sin internet) ----------

// Convierte la respuesta de Factiliza en una PersonaConsultada, o explica por qué no se pudo.
pub fn interpretar_respuesta(tipo: &str, estado: u16, cuerpo: &str) -> Result<PersonaConsultada, MotivoFallo> {
    let json: Option<Value> = serde_json::from_str(cuerpo).ok();
    let mensaje = json
        .as_ref()
        .and_then(|valor| valor.get("message"))
        .and_then(|valor| valor.as_str())
        .unwrap_or("")
        .to_string();

    if !(200..300).contains(&estado) {
        return Err(match estado {
            401 | 403 => MotivoFallo::TokenInvalido,
            429 => MotivoFallo::LimiteAgotado,
            404 => MotivoFallo::NoEncontrado,
            // Factiliza responde 400 tanto cuando el documento no existe como en otros errores:
            // el texto del mensaje ayuda a distinguirlos.
            _ => clasificar_mensaje(estado, &mensaje),
        });
    }

    let json = json.ok_or_else(|| MotivoFallo::Otro("la respuesta de Factiliza no es JSON".to_string()))?;
    if json.get("success").and_then(|valor| valor.as_bool()) == Some(false) {
        return Err(clasificar_mensaje(estado, &mensaje));
    }
    let datos = match json.get("data") {
        Some(datos)
            if datos.get("numero").is_some()
                || datos.get("nombres").is_some()
                || datos.get("nombre_o_razon_social").is_some() =>
        {
            datos
        }
        _ => return Err(MotivoFallo::NoEncontrado),
    };

    let persona = if tipo == TIPO_DNI {
        PersonaConsultada {
            nombres: texto(datos, "nombres", 100),
            paterno: texto(datos, "apellido_paterno", 100),
            materno: texto(datos, "apellido_materno", 100),
            razon_social: None,
            direccion: direccion(datos),
            sexo: texto(datos, "sexo", 30).and_then(|sexo| normalizar_sexo(&sexo)),
            fecha_nacimiento: texto(datos, "fecha_nacimiento", 30).and_then(|fecha| normalizar_fecha(&fecha)),
        }
    } else if tipo == TIPO_RUC {
        PersonaConsultada {
            nombres: None,
            paterno: None,
            materno: None,
            razon_social: texto(datos, "nombre_o_razon_social", 200),
            direccion: direccion(datos),
            sexo: None,
            fecha_nacimiento: None,
        }
    } else {
        return Err(MotivoFallo::NoEncontrado);
    };

    if persona.nombres.is_some() || persona.razon_social.is_some() {
        Ok(persona)
    } else {
        Err(MotivoFallo::NoEncontrado)
    }
}

// Un error de Factiliza sin código claro: se decide por lo que dice el mensaje.
fn clasificar_mensaje(estado: u16, mensaje: &str) -> MotivoFallo {
    let m = mensaje.to_lowercase();
    if m.contains("token") || m.contains("autoriz") || m.contains("unauthorized") || m.contains("api key") {
        MotivoFallo::TokenInvalido
    } else if m.contains("límite")
        || m.contains("limite")
        || m.contains("cuota")
        || m.contains("agotad")
        || m.contains("excedi")
        || m.contains("limit")
    {
        MotivoFallo::LimiteAgotado
    } else if estado == 400 || m.contains("no se encontr") || m.contains("no encontr") || m.contains("not found") {
        MotivoFallo::NoEncontrado
    } else {
        MotivoFallo::Otro(format!(
            "Factiliza respondió {}: {}",
            estado,
            mensaje.chars().take(150).collect::<String>()
        ))
    }
}

// Un campo de texto de la respuesta: sin espacios de más, en MAYÚSCULAS, y None si viene vacío o no es texto.
fn texto(datos: &Value, campo: &str, maximo: usize) -> Option<String> {
    let crudo = datos.get(campo)?.as_str()?;
    let limpio = crudo.split_whitespace().collect::<Vec<_>>().join(" ").to_uppercase();
    if limpio.is_empty() || limpio == "-" {
        None
    } else {
        Some(limpio.chars().take(maximo).collect())
    }
}

// "CASERIO PUÑA, CAJAMARCA - CHOTA - TACABAMBA": la calle y, al final, la zona (departamento - provincia - distrito).
// Así el chat puede mostrar solo la zona y ocultar la calle (ver formato::direccion_enmascarada).
// En la base la dirección admite 120 caracteres: si no cabe, se acorta la calle y se conserva la zona.
fn direccion(datos: &Value) -> Option<String> {
    let calle = texto(datos, "direccion", 120)?;
    let zona = ["departamento", "provincia", "distrito"]
        .iter()
        .filter_map(|campo| texto(datos, campo, 40))
        .collect::<Vec<_>>()
        .join(" - ");
    if zona.is_empty() {
        return Some(calle);
    }
    let reservado = zona.chars().count() + 2; // la zona y el ", " que la separa de la calle
    let calle: String = calle.chars().take(120usize.saturating_sub(reservado)).collect();
    let calle = calle.trim_end_matches([',', ' ']).to_string();
    if calle.is_empty() {
        None
    } else {
        Some(format!("{}, {}", calle, zona))
    }
}

// "M", "Masculino", "HOMBRE" o "1" -> "M"; "F", "Femenino", "MUJER" o "2" -> "F"; otra cosa -> None.
fn normalizar_sexo(texto: &str) -> Option<String> {
    match texto.trim().to_uppercase().as_str() {
        "M" | "MASCULINO" | "HOMBRE" | "1" => Some("M".to_string()),
        "F" | "FEMENINO" | "MUJER" | "2" => Some("F".to_string()),
        _ => None,
    }
}

// ---------- Modo simulado (solo para desarrollar) ----------

// Regla de la simulación: los documentos que terminan en 0 "no se encuentran".
// Sirve para probar el camino de ingreso manual.
fn simulada(tipo: &str, numero: &str) -> Option<PersonaConsultada> {
    if numero.ends_with('0') {
        return None;
    }

    if tipo == TIPO_DNI {
        Some(PersonaConsultada {
            nombres: Some("JUAN CARLOS".to_string()),
            paterno: Some("PEREZ".to_string()),
            materno: Some("GOMEZ".to_string()),
            razon_social: None,
            direccion: Some("AV. ESPAÑA 123 - TRUJILLO".to_string()),
            sexo: Some("M".to_string()),
            fecha_nacimiento: Some("1995-03-15".to_string()),
        })
    } else if tipo == TIPO_RUC {
        Some(PersonaConsultada {
            nombres: None,
            paterno: None,
            materno: None,
            razon_social: Some("EMPRESA DE PRUEBA S.A.C.".to_string()),
            direccion: Some("CAL. LOS PINOS 456 - TRUJILLO".to_string()),
            sexo: None,
            fecha_nacimiento: None,
        })
    } else {
        None // carnet de extranjería y pasaporte no tienen consulta pública
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    // Los ejemplos de la documentación oficial de Factiliza.
    const DNI_EJEMPLO: &str = r#"{"status":200,"success":true,"message":"Exito","data":{"numero":"27427864","nombres":"JOSE PEDRO","apellido_paterno":"CASTILLO","apellido_materno":"TERRONES","nombre_completo":"CASTILLO TERRONES, JOSE PEDRO","departamento":"CAJAMARCA","provincia":"CHOTA","distrito":"TACABAMBA","direccion":"CASERIO PUÑA","direccion_completa":"CASERIO PUÑA, CAJAMARCA - CHOTA - TACABAMBA","ubigeo_reniec":"060615","ubigeo_sunat":"060417","ubigeo":["06","0604","060417"],"fecha_nacimiento":"","sexo":""}}"#;
    const RUC_EJEMPLO: &str = r#"{"status":200,"success":true,"message":"Exito","data":{"numero":"20552103816","nombre_o_razon_social":"AGROLIGHT PERU S.A.C.","tipo_contribuyente":"SOCIEDAD ANONIMA CERRADA","estado":"SUSPENSION TEMPORAL","condicion":"HABIDO","departamento":"LIMA","provincia":"LIMA","distrito":"SANTA ANITA","direccion":"PJ. JORGE BASADRE NRO. 158 URB. POP LA UNIVERSAL 2DA ET.","direccion_completa":"PJ. JORGE BASADRE NRO. 158 URB. POP LA UNIVERSAL 2DA ET., LIMA - LIMA - SANTA ANITA","ubigeo_sunat":"150137","ubigeo":["15","1501","150137"]}}"#;

    #[test]
    fn el_dni_del_ejemplo_se_entiende() {
        let p = interpretar_respuesta(TIPO_DNI, 200, DNI_EJEMPLO).ok().unwrap();
        assert_eq!(p.nombres.as_deref(), Some("JOSE PEDRO"));
        assert_eq!(p.paterno.as_deref(), Some("CASTILLO"));
        assert_eq!(p.materno.as_deref(), Some("TERRONES"));
        assert_eq!(p.direccion.as_deref(), Some("CASERIO PUÑA, CAJAMARCA - CHOTA - TACABAMBA"));
        // Fecha y sexo vacíos en la respuesta: quedan sin valor (no se inventa nada).
        assert_eq!(p.sexo, None);
        assert_eq!(p.fecha_nacimiento, None);
        assert_eq!(p.razon_social, None);
    }

    #[test]
    fn la_zona_de_la_direccion_es_lo_ultimo_para_poder_enmascarar_la_calle() {
        let p = interpretar_respuesta(TIPO_DNI, 200, DNI_EJEMPLO).ok().unwrap();
        let enmascarada = crate::modulos::comun::formato::direccion_enmascarada(p.direccion.as_deref().unwrap());
        assert_eq!(enmascarada, "•••••• - TACABAMBA");
    }

    #[test]
    fn el_ruc_del_ejemplo_se_entiende() {
        let p = interpretar_respuesta(TIPO_RUC, 200, RUC_EJEMPLO).ok().unwrap();
        assert_eq!(p.razon_social.as_deref(), Some("AGROLIGHT PERU S.A.C."));
        assert_eq!(p.nombres, None);
        assert_eq!(p.sexo, None);
        assert!(p.direccion.as_deref().unwrap().ends_with(", LIMA - LIMA - SANTA ANITA"));
        assert!(p.direccion.as_deref().unwrap().chars().count() <= 120);
    }

    #[test]
    fn fecha_y_sexo_cuando_si_vienen() {
        let cuerpo = r#"{"success":true,"data":{"numero":"1","nombres":" ana  maría ","apellido_paterno":"diaz","fecha_nacimiento":"1995-03-15","sexo":"Femenino"}}"#;
        let p = interpretar_respuesta(TIPO_DNI, 200, cuerpo).ok().unwrap();
        assert_eq!(p.nombres.as_deref(), Some("ANA MARÍA"));
        assert_eq!(p.materno, None);
        assert_eq!(p.sexo.as_deref(), Some("F"));
        assert_eq!(p.fecha_nacimiento.as_deref(), Some("1995-03-15"));
        assert_eq!(p.direccion, None); // sin dirección: el chat la pregunta
    }

    #[test]
    fn formatos_de_sexo() {
        for (entrada, esperado) in [
            ("M", Some("M")),
            ("masculino", Some("M")),
            ("1", Some("M")),
            ("F", Some("F")),
            ("MUJER", Some("F")),
            ("2", Some("F")),
            ("", None),
            ("X", None),
        ] {
            assert_eq!(normalizar_sexo(entrada).as_deref(), esperado, "{entrada}");
        }
    }

    #[test]
    fn una_direccion_larga_se_acorta_sin_perder_la_zona() {
        let calle = "A".repeat(200);
        let cuerpo = format!(
            r#"{{"success":true,"data":{{"numero":"1","nombres":"ANA","departamento":"LIMA","provincia":"LIMA","distrito":"SANTA ANITA","direccion":"{}"}}}}"#,
            calle
        );
        let d = interpretar_respuesta(TIPO_DNI, 200, &cuerpo).ok().unwrap().direccion.unwrap();
        assert!(d.chars().count() <= 120, "{}", d.chars().count());
        assert!(d.ends_with(", LIMA - LIMA - SANTA ANITA"));
    }

    #[test]
    fn sin_zona_la_direccion_va_sola() {
        let cuerpo = r#"{"success":true,"data":{"numero":"1","nombres":"ANA","direccion":"jr. lima 45"}}"#;
        assert_eq!(
            interpretar_respuesta(TIPO_DNI, 200, cuerpo).ok().unwrap().direccion.as_deref(),
            Some("JR. LIMA 45")
        );
        let cuerpo = r#"{"success":true,"data":{"numero":"1","nombres":"ANA","direccion":"-"}}"#;
        assert_eq!(interpretar_respuesta(TIPO_DNI, 200, cuerpo).ok().unwrap().direccion, None);
    }

    #[test]
    fn documento_que_no_existe() {
        assert_eq!(
            interpretar_respuesta(TIPO_DNI, 400, r#"{"status":400,"success":false,"message":"Bad Request"}"#).err(),
            Some(MotivoFallo::NoEncontrado)
        );
        assert_eq!(interpretar_respuesta(TIPO_DNI, 404, "").err(), Some(MotivoFallo::NoEncontrado));
        assert_eq!(
            interpretar_respuesta(TIPO_DNI, 200, r#"{"success":false,"message":"No se encontraron datos"}"#).err(),
            Some(MotivoFallo::NoEncontrado)
        );
        assert_eq!(
            interpretar_respuesta(TIPO_DNI, 200, r#"{"success":true,"data":null}"#).err(),
            Some(MotivoFallo::NoEncontrado)
        );
        assert_eq!(
            interpretar_respuesta(TIPO_DNI, 200, r#"{"success":true,"data":{}}"#).err(),
            Some(MotivoFallo::NoEncontrado)
        );
        // Respondió, pero sin ningún nombre: no sirve para el registro.
        assert_eq!(
            interpretar_respuesta(TIPO_DNI, 200, r#"{"success":true,"data":{"numero":"1","nombres":""}}"#).err(),
            Some(MotivoFallo::NoEncontrado)
        );
    }

    #[test]
    fn token_y_limite() {
        assert_eq!(interpretar_respuesta(TIPO_DNI, 401, "").err(), Some(MotivoFallo::TokenInvalido));
        assert_eq!(interpretar_respuesta(TIPO_DNI, 403, "").err(), Some(MotivoFallo::TokenInvalido));
        assert_eq!(interpretar_respuesta(TIPO_DNI, 429, "").err(), Some(MotivoFallo::LimiteAgotado));
        assert_eq!(
            interpretar_respuesta(TIPO_DNI, 400, r#"{"success":false,"message":"Token inválido"}"#).err(),
            Some(MotivoFallo::TokenInvalido)
        );
        assert_eq!(
            interpretar_respuesta(TIPO_DNI, 400, r#"{"success":false,"message":"Límite de consultas excedido"}"#).err(),
            Some(MotivoFallo::LimiteAgotado)
        );
    }

    #[test]
    fn errores_raros_no_rompen_nada() {
        assert!(matches!(interpretar_respuesta(TIPO_DNI, 500, "<html>boom</html>").err(), Some(MotivoFallo::Otro(m)) if m.contains("500")));
        assert!(matches!(interpretar_respuesta(TIPO_DNI, 200, "no es json").err(), Some(MotivoFallo::Otro(_))));
        assert_eq!(interpretar_respuesta("4", 200, DNI_EJEMPLO).err(), Some(MotivoFallo::NoEncontrado)); // carnet: sin consulta
    }

    #[test]
    fn el_modo_simulado_conserva_su_regla() {
        assert!(simulada(TIPO_DNI, "12345678").is_some());
        assert!(simulada(TIPO_DNI, "12345670").is_none());
        assert_eq!(
            simulada(TIPO_RUC, "20123456781").unwrap().razon_social.as_deref(),
            Some("EMPRESA DE PRUEBA S.A.C.")
        );
        assert!(simulada("4", "123").is_none());
    }
}
