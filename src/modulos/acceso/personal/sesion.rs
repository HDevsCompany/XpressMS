// sesion.rs: reconoce QUIÉN hace cada petición, leyendo la cookie de sesión.
//
// Dos formas de usarlo en una ruta:
//   - Ruta de datos (JSON): recibir "PersonalAutenticado(usuario)" como parámetro. Si no hay sesión
//     válida, axum responde 401 solo y la función ni se ejecuta.
//   - Página (HTML): llamar a usuario_actual(...) y, si devuelve None, redirigir a /ingresar.
// Para limitar por rol, la ruta llama después a exigir_rol(&usuario, &[ROL_ADMIN, ...]).

use axum::extract::FromRequestParts;
use axum::http::{header, request::Parts, HeaderMap, StatusCode};
use sqlx::PgPool;

use super::modelo::UsuarioActual;
use super::roles::tiene_algun_rol;
use super::{claves, cookies, repositorio};
use crate::modulos::comun::formato::a_titulo;

// El token de la cookie de sesión, si el navegador la envió.
pub fn token_de(cabeceras: &HeaderMap) -> Option<String> {
    cabeceras
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|valor| valor.to_str().ok())
        .find_map(|encabezado| cookies::leer(encabezado, cookies::NOMBRE))
}

// Quién está en sesión: None si no hay cookie, la sesión venció, se cerró o la cuenta se desactivó.
pub async fn usuario_actual(
    pool: &PgPool,
    cabeceras: &HeaderMap,
) -> Result<Option<UsuarioActual>, sqlx::Error> {
    let Some(token) = token_de(cabeceras) else {
        return Ok(None);
    };
    let Some(fila) = repositorio::buscar_sesion(pool, &claves::huella_token(&token)).await? else {
        return Ok(None);
    };
    let roles = repositorio::listar_roles(pool, &fila.codigo).await?;

    // Para saludar: primer nombre y apellido paterno, con formato "Pedro Guevara".
    let primer_nombre = fila.nombres.split_whitespace().next().unwrap_or("");
    let nombre = format!("{} {}", a_titulo(primer_nombre), a_titulo(&fila.paterno));

    Ok(Some(UsuarioActual {
        codigo: fila.codigo,
        usuario: fila.usuario,
        nombre,
        roles,
        debe_cambiar_clave: fila.debe_cambiar_clave,
    }))
}

// Comprueba que la persona tenga alguno de los roles permitidos y que ya no use clave temporal.
// 403 = "prohibido": está identificada, pero no puede hacer esto.
pub fn exigir_rol(usuario: &UsuarioActual, permitidos: &[&str]) -> Result<(), StatusCode> {
    if usuario.debe_cambiar_clave || !tiene_algun_rol(&usuario.roles, permitidos) {
        Err(StatusCode::FORBIDDEN)
    } else {
        Ok(())
    }
}

// Parámetro de ruta que exige sesión válida. Dentro viene la persona ya identificada.
pub struct PersonalAutenticado(pub UsuarioActual);

impl FromRequestParts<PgPool> for PersonalAutenticado {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, pool: &PgPool) -> Result<Self, Self::Rejection> {
        match usuario_actual(pool, &parts.headers).await {
            Ok(Some(usuario)) => Ok(PersonalAutenticado(usuario)),
            Ok(None) => Err(StatusCode::UNAUTHORIZED), // 401 = "no identificado"
            Err(error) => {
                eprintln!("Error de base de datos: {}", error);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}
