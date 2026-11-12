// repositorio.rs (acceso/personal): todo el SQL de empleados, usuarios, roles y sesiones.
// Ningún otro archivo escribe SQL sobre estas tablas. Aquí no se decide nada (eso lo hace
// autenticacion.rs): solo se guarda y se consulta.

use sqlx::error::ErrorKind;
use sqlx::PgPool;

use super::modelo::{FilaCredencial, FilaSesion, NuevoPersonal, ResultadoPersonal};
use crate::modulos::comun::codigos::generar_codigo;

// ---------------------------------------------------------------------------------------------
// Ingreso
// ---------------------------------------------------------------------------------------------

// Busca la cuenta por nombre de usuario (ya en minúsculas). Solo cuentas activas de empleados activos.
pub async fn buscar_credencial(
    pool: &PgPool,
    usuario: &str,
) -> Result<Option<FilaCredencial>, sqlx::Error> {
    sqlx::query_as::<_, FilaCredencial>(
        "select u.UsuariosCodigo as codigo, u.UsuariosClaveHash as clave_hash, \
                u.UsuariosDebeCambiarClave as debe_cambiar_clave, \
                (u.UsuariosBloqueadoHasta is not null and u.UsuariosBloqueadoHasta > now()) as bloqueado \
         from Acceso.Usuarios u \
         join Acceso.Empleados e on e.EmpleadosCodigo = u.UsuariosEmpleado \
         where u.UsuariosNombreUsuario = $1 and u.UsuariosEstado = 'A' and e.EmpleadosEstado = 'A'",
    )
    .bind(usuario)
    .fetch_optional(pool)
    .await
}

// Suma un intento fallido. Si llega al máximo, bloquea la cuenta unos minutos y reinicia el conteo.
// Devuelve true si la cuenta quedó bloqueada con este fallo.
pub async fn registrar_fallo(
    pool: &PgPool,
    usuario: &str,
    maximo: i32,
    minutos: i32,
) -> Result<bool, sqlx::Error> {
    // En un UPDATE, las dos columnas se calculan con el valor ANTERIOR, por eso la condición se repite.
    sqlx::query_scalar::<_, bool>(
        "update Acceso.Usuarios set \
            UsuariosBloqueadoHasta = case when UsuariosIntentosFallidos + 1 >= $2::int \
                then now() + make_interval(mins => $3::int) else UsuariosBloqueadoHasta end, \
            UsuariosIntentosFallidos = case when UsuariosIntentosFallidos + 1 >= $2::int \
                then 0 else UsuariosIntentosFallidos + 1 end \
         where UsuariosCodigo = $1 \
         returning (UsuariosBloqueadoHasta is not null and UsuariosBloqueadoHasta > now())",
    )
    .bind(usuario)
    .bind(maximo)
    .bind(minutos)
    .fetch_one(pool)
    .await
}

// Ingreso correcto: limpia los intentos fallidos y anota la fecha del último acceso.
pub async fn registrar_acceso(pool: &PgPool, usuario: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "update Acceso.Usuarios set UsuariosIntentosFallidos = 0, UsuariosBloqueadoHasta = null, \
                UsuariosUltimoAcceso = now() \
         where UsuariosCodigo = $1",
    )
    .bind(usuario)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Sesiones
// ---------------------------------------------------------------------------------------------

// Abre una sesión. Se guarda la huella del token, nunca el token.
pub async fn crear_sesion(
    pool: &PgPool,
    usuario: &str,
    huella: &str,
    horas: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "insert into Acceso.Sesiones (SesionesCodigo, SesionesUsuario, SesionesTokenHash, SesionesFechaExpiracion) \
         values ($1, $2, $3, now() + make_interval(hours => $4::int))",
    )
    .bind(generar_codigo())
    .bind(usuario)
    .bind(huella)
    .bind(horas)
    .execute(pool)
    .await?;
    Ok(())
}

// Borra las sesiones que vencieron o se cerraron hace más de un día (limpieza sin riesgo).
pub async fn limpiar_sesiones(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "delete from Acceso.Sesiones \
         where SesionesFechaExpiracion < now() - interval '1 day'",
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Reconoce una sesión por la huella de su token: debe estar abierta, sin vencer, y con usuario y empleado activos.
pub async fn buscar_sesion(
    pool: &PgPool,
    huella: &str,
) -> Result<Option<FilaSesion>, sqlx::Error> {
    sqlx::query_as::<_, FilaSesion>(
        "select u.UsuariosCodigo as codigo, u.UsuariosNombreUsuario as usuario, \
                e.EmpleadosNombres as nombres, e.EmpleadosPaterno as paterno, \
                u.UsuariosDebeCambiarClave as debe_cambiar_clave \
         from Acceso.Sesiones s \
         join Acceso.Usuarios u on u.UsuariosCodigo = s.SesionesUsuario \
         join Acceso.Empleados e on e.EmpleadosCodigo = u.UsuariosEmpleado \
         where s.SesionesTokenHash = $1 and s.SesionesEstado = 'A' and s.SesionesFechaExpiracion > now() \
           and u.UsuariosEstado = 'A' and e.EmpleadosEstado = 'A'",
    )
    .bind(huella)
    .fetch_optional(pool)
    .await
}

// Cierra una sesión (cerrar sesión).
pub async fn cerrar_sesion(pool: &PgPool, huella: &str) -> Result<(), sqlx::Error> {
    sqlx::query("update Acceso.Sesiones set SesionesEstado = 'E' where SesionesTokenHash = $1")
        .bind(huella)
        .execute(pool)
        .await?;
    Ok(())
}

// Cierra todas las sesiones de un usuario menos la actual (al cambiar la clave).
pub async fn cerrar_otras_sesiones(
    pool: &PgPool,
    usuario: &str,
    huella_actual: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "update Acceso.Sesiones set SesionesEstado = 'E' \
         where SesionesUsuario = $1 and SesionesTokenHash <> $2 and SesionesEstado = 'A'",
    )
    .bind(usuario)
    .bind(huella_actual)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Roles y claves
// ---------------------------------------------------------------------------------------------

// Los códigos de los roles activos de un usuario, en orden ("ADM", "TEC").
pub async fn listar_roles(pool: &PgPool, usuario: &str) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "select r.RolesCodigo \
         from Acceso.UsuariosRoles ur \
         join Acceso.Roles r on r.RolesCodigo = ur.UsuariosRolesRol \
         where ur.UsuariosRolesUsuario = $1 and r.RolesEstado = 'A' \
         order by r.RolesCodigo",
    )
    .bind(usuario)
    .fetch_all(pool)
    .await
}

// El resultado Argon2 guardado de un usuario (para comprobar su clave actual).
pub async fn obtener_clave_hash(
    pool: &PgPool,
    usuario: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "select UsuariosClaveHash from Acceso.Usuarios where UsuariosCodigo = $1",
    )
    .bind(usuario)
    .fetch_optional(pool)
    .await
}

// Guarda una clave que la persona eligió ella misma: ya no es temporal.
pub async fn guardar_clave(
    pool: &PgPool,
    usuario: &str,
    clave_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "update Acceso.Usuarios set UsuariosClaveHash = $2, UsuariosDebeCambiarClave = false \
         where UsuariosCodigo = $1",
    )
    .bind(usuario)
    .bind(clave_hash)
    .execute(pool)
    .await?;
    Ok(())
}

// Restablece la clave de una cuenta con una temporal: obliga a cambiarla, quita bloqueos y cierra
// todas sus sesiones. Devuelve false si el usuario no existe.
pub async fn restablecer_clave(
    pool: &PgPool,
    usuario: &str,
    clave_hash: &str,
) -> Result<bool, sqlx::Error> {
    let mut transaccion = pool.begin().await?;
    let codigo = sqlx::query_scalar::<_, String>(
        "update Acceso.Usuarios set UsuariosClaveHash = $2, UsuariosDebeCambiarClave = true, \
                UsuariosIntentosFallidos = 0, UsuariosBloqueadoHasta = null \
         where UsuariosNombreUsuario = $1 \
         returning UsuariosCodigo",
    )
    .bind(usuario)
    .bind(clave_hash)
    .fetch_optional(&mut *transaccion)
    .await?;
    let Some(codigo) = codigo else {
        return Ok(false);
    };
    sqlx::query("update Acceso.Sesiones set SesionesEstado = 'E' where SesionesUsuario = $1")
        .bind(&codigo)
        .execute(&mut *transaccion)
        .await?;
    transaccion.commit().await?;
    Ok(true)
}

// ---------------------------------------------------------------------------------------------
// Alta de personal
// ---------------------------------------------------------------------------------------------

// ¿Ya existe algún administrador activo? (crear-admin solo sirve para el primero).
pub async fn existe_administrador(pool: &PgPool) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "select exists (select 1 from Acceso.UsuariosRoles ur \
                        join Acceso.Usuarios u on u.UsuariosCodigo = ur.UsuariosRolesUsuario \
                        where ur.UsuariosRolesRol = 'ADM' and u.UsuariosEstado = 'A')",
    )
    .fetch_one(pool)
    .await
}

// Convierte un error de "valor repetido" en el resultado que corresponde (documento o usuario).
// Cualquier otro error de la base se devuelve tal cual.
fn convertir_repetido(error: sqlx::Error) -> Result<ResultadoPersonal, sqlx::Error> {
    let repetido = error.as_database_error().and_then(|base| {
        if !matches!(base.kind(), ErrorKind::UniqueViolation) {
            return None;
        }
        let indice = base.constraint()?; // nombre del índice único que se violó
        if indice.contains("documento") {
            Some(ResultadoPersonal::DocumentoRepetido)
        } else if indice.contains("nombreusuario") {
            Some(ResultadoPersonal::UsuarioRepetido)
        } else {
            None
        }
    });
    repetido.map_or(Err(error), Ok)
}

// Da de alta a una persona del personal: empleado + usuario + roles, todo o nada (una transacción).
// Si algo falla (por ejemplo, el usuario o el documento ya existen), no queda nada a medias.
pub async fn crear_personal(
    pool: &PgPool,
    nuevo: &NuevoPersonal,
) -> Result<ResultadoPersonal, sqlx::Error> {
    let codigo_empleado = generar_codigo();
    let codigo_usuario = generar_codigo();
    let mut transaccion = pool.begin().await?;

    let empleado = sqlx::query(
        "insert into Acceso.Empleados (EmpleadosCodigo, EmpleadosTipoDocumento, EmpleadosNumeroDocumento, \
                EmpleadosPaterno, EmpleadosMaterno, EmpleadosNombres, EmpleadosTelefono, EmpleadosCorreo) \
         values ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&codigo_empleado)
    .bind(&nuevo.tipo_documento)
    .bind(&nuevo.numero_documento)
    .bind(&nuevo.paterno)
    .bind(nuevo.materno.as_deref())
    .bind(&nuevo.nombres)
    .bind(nuevo.telefono.as_deref())
    .bind(nuevo.correo.as_deref())
    .execute(&mut *transaccion)
    .await;
    if let Err(error) = empleado {
        return convertir_repetido(error);
    }

    let usuario = sqlx::query(
        "insert into Acceso.Usuarios (UsuariosCodigo, UsuariosEmpleado, UsuariosNombreUsuario, UsuariosClaveHash) \
         values ($1, $2, $3, $4)",
    )
    .bind(&codigo_usuario)
    .bind(&codigo_empleado)
    .bind(&nuevo.usuario)
    .bind(&nuevo.clave_hash)
    .execute(&mut *transaccion)
    .await;
    if let Err(error) = usuario {
        return convertir_repetido(error);
    }

    for rol in &nuevo.roles {
        sqlx::query(
            "insert into Acceso.UsuariosRoles (UsuariosRolesUsuario, UsuariosRolesRol) values ($1, $2)",
        )
        .bind(&codigo_usuario)
        .bind(rol)
        .execute(&mut *transaccion)
        .await?;
    }

    transaccion.commit().await?;
    Ok(ResultadoPersonal::Hecho)
}
