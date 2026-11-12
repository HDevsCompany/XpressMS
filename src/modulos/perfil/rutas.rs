// rutas.rs (perfil): la "puerta" de «Mi perfil y datos». Solo devuelve JSON y no cambia nada:
//
//   POST /perfil/datos  { token } -> todo lo que la página de perfil muestra (datos, contacto, privacidad,
//                                    direcciones y personas autorizadas).
//
// Solo responde si el token es de una conversación con un cliente reconocido (si no, 401). Los datos delicados
// salen enmascarados (documento, celular, correo, fecha de nacimiento en la tarjeta del pase), igual que en el chat.
// Los cambios (nuevas direcciones, personas autorizadas...) los hace Juno en el chat; ver agente/flujo_perfil.rs.
// Esta ruta no escribe SQL: usa los repositorios de agente, acceso y perfil.

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::modelo::{Direccion, Persona};
use super::repositorio;
use crate::modulos::acceso::modelo::ClienteCompleto;
use crate::modulos::acceso::repositorio as acceso_repo;
use crate::modulos::agente::repositorio as agente_repo;
use crate::modulos::comun::formato;

// Máximos del perfil (los usa también Juno al guardar).
pub const MAX_DIRECCIONES: usize = 5;
pub const MAX_PERSONAS: usize = 3;

#[derive(Deserialize)]
struct PerfilEntrada {
    token: Option<String>,
}

// Una persona autorizada, lista para mostrar: el documento sale enmascarado ("DNI 4•••••89").
#[derive(Serialize)]
struct PersonaVista {
    codigo: String,
    nombre: String,
    iniciales: String,
    documento: String,
    parentesco: String,
}

// Lo que recibe la página.
#[derive(Serialize)]
struct Perfil {
    // --- pase ---
    codigo: String,        // código de cliente (lo lleva el código de barras)
    estado: String,        // "Activo"
    variante: String,      // "persona" o "empresa"
    tipo_persona: String,  // "Persona natural" o "Empresa"
    nombre: String,        // "Pedro Alonso Ramos Quispe"
    iniciales: String,     // "PQ"
    tipo_documento: String,   // "DNI", "RUC", "Carnet de extranjería" o "Pasaporte" (el pase lo muestra aparte del número)
    numero_documento: String, // "12345678" (completo: es el mismo que la persona escribió para entrar)
    documento: String,     // "DNI 1•••••78" (el que sale oculto en «Datos personales»)
    documento_completo: String, // "DNI 12345678" (el pase y el botón «Mostrar»: es el mismo que la persona escribió para entrar)
    sexo: Option<String>,
    nacimiento_enmascarado: Option<String>, // "••/••/1999"
    nacimiento: Option<String>,             // "14 de marzo de 1999" (solo en la tarjeta de datos personales)
    direccion_enmascarada: String,          // "•••••• - Trujillo" (el pase)
    direccion: String,                      // "Urb. Los Jardines Mz. B Lt. 4 - Trujillo" (la tarjeta de contacto)
    celular: String,                        // "+51 9•• ••• 321"
    celular_verificado: bool,
    correo: Option<String>,                 // "p•••@gmail.com"
    // --- privacidad ---
    terminos_aceptados: Option<String>,     // "30/10/2026"
    // --- tarjetas ---
    direcciones: Vec<Direccion>,
    personas: Vec<PersonaVista>,
    max_direcciones: usize,
    max_personas: usize,
}

pub fn rutas() -> Router<PgPool> {
    Router::new().route("/perfil/datos", post(datos))
}

fn error_interno(error: sqlx::Error) -> StatusCode {
    eprintln!("Error de base de datos: {}", error);
    StatusCode::INTERNAL_SERVER_ERROR
}

// POST /perfil/datos
async fn datos(
    State(pool): State<PgPool>,
    Json(entrada): Json<PerfilEntrada>,
) -> Result<Json<Perfil>, StatusCode> {
    // 1. ¿Quién es? Solo un cliente reconocido puede ver su perfil.
    let token = entrada.token.ok_or(StatusCode::UNAUTHORIZED)?;
    let conversacion = agente_repo::buscar_por_token(&pool, &token)
        .await
        .map_err(error_interno)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let codigo = conversacion.cliente.ok_or(StatusCode::UNAUTHORIZED)?;

    // 2. Sus datos. Si el cliente ya no existe, es como no estar reconocido.
    let cliente = acceso_repo::obtener_cliente(&pool, &codigo)
        .await
        .map_err(error_interno)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let cuenta = repositorio::datos_de_cuenta(&pool, &codigo)
        .await
        .map_err(error_interno)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 3. Sus direcciones (la primera vez se copia la del registro como «Casa» principal) y sus personas autorizadas.
    repositorio::asegurar_direccion_inicial(&pool, &codigo).await.map_err(error_interno)?;
    let direcciones = repositorio::listar_direcciones(&pool, &codigo).await.map_err(error_interno)?;
    let personas = repositorio::listar_personas(&pool, &codigo).await.map_err(error_interno)?;

    Ok(Json(armar_perfil(&cliente, cuenta.codigo, &cuenta.estado, cuenta.celular_verificado, cuenta.consentimiento.as_deref(), direcciones, personas)))
}

// Convierte los datos guardados en lo que ve la página (todo ya enmascarado y presentable). Es una función pura.
fn armar_perfil(
    cliente: &ClienteCompleto,
    codigo: String,
    estado: &str,
    celular_verificado: bool,
    consentimiento: Option<&str>,
    direcciones: Vec<Direccion>,
    personas: Vec<Persona>,
) -> Perfil {
    let es_empresa = cliente.tipo_documento == "6" || cliente.nombres.as_deref().unwrap_or("").is_empty();

    let nombre = if es_empresa {
        formato::a_titulo(cliente.razon_social.as_deref().unwrap_or(""))
    } else {
        [cliente.nombres.as_deref(), cliente.paterno.as_deref(), cliente.materno.as_deref()]
            .iter()
            .flatten()
            .map(|parte| formato::a_titulo(parte))
            .filter(|parte| !parte.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    };

    let nacimiento = cliente.fecha_nacimiento.as_deref().filter(|f| !f.is_empty());

    Perfil {
        codigo,
        estado: if estado == "A" { "Activo".to_string() } else { "Inactivo".to_string() },
        variante: if es_empresa { "empresa" } else { "persona" }.to_string(),
        tipo_persona: if es_empresa { "Empresa" } else { "Persona natural" }.to_string(),
        iniciales: formato::iniciales(&nombre),
        nombre,
        tipo_documento: formato::nombre_tipo_documento(&cliente.tipo_documento).to_string(),
        numero_documento: cliente.numero_documento.clone(),
        documento: format!(
            "{} {}",
            formato::nombre_tipo_documento(&cliente.tipo_documento),
            formato::documento_enmascarado(&cliente.numero_documento)
        ),
        documento_completo: format!(
            "{} {}",
            formato::nombre_tipo_documento(&cliente.tipo_documento),
            cliente.numero_documento
        ),
        sexo: cliente.sexo.as_deref().filter(|s| !s.is_empty()).map(|s| formato::nombre_sexo(s).to_string()),
        nacimiento_enmascarado: nacimiento.map(formato::fecha_enmascarada),
        nacimiento: nacimiento.map(formato::fecha_larga),
        direccion_enmascarada: formato::direccion_enmascarada(&cliente.direccion),
        direccion: formato::a_titulo(&cliente.direccion),
        celular: formato::telefono_enmascarado(&cliente.telefono),
        celular_verificado,
        correo: cliente.correo.as_deref().filter(|c| !c.is_empty()).map(formato::correo_enmascarado),
        terminos_aceptados: consentimiento.map(formato::formatear_fecha),
        direcciones,
        personas: personas.iter().map(vista_de_persona).collect(),
        max_direcciones: MAX_DIRECCIONES,
        max_personas: MAX_PERSONAS,
    }
}

// Una persona autorizada tal como la ve la página: con el documento enmascarado.
fn vista_de_persona(persona: &Persona) -> PersonaVista {
    PersonaVista {
        codigo: persona.codigo.clone(),
        iniciales: formato::iniciales(&formato::a_titulo(&persona.nombre)),
        nombre: formato::a_titulo(&persona.nombre),
        documento: format!(
            "{} {}",
            formato::nombre_tipo_documento(&persona.tipo_documento),
            formato::documento_enmascarado(&persona.numero_documento)
        ),
        parentesco: persona.parentesco.clone(),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn cliente_persona() -> ClienteCompleto {
        ClienteCompleto {
            tipo_documento: "1".to_string(),
            numero_documento: "12345678".to_string(),
            paterno: Some("RAMOS".to_string()),
            materno: Some("QUISPE".to_string()),
            nombres: Some("PEDRO ALONSO".to_string()),
            razon_social: None,
            direccion: "URB. LOS JARDINES MZ. B LT. 4 - TRUJILLO".to_string(),
            sexo: Some("M".to_string()),
            fecha_nacimiento: Some("1999-03-14".to_string()),
            telefono: "987654321".to_string(),
            correo: Some("pedro@gmail.com".to_string()),
        }
    }

    #[test]
    fn el_perfil_sale_enmascarado_y_presentable() {
        let perfil = armar_perfil(&cliente_persona(), "K7TQ3M9XA".to_string(), "A", true, Some("2026-10-30"), vec![], vec![]);
        assert_eq!(perfil.nombre, "Pedro Alonso Ramos Quispe");
        assert_eq!(perfil.iniciales, "PQ");
        assert_eq!(perfil.documento, "DNI 1•••••78");
        assert_eq!(perfil.documento_completo, "DNI 12345678");
        assert_eq!(perfil.tipo_documento, "DNI");
        assert_eq!(perfil.numero_documento, "12345678");
        assert_eq!(perfil.direccion, "Urb. Los Jardines Mz. B Lt. 4 - Trujillo");
        assert_eq!(perfil.tipo_persona, "Persona natural");
        assert_eq!(perfil.nacimiento.as_deref(), Some("14 de marzo de 1999"));
        assert_eq!(perfil.nacimiento_enmascarado.as_deref(), Some("••/••/1999"));
        assert_eq!(perfil.direccion_enmascarada, "•••••• - TRUJILLO");
        assert_eq!(perfil.celular, "+51 9•• ••• 321");
        assert_eq!(perfil.correo.as_deref(), Some("p•••@gmail.com"));
        assert_eq!(perfil.terminos_aceptados.as_deref(), Some("30/10/2026"));
        assert_eq!(perfil.estado, "Activo");
    }

    #[test]
    fn una_empresa_usa_su_razon_social() {
        let mut empresa = cliente_persona();
        empresa.tipo_documento = "6".to_string();
        empresa.numero_documento = "20123456789".to_string();
        empresa.nombres = None;
        empresa.razon_social = Some("COMERCIAL ABC SAC".to_string());
        let perfil = armar_perfil(&empresa, "C".to_string(), "A", false, None, vec![], vec![]);
        assert_eq!(perfil.variante, "empresa");
        assert_eq!(perfil.nombre, "Comercial Abc Sac");
        assert_eq!(perfil.documento, "RUC 2••••••••89");
        assert_eq!(perfil.tipo_documento, "RUC");
        assert_eq!(perfil.numero_documento, "20123456789");
        assert!(perfil.terminos_aceptados.is_none());
    }

    #[test]
    fn la_persona_autorizada_sale_con_el_documento_oculto() {
        let persona = Persona {
            codigo: "P1".to_string(),
            tipo_documento: "1".to_string(),
            numero_documento: "40123489".to_string(),
            nombre: "ROSA ELENA QUISPE VARGAS".to_string(),
            parentesco: "Madre".to_string(),
        };
        let vista = vista_de_persona(&persona);
        assert_eq!(vista.documento, "DNI 4•••••89");
        assert_eq!(vista.nombre, "Rosa Elena Quispe Vargas");
        assert_eq!(vista.iniciales, "RV");
    }
}
