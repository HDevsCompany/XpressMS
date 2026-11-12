// autenticacion.rs: las reglas de entrar y de cambiar la clave. Aquí se decide; el SQL está en repositorio.rs.
//
// Reglas:
//   - Mensaje único para usuario o contraseña incorrectos: no se revela si el usuario existe.
//   - Si el usuario no existe igual se hace un cálculo Argon2, para que tarde lo mismo y no delate nada.
//   - 5 intentos fallidos seguidos bloquean la cuenta 15 minutos.
//   - Al ingresar se abre una sesión de 8 horas; el token se entrega una vez y en la base solo queda su huella.

use sqlx::PgPool;

use super::cookies::HORAS_SESION;
use super::modelo::UsuarioActual;
use super::{claves, repositorio};
use crate::modulos::comun::codigos::generar_token;
use crate::modulos::comun::validaciones::{clave_valida, normalizar_usuario};

pub const MAX_INTENTOS: i32 = 5;
pub const MINUTOS_BLOQUEO: i32 = 15;

// Tope de largo de lo que se acepta como clave al ingresar (evita enviar textos enormes para gastar el servidor).
const LARGO_MAXIMO_CLAVE: usize = 200;

pub enum ResultadoEntrada {
    // Ingresó: "token" es el valor para la cookie; "debe_cambiar_clave" indica clave temporal.
    Correcto { token: String, debe_cambiar_clave: bool },
    Incorrecto,
    Bloqueado,
}

pub enum ResultadoCambio {
    Hecho,
    ClaveActualIncorrecta,
    ClaveNuevaInvalida(String), // el mensaje para mostrar
}

// Valida usuario y clave. Si son correctos, abre la sesión.
pub async fn entrar(
    pool: &PgPool,
    usuario: &str,
    clave: &str,
) -> Result<ResultadoEntrada, sqlx::Error> {
    // 1. Buscamos la cuenta (si el usuario ni siquiera tiene forma válida, ni consultamos).
    let credencial = match normalizar_usuario(usuario) {
        Some(nombre) => repositorio::buscar_credencial(pool, &nombre).await?,
        None => None,
    };
    let Some(credencial) = credencial else {
        let _ = claves::hashear_async(clave.to_string()).await; // gasta el mismo tiempo que una verificación
        return Ok(ResultadoEntrada::Incorrecto);
    };

    // 2. Cuenta bloqueada: ni se prueba la clave.
    if credencial.bloqueado {
        return Ok(ResultadoEntrada::Bloqueado);
    }

    // 3. Comprobamos la clave.
    let coincide = clave.chars().count() <= LARGO_MAXIMO_CLAVE
        && claves::verificar_async(clave.to_string(), credencial.clave_hash.clone()).await;
    if !coincide {
        let bloqueada =
            repositorio::registrar_fallo(pool, &credencial.codigo, MAX_INTENTOS, MINUTOS_BLOQUEO)
                .await?;
        return Ok(if bloqueada {
            ResultadoEntrada::Bloqueado
        } else {
            ResultadoEntrada::Incorrecto
        });
    }

    // 4. Correcto: limpiamos los fallos y abrimos la sesión.
    repositorio::registrar_acceso(pool, &credencial.codigo).await?;
    repositorio::limpiar_sesiones(pool).await?;
    let token = generar_token();
    repositorio::crear_sesion(
        pool,
        &credencial.codigo,
        &claves::huella_token(&token),
        HORAS_SESION as i32,
    )
    .await?;

    Ok(ResultadoEntrada::Correcto {
        token,
        debe_cambiar_clave: credencial.debe_cambiar_clave,
    })
}

// Cambia la clave de quien está en sesión. Pide la clave actual (por si alguien dejó la sesión abierta).
// Al terminar se cierran las demás sesiones de esa cuenta; la actual sigue abierta.
pub async fn cambiar_clave(
    pool: &PgPool,
    usuario: &UsuarioActual,
    huella_sesion: &str,
    actual: &str,
    nueva: &str,
) -> Result<ResultadoCambio, sqlx::Error> {
    // 1. La clave nueva debe cumplir las reglas y ser distinta de la actual.
    if let Err(mensaje) = clave_valida(nueva) {
        return Ok(ResultadoCambio::ClaveNuevaInvalida(mensaje));
    }
    if nueva == actual {
        return Ok(ResultadoCambio::ClaveNuevaInvalida(
            "La clave nueva debe ser distinta de la actual.".to_string(),
        ));
    }

    // 2. Comprobamos la clave actual. Un error cuenta como intento fallido (protege una sesión abandonada).
    let guardada = repositorio::obtener_clave_hash(pool, &usuario.codigo).await?;
    let coincide = match guardada {
        Some(hash) if actual.chars().count() <= LARGO_MAXIMO_CLAVE => {
            claves::verificar_async(actual.to_string(), hash).await
        }
        _ => false,
    };
    if !coincide {
        repositorio::registrar_fallo(pool, &usuario.codigo, MAX_INTENTOS, MINUTOS_BLOQUEO).await?;
        return Ok(ResultadoCambio::ClaveActualIncorrecta);
    }

    // 3. Guardamos la clave nueva y cerramos las otras sesiones.
    let hash = match claves::hashear_async(nueva.to_string()).await {
        Ok(hash) => hash,
        Err(mensaje) => {
            eprintln!("{}", mensaje);
            return Ok(ResultadoCambio::ClaveNuevaInvalida(
                "No se pudo guardar la clave. Inténtalo de nuevo.".to_string(),
            ));
        }
    };
    repositorio::guardar_clave(pool, &usuario.codigo, &hash).await?;
    repositorio::cerrar_otras_sesiones(pool, &usuario.codigo, huella_sesion).await?;
    Ok(ResultadoCambio::Hecho)
}
