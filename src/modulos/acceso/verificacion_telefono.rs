// verificacion_telefono.rs: la verificación del celular con un código de 6 dígitos por SMS.
// Une las piezas: codigo_otp.rs (generar y comparar, sin base de datos), repositorio_verificacion.rs (la base)
// y comun/sms.rs (el envío por Textbee). El chat (agente/flujo.rs) solo llama a estas dos funciones:
//   - solicitar_codigo: crea el código, lo guarda (solo su huella) y lo envía por SMS.
//   - verificar_codigo: compara lo que escribió la persona con el código guardado.
//
// Reglas de seguridad:
//   - Máximo 3 códigos pedidos cada 10 minutos por celular y, aparte, por red (dirección de internet).
//   - Cada código vale 10 minutos y admite 5 intentos; después hay que pedir uno nuevo.
//   - Un código solo sirve una vez y al pedir uno nuevo los anteriores dejan de servir.
//   - Si el SMS seguro no salió (sin conexión, clave inválida...), el pedido no se cuenta contra el límite.
//   - Si Textbee tarda en contestar (respuesta diferida), el código se conserva porque el SMS puede llegar igual.

use sqlx::PgPool;

use super::codigo_otp::{generar_codigo_otp, generar_sal, huella, huellas_iguales, mensaje_sms, VIGENCIA_MINUTOS};
use super::repositorio_verificacion as repo;
use crate::modulos::comun::codigos::generar_codigo;
use crate::modulos::comun::{sms, validaciones};

// Máximo de códigos que se pueden pedir por celular (y por red) dentro de la ventana.
pub const MAX_SOLICITUDES: i64 = 3;
// Tamaño de la ventana, en minutos.
pub const VENTANA_MINUTOS: i32 = 10;
// Intentos permitidos por cada código.
pub const MAX_INTENTOS: i16 = 5;

// Qué pasó al pedir un código.
#[derive(Debug, PartialEq, Eq)]
pub enum Solicitud {
    Enviado,
    LimiteExcedido,  // ya se pidieron 3 códigos en 10 minutos (desde ese celular o desde esa red)
    NoSePudoEnviar,  // el SMS seguro no salió (sin internet, clave inválida, sin teléfono pasarela...)
    Demorado,        // Textbee no contestó a tiempo: el SMS puede llegar igual, el código sigue vigente
    NumeroInvalido,  // no es un celular peruano válido
}

// Qué pasó al comparar el código escrito.
#[derive(Debug, PartialEq, Eq)]
pub enum Resultado {
    Verificado,
    Incorrecto { restantes: i16 }, // no coincide; le quedan estos intentos
    Vencido,                       // pasaron los 10 minutos
    SinCodigo,                     // no hay un código pendiente para ese celular
    DemasiadosIntentos,            // agotó los 5 intentos de este código
}

// Crea un código para este celular (9 dígitos, como se guarda en el registro), lo guarda y lo envía.
pub async fn solicitar_codigo(
    pool: &PgPool,
    conversacion: &str,
    telefono: &str,
    red: &str,
) -> Result<Solicitud, sqlx::Error> {
    let Some(telefono_e164) = validaciones::a_e164(telefono) else {
        return Ok(Solicitud::NumeroInvalido);
    };

    let codigo = generar_codigo_otp();
    let sal = generar_sal();
    let hash = huella(&sal, &codigo);
    let id = generar_codigo();

    // Guarda el código solo si no se pasó el límite (el conteo y el guardado van en una sola instrucción).
    let guardado = repo::crear_pendiente(
        pool,
        &id,
        conversacion,
        &telefono_e164,
        &sal,
        &hash,
        red,
        VIGENCIA_MINUTOS,
        VENTANA_MINUTOS,
        MAX_SOLICITUDES,
    )
    .await?;
    if !guardado {
        return Ok(Solicitud::LimiteExcedido);
    }

    match sms::enviar(&telefono_e164, &mensaje_sms(&codigo)).await {
        Ok(()) => {
            repo::expirar_anteriores(pool, conversacion, &telefono_e164, &id).await?;
            Ok(Solicitud::Enviado)
        }
        // Respuesta diferida: la petición salió pero Textbee tardó. El SMS puede estar en camino, así que el código
        // se conserva (y los anteriores dejan de servir) para no dejar a la persona con un código que llega "huérfano".
        Err(error @ sms::ErrorSms::SinRespuesta(_)) => {
            eprintln!("[SMS] Textbee no confirmó a tiempo el código de verificación: {}", error);
            repo::expirar_anteriores(pool, conversacion, &telefono_e164, &id).await?;
            Ok(Solicitud::Demorado)
        }
        // Fallo seguro: el SMS no salió. Se borra el código y el pedido no cuenta contra el límite.
        // Queda en el registro del servidor (nunca con la clave ni el código) para poder revisar qué falló.
        Err(error) => {
            eprintln!("[SMS] No se pudo enviar el código de verificación: {}", error);
            repo::anular(pool, &id).await?;
            Ok(Solicitud::NoSePudoEnviar)
        }
    }
}

// Compara el código que escribió la persona (ya reducido a 6 dígitos) con el último pendiente de este celular.
pub async fn verificar_codigo(
    pool: &PgPool,
    conversacion: &str,
    telefono: &str,
    codigo_escrito: &str,
) -> Result<Resultado, sqlx::Error> {
    let Some(telefono_e164) = validaciones::a_e164(telefono) else {
        return Ok(Resultado::SinCodigo);
    };

    let Some(pendiente) = repo::buscar_pendiente(pool, conversacion, &telefono_e164).await? else {
        return Ok(Resultado::SinCodigo);
    };

    if !pendiente.vigente {
        repo::marcar_expirado(pool, &pendiente.codigo).await?;
        return Ok(Resultado::Vencido);
    }
    if pendiente.intentos >= MAX_INTENTOS {
        repo::marcar_expirado(pool, &pendiente.codigo).await?;
        return Ok(Resultado::DemasiadosIntentos);
    }

    // El intento se cuenta antes de comparar.
    let Some(intentos) = repo::sumar_intento(pool, &pendiente.codigo).await? else {
        return Ok(Resultado::SinCodigo); // en ese instante otro mensaje ya lo había usado
    };

    if huellas_iguales(&huella(&pendiente.sal, codigo_escrito), &pendiente.hash) {
        // Si justo venció o ya se usó, la condición del SQL lo detecta.
        return Ok(if repo::marcar_verificado(pool, &pendiente.codigo).await? {
            Resultado::Verificado
        } else {
            Resultado::Vencido
        });
    }

    let restantes = MAX_INTENTOS - intentos;
    if restantes <= 0 {
        repo::marcar_expirado(pool, &pendiente.codigo).await?;
        return Ok(Resultado::DemasiadosIntentos);
    }
    Ok(Resultado::Incorrecto { restantes })
}
