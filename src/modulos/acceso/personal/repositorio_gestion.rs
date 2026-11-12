// repositorio_gestion.rs (acceso/personal): el SQL con el que el Administrador ve y administra al personal.
// (El alta está en repositorio.rs, junto al resto del SQL de acceso.) Aquí no se decide nada: las reglas
// están en gestion.rs. Solo hay una excepción por seguridad de datos: "el sistema siempre debe tener al menos
// un administrador activo", que se comprueba aquí dentro de la transacción para que nunca quede a medias.
//
// "Eliminar" nunca borra la fila: pone Estado = 'E' en el empleado y en su usuario.

use sqlx::{PgConnection, PgPool};

use super::modelo::{FilaPersonal, ResultadoPersonal};
use super::validaciones::DatosPersonales;

// Cuántos administradores activos hay (con usuario y empleado activos).
async fn contar_administradores(conexion: &mut PgConnection) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "select count(*) from Acceso.UsuariosRoles ur \
         join Acceso.Usuarios u on u.UsuariosCodigo = ur.UsuariosRolesUsuario \
         join Acceso.Empleados e on e.EmpleadosCodigo = u.UsuariosEmpleado \
         where ur.UsuariosRolesRol = 'ADM' and u.UsuariosEstado = 'A' and e.EmpleadosEstado = 'A'",
    )
    .fetch_one(&mut *conexion)
    .await
}

// Todo el personal activo con su cuenta y roles, ordenado por apellido.
pub async fn listar(pool: &PgPool) -> Result<Vec<FilaPersonal>, sqlx::Error> {
    sqlx::query_as::<_, FilaPersonal>(
        "select u.UsuariosCodigo as codigo, e.EmpleadosTipoDocumento as tipo_documento, \
                e.EmpleadosNumeroDocumento as numero_documento, e.EmpleadosPaterno as paterno, \
                e.EmpleadosMaterno as materno, e.EmpleadosNombres as nombres, \
                e.EmpleadosTelefono as telefono, e.EmpleadosCorreo as correo, \
                u.UsuariosNombreUsuario as usuario, u.UsuariosDebeCambiarClave as debe_cambiar_clave, \
                (u.UsuariosBloqueadoHasta is not null and u.UsuariosBloqueadoHasta > now()) as bloqueado, \
                to_char(u.UsuariosUltimoAcceso at time zone 'America/Lima', 'DD/MM/YYYY HH24:MI') as ultimo_acceso, \
                array(select ur.UsuariosRolesRol::text from Acceso.UsuariosRoles ur \
                      where ur.UsuariosRolesUsuario = u.UsuariosCodigo order by 1) as roles \
         from Acceso.Usuarios u \
         join Acceso.Empleados e on e.EmpleadosCodigo = u.UsuariosEmpleado \
         where u.UsuariosEstado = 'A' and e.EmpleadosEstado = 'A' \
         order by e.EmpleadosPaterno, e.EmpleadosNombres",
    )
    .fetch_all(pool)
    .await
}

// Cambia los datos personales y reemplaza los roles, todo o nada.
// Si quedara sin administradores, se deshace todo y devuelve UltimoAdmin.
pub async fn actualizar(
    pool: &PgPool,
    usuario: &str,
    datos: &DatosPersonales,
    roles: &[String],
) -> Result<ResultadoPersonal, sqlx::Error> {
    let mut transaccion = pool.begin().await?;

    let filas = sqlx::query(
        "update Acceso.Empleados set EmpleadosPaterno = $2, EmpleadosMaterno = $3, EmpleadosNombres = $4, \
                EmpleadosTelefono = $5, EmpleadosCorreo = $6 \
         where EmpleadosEstado = 'A' and EmpleadosCodigo = \
               (select UsuariosEmpleado from Acceso.Usuarios where UsuariosCodigo = $1 and UsuariosEstado = 'A')",
    )
    .bind(usuario)
    .bind(&datos.paterno)
    .bind(datos.materno.as_deref())
    .bind(&datos.nombres)
    .bind(datos.telefono.as_deref())
    .bind(datos.correo.as_deref())
    .execute(&mut *transaccion)
    .await?;
    if filas.rows_affected() == 0 {
        return Ok(ResultadoPersonal::NoEncontrado);
    }

    sqlx::query("delete from Acceso.UsuariosRoles where UsuariosRolesUsuario = $1")
        .bind(usuario)
        .execute(&mut *transaccion)
        .await?;
    for rol in roles {
        sqlx::query(
            "insert into Acceso.UsuariosRoles (UsuariosRolesUsuario, UsuariosRolesRol) values ($1, $2)",
        )
        .bind(usuario)
        .bind(rol)
        .execute(&mut *transaccion)
        .await?;
    }

    if contar_administradores(&mut transaccion).await? == 0 {
        return Ok(ResultadoPersonal::UltimoAdmin); // al salir sin "commit", todo se deshace
    }
    transaccion.commit().await?;
    Ok(ResultadoPersonal::Hecho)
}

// Pone una clave temporal nueva: obliga a cambiarla, quita bloqueos y cierra todas las sesiones de esa cuenta.
// Devuelve el nombre de usuario, o None si la cuenta no existe o ya fue eliminada.
pub async fn restablecer_clave(
    pool: &PgPool,
    usuario: &str,
    clave_hash: &str,
) -> Result<Option<String>, sqlx::Error> {
    let mut transaccion = pool.begin().await?;
    let nombre = sqlx::query_scalar::<_, String>(
        "update Acceso.Usuarios set UsuariosClaveHash = $2, UsuariosDebeCambiarClave = true, \
                UsuariosIntentosFallidos = 0, UsuariosBloqueadoHasta = null \
         where UsuariosCodigo = $1 and UsuariosEstado = 'A' \
         returning UsuariosNombreUsuario",
    )
    .bind(usuario)
    .bind(clave_hash)
    .fetch_optional(&mut *transaccion)
    .await?;
    if nombre.is_none() {
        return Ok(None);
    }
    sqlx::query("update Acceso.Sesiones set SesionesEstado = 'E' where SesionesUsuario = $1")
        .bind(usuario)
        .execute(&mut *transaccion)
        .await?;
    transaccion.commit().await?;
    Ok(nombre)
}

// Da de baja a la persona: elimina su usuario y su empleado y cierra sus sesiones.
// Si fuera el último administrador, no se hace nada y devuelve UltimoAdmin.
pub async fn eliminar(pool: &PgPool, usuario: &str) -> Result<ResultadoPersonal, sqlx::Error> {
    let mut transaccion = pool.begin().await?;
    let empleado = sqlx::query_scalar::<_, String>(
        "update Acceso.Usuarios set UsuariosEstado = 'E' \
         where UsuariosCodigo = $1 and UsuariosEstado = 'A' \
         returning UsuariosEmpleado",
    )
    .bind(usuario)
    .fetch_optional(&mut *transaccion)
    .await?;
    let Some(empleado) = empleado else {
        return Ok(ResultadoPersonal::NoEncontrado);
    };
    sqlx::query("update Acceso.Empleados set EmpleadosEstado = 'E' where EmpleadosCodigo = $1")
        .bind(&empleado)
        .execute(&mut *transaccion)
        .await?;
    sqlx::query("update Acceso.Sesiones set SesionesEstado = 'E' where SesionesUsuario = $1")
        .bind(usuario)
        .execute(&mut *transaccion)
        .await?;

    if contar_administradores(&mut transaccion).await? == 0 {
        return Ok(ResultadoPersonal::UltimoAdmin);
    }
    transaccion.commit().await?;
    Ok(ResultadoPersonal::Hecho)
}
