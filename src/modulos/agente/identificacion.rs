// identificacion.rs: reconocer a un cliente desde la página de ingreso (fuera del chat).
// Es la misma verificación que hace el agente con «Ya soy cliente» (documento + últimos 3 dígitos del celular),
// pero con tres diferencias de seguridad:
//   - los intentos fallidos se limitan por documento y por dirección de internet (ver limite_intentos.rs);
//   - si algo no coincide, la respuesta es siempre la misma, así no se revela si un documento está registrado;
//   - nunca se muestran datos del cliente.
// Regla de este archivo: no escribe SQL; usa los repositorios de acceso y del agente.

use std::time::Duration;

use sqlx::PgPool;

use super::limite_intentos as limite;
use super::modelo::Datos;
use super::pasos::LIBRE;
use super::repositorio;
use crate::modulos::acceso::repositorio as acceso_repo;
use crate::modulos::comun::validaciones;

// Cuántos fallos se permiten en la ventana de tiempo antes de bloquear.
const MAX_FALLOS_DOCUMENTO: usize = 5; // por documento (protege a la persona a la que pertenece)
const MAX_FALLOS_RED: usize = 20; // por dirección de internet (frena a quien prueba muchos documentos)
const VENTANA: Duration = Duration::from_secs(15 * 60);

// Tipos de documento que existen (códigos SUNAT de Comun.TiposDocumento).
const TIPOS: [&str; 4] = ["1", "4", "6", "7"];

#[derive(Debug, PartialEq)]
pub enum Resultado {
    Reconocido(String), // código del cliente
    NoVerificado,       // el documento no existe o los dígitos no coinciden (no se distingue)
    Bloqueado,          // demasiados intentos: que espere unos minutos
    Invalido(String),   // dato mal escrito: el texto explica qué corregir
}

// Revisa el documento y los últimos 3 dígitos del celular.
// "red" identifica a quien intenta (su dirección de internet), para el límite por red.
pub async fn identificar(
    pool: &PgPool,
    tipo: &str,
    numero: &str,
    ultimos: &str,
    red: &str,
) -> Result<Resultado, sqlx::Error> {
    if !TIPOS.contains(&tipo) {
        return Ok(Resultado::Invalido("Elige el tipo de documento.".to_string()));
    }
    let numero = match validaciones::validar_documento(tipo, numero) {
        Ok(numero) => numero,
        Err(mensaje) => return Ok(Resultado::Invalido(mensaje)),
    };
    let ultimos: String = ultimos.chars().filter(|c| c.is_ascii_digit()).collect();
    if ultimos.len() != 3 {
        return Ok(Resultado::Invalido(
            "Escribe los últimos 3 dígitos de tu celular.".to_string(),
        ));
    }

    let clave_documento = format!("doc:{tipo}:{numero}");
    let clave_red = format!("red:{red}");
    if limite::bloqueado(&clave_documento, MAX_FALLOS_DOCUMENTO, VENTANA)
        || limite::bloqueado(&clave_red, MAX_FALLOS_RED, VENTANA)
    {
        return Ok(Resultado::Bloqueado);
    }

    let mut reconocido = None;
    if let Some(cliente) = acceso_repo::buscar_cliente_por_documento(pool, tipo, &numero).await? {
        let telefono = acceso_repo::obtener_telefono(pool, &cliente.codigo)
            .await?
            .unwrap_or_default();
        if !telefono.is_empty() && telefono.ends_with(&ultimos) {
            reconocido = Some(cliente.codigo);
        }
    }

    match reconocido {
        Some(codigo) => {
            limite::limpiar(&clave_documento);
            Ok(Resultado::Reconocido(codigo))
        }
        None => {
            limite::anotar_fallo(&clave_documento);
            limite::anotar_fallo(&clave_red);
            Ok(Resultado::NoVerificado)
        }
    }
}

// Deja al cliente reconocido dentro del chat: usa su conversación (si el navegador trajo un token válido)
// o crea una nueva y la vincula con el cliente. Devuelve el token de la conversación, que el navegador guarda
// para que, al abrir el lienzo, el agente ya lo conozca.
pub async fn entrar_al_chat(
    pool: &PgPool,
    token: Option<&str>,
    cliente: &str,
) -> Result<String, sqlx::Error> {
    let existente = match token {
        Some(token) => repositorio::buscar_por_token(pool, token).await?,
        None => None,
    };
    let conversacion = match existente {
        Some(conversacion) => conversacion,
        None => repositorio::crear_conversacion(pool).await?,
    };

    repositorio::vincular_cliente(pool, &conversacion.codigo, cliente, None).await?;
    // Si la conversación estaba a medias (por ejemplo, en pleno registro), empieza limpia.
    repositorio::guardar_estado(pool, &conversacion.codigo, LIBRE, &Datos::default()).await?;
    // El saludo («Hola, Pedro») lo muestra el lienzo; la conversación empieza vacía.

    Ok(conversacion.token)
}
