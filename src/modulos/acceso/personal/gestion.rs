// gestion.rs (acceso/personal): las reglas para dar de alta, editar, restablecer la clave y eliminar al personal.
// Aquí se decide: se revisa lo que llegó, se genera la clave temporal y se llama al repositorio. Sin SQL.
// Quién puede hacerlo (solo el Administrador) lo decide rutas_gestion.rs antes de llegar aquí.
//
// Protecciones:
//   - Nadie puede eliminarse a sí mismo, quitarse su propio rol de Administrador ni restablecerse la clave
//     desde aquí (para eso existe "Cambiar mi clave"): evita quedarse fuera sin querer.
//   - El sistema nunca se queda sin administrador activo (lo comprueba el repositorio dentro de la transacción).

use sqlx::PgPool;

use super::modelo::{
    ClaveTemporal, EntradaAlta, EntradaEdicion, NuevoPersonal, ResultadoPersonal, UsuarioActual,
};
use super::roles::ROL_ADMIN;
use super::validaciones::{
    validar_datos_personales, validar_documento_personal, validar_roles, validar_usuario,
};
use super::{claves, repositorio, repositorio_gestion};

// El resultado que entiende rutas_gestion.rs: cada caso tiene su código HTTP y su mensaje.
pub enum Gestion<T> {
    Hecho(T),
    Invalido(String), // lo enviado no cumple las reglas (400)
    Repetido(String), // ya existe (409)
    NoEncontrado,     // no existe o ya fue eliminado (404)
}

// Convierte el resultado del repositorio en el de la gestión, con el mensaje para la persona.
fn traducir(resultado: ResultadoPersonal) -> Gestion<()> {
    match resultado {
        ResultadoPersonal::Hecho => Gestion::Hecho(()),
        ResultadoPersonal::DocumentoRepetido => {
            Gestion::Repetido("Ya hay una persona activa con ese documento.".to_string())
        }
        ResultadoPersonal::UsuarioRepetido => {
            Gestion::Repetido("Ese nombre de usuario ya está en uso. Elige otro.".to_string())
        }
        ResultadoPersonal::UltimoAdmin => Gestion::Invalido(
            "No se puede: el sistema debe tener siempre al menos un administrador activo.".to_string(),
        ),
        ResultadoPersonal::NoEncontrado => Gestion::NoEncontrado,
    }
}

// Pasa un Gestion<()> a Gestion<T>, poniendo "valor" si salió bien.
fn con_valor<T>(gestion: Gestion<()>, valor: T) -> Gestion<T> {
    match gestion {
        Gestion::Hecho(()) => Gestion::Hecho(valor),
        Gestion::Invalido(mensaje) => Gestion::Invalido(mensaje),
        Gestion::Repetido(mensaje) => Gestion::Repetido(mensaje),
        Gestion::NoEncontrado => Gestion::NoEncontrado,
    }
}

// Da de alta a una persona con una clave temporal, que se devuelve para entregársela una sola vez.
pub async fn crear(pool: &PgPool, entrada: EntradaAlta) -> Result<Gestion<ClaveTemporal>, sqlx::Error> {
    // 1. Revisamos todo lo enviado.
    let (tipo_documento, numero_documento) =
        match validar_documento_personal(&entrada.tipo_documento, &entrada.numero_documento) {
            Ok(documento) => documento,
            Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
        };
    let datos = match validar_datos_personales(
        &entrada.paterno,
        entrada.materno.as_deref().unwrap_or(""),
        &entrada.nombres,
        entrada.telefono.as_deref().unwrap_or(""),
        entrada.correo.as_deref().unwrap_or(""),
    ) {
        Ok(datos) => datos,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };
    let usuario = match validar_usuario(&entrada.usuario) {
        Ok(usuario) => usuario,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };
    let roles = match validar_roles(&entrada.roles) {
        Ok(roles) => roles,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };

    // 2. Clave temporal: se protege con Argon2 antes de guardarla; la original solo se devuelve una vez.
    let clave = claves::generar_clave_temporal();
    let clave_hash = match claves::hashear_async(clave.clone()).await {
        Ok(hash) => hash,
        Err(mensaje) => {
            eprintln!("{}", mensaje);
            return Ok(Gestion::Invalido("No se pudo crear la clave. Inténtalo de nuevo.".to_string()));
        }
    };

    // 3. Guardamos todo junto.
    let nuevo = NuevoPersonal {
        tipo_documento,
        numero_documento,
        paterno: datos.paterno,
        materno: datos.materno,
        nombres: datos.nombres,
        telefono: datos.telefono,
        correo: datos.correo,
        usuario: usuario.clone(),
        clave_hash,
        roles,
    };
    let resultado = repositorio::crear_personal(pool, &nuevo).await?;
    Ok(con_valor(
        traducir(resultado),
        ClaveTemporal {
            usuario,
            clave_temporal: clave,
        },
    ))
}

// Edita los datos personales y los roles de una persona (identificada por el código de su usuario).
pub async fn actualizar(
    pool: &PgPool,
    actor: &UsuarioActual,
    codigo: &str,
    entrada: EntradaEdicion,
) -> Result<Gestion<()>, sqlx::Error> {
    let datos = match validar_datos_personales(
        &entrada.paterno,
        entrada.materno.as_deref().unwrap_or(""),
        &entrada.nombres,
        entrada.telefono.as_deref().unwrap_or(""),
        entrada.correo.as_deref().unwrap_or(""),
    ) {
        Ok(datos) => datos,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };
    let roles = match validar_roles(&entrada.roles) {
        Ok(roles) => roles,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };
    if actor.codigo == codigo && !roles.iter().any(|rol| rol == ROL_ADMIN) {
        return Ok(Gestion::Invalido(
            "No puedes quitarte a ti mismo el rol de Administrador.".to_string(),
        ));
    }
    Ok(traducir(repositorio_gestion::actualizar(pool, codigo, &datos, &roles).await?))
}

// Restablece la clave de otra persona con una temporal nueva, que se devuelve una sola vez.
pub async fn restablecer_clave(
    pool: &PgPool,
    actor: &UsuarioActual,
    codigo: &str,
) -> Result<Gestion<ClaveTemporal>, sqlx::Error> {
    if actor.codigo == codigo {
        return Ok(Gestion::Invalido(
            "Para cambiar tu propia clave usa «Cambiar mi clave» en el inicio del panel.".to_string(),
        ));
    }
    let clave = claves::generar_clave_temporal();
    let clave_hash = match claves::hashear_async(clave.clone()).await {
        Ok(hash) => hash,
        Err(mensaje) => {
            eprintln!("{}", mensaje);
            return Ok(Gestion::Invalido("No se pudo crear la clave. Inténtalo de nuevo.".to_string()));
        }
    };
    match repositorio_gestion::restablecer_clave(pool, codigo, &clave_hash).await? {
        Some(usuario) => Ok(Gestion::Hecho(ClaveTemporal {
            usuario,
            clave_temporal: clave,
        })),
        None => Ok(Gestion::NoEncontrado),
    }
}

// Da de baja a una persona.
pub async fn eliminar(
    pool: &PgPool,
    actor: &UsuarioActual,
    codigo: &str,
) -> Result<Gestion<()>, sqlx::Error> {
    if actor.codigo == codigo {
        return Ok(Gestion::Invalido("No puedes eliminar tu propia cuenta.".to_string()));
    }
    Ok(traducir(repositorio_gestion::eliminar(pool, codigo).await?))
}
