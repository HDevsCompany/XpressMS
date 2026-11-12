// repositorio.rs (perfil): aquí están TODAS las consultas SQL de las direcciones y de las personas autorizadas.
// Ningún otro archivo del perfil ni del agente escribe SQL sobre estas tablas.
// Las operaciones que tocan varias filas (cambiar la dirección principal) van en una transacción: o se hace todo o nada.

use sqlx::PgPool;

use super::modelo::{DatosCuenta, Direccion, Persona};
use crate::modulos::comun::codigos::generar_codigo;

// ---------- Cuenta ----------

pub async fn datos_de_cuenta(pool: &PgPool, cliente: &str) -> Result<Option<DatosCuenta>, sqlx::Error> {
    sqlx::query_as::<_, DatosCuenta>(
        "select ClientesCodigo as codigo, ClientesEstado as estado, \
                (ClientesTelefonoVerificado is not null) as celular_verificado, \
                to_char(ClientesFechaConsentimiento at time zone 'America/Lima', 'YYYY-MM-DD') as consentimiento \
         from Acceso.Clientes where ClientesCodigo = $1",
    )
    .bind(cliente)
    .fetch_optional(pool)
    .await
}

// ---------- Direcciones ----------

// La primera vez, copia la dirección con la que el cliente se registró como su dirección «Casa» principal.
// Si el cliente ya tiene (o tuvo) direcciones guardadas, no hace nada.
pub async fn asegurar_direccion_inicial(pool: &PgPool, cliente: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "insert into Acceso.ClientesDirecciones \
            (ClientesDireccionesCodigo, ClientesDireccionesCliente, ClientesDireccionesNombre, \
             ClientesDireccionesDireccion, ClientesDireccionesPrincipal) \
         select $1, c.ClientesCodigo, 'Casa', c.ClientesDireccion, true \
         from Acceso.Clientes c \
         where c.ClientesCodigo = $2 and coalesce(c.ClientesDireccion, '') <> '' \
           and not exists (select 1 from Acceso.ClientesDirecciones d where d.ClientesDireccionesCliente = c.ClientesCodigo)",
    )
    .bind(generar_codigo())
    .bind(cliente)
    .execute(pool)
    .await?;
    Ok(())
}

// Las direcciones activas: primero la principal y luego por orden de registro.
pub async fn listar_direcciones(pool: &PgPool, cliente: &str) -> Result<Vec<Direccion>, sqlx::Error> {
    sqlx::query_as::<_, Direccion>(
        "select ClientesDireccionesCodigo as codigo, ClientesDireccionesNombre as nombre, \
                ClientesDireccionesDireccion as direccion, ClientesDireccionesReferencia as referencia, \
                ClientesDireccionesPrincipal as principal \
         from Acceso.ClientesDirecciones \
         where ClientesDireccionesCliente = $1 and ClientesDireccionesEstado = 'A' \
         order by ClientesDireccionesPrincipal desc, ClientesDireccionesFechaRegistro, ClientesDireccionesCodigo",
    )
    .bind(cliente)
    .fetch_all(pool)
    .await
}

// Guarda una dirección nueva. Si es la principal, las demás dejan de serlo (todo en una transacción).
pub async fn crear_direccion(
    pool: &PgPool,
    cliente: &str,
    nombre: &str,
    direccion: &str,
    referencia: Option<&str>,
    principal: bool,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    if principal {
        sqlx::query(
            "update Acceso.ClientesDirecciones set ClientesDireccionesPrincipal = false \
             where ClientesDireccionesCliente = $1 and ClientesDireccionesPrincipal",
        )
        .bind(cliente)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        "insert into Acceso.ClientesDirecciones \
            (ClientesDireccionesCodigo, ClientesDireccionesCliente, ClientesDireccionesNombre, \
             ClientesDireccionesDireccion, ClientesDireccionesReferencia, ClientesDireccionesPrincipal) \
         values ($1, $2, $3, $4, $5, $6)",
    )
    .bind(generar_codigo())
    .bind(cliente)
    .bind(nombre)
    .bind(direccion)
    .bind(referencia)
    .bind(principal)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

// Cambia cuál es la dirección principal. Devuelve false si esa dirección no es de este cliente (o ya no está activa).
pub async fn hacer_principal(pool: &PgPool, cliente: &str, direccion: &str) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "update Acceso.ClientesDirecciones set ClientesDireccionesPrincipal = false \
         where ClientesDireccionesCliente = $1 and ClientesDireccionesPrincipal",
    )
    .bind(cliente)
    .execute(&mut *tx)
    .await?;
    let cambiada = sqlx::query(
        "update Acceso.ClientesDirecciones set ClientesDireccionesPrincipal = true \
         where ClientesDireccionesCodigo = $1 and ClientesDireccionesCliente = $2 and ClientesDireccionesEstado = 'A'",
    )
    .bind(direccion)
    .bind(cliente)
    .execute(&mut *tx)
    .await?
    .rows_affected()
        > 0;
    if cambiada {
        tx.commit().await?;
    } else {
        tx.rollback().await?; // no se dejó al cliente sin dirección principal
    }
    Ok(cambiada)
}

// Quita una dirección. Si era la principal y quedan otras, la más antigua pasa a ser la principal.
// Devuelve false si esa dirección no es de este cliente (o ya estaba quitada).
pub async fn quitar_direccion(pool: &PgPool, cliente: &str, direccion: &str) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let quitada = sqlx::query(
        "update Acceso.ClientesDirecciones \
         set ClientesDireccionesEstado = 'E', ClientesDireccionesPrincipal = false \
         where ClientesDireccionesCodigo = $1 and ClientesDireccionesCliente = $2 and ClientesDireccionesEstado = 'A'",
    )
    .bind(direccion)
    .bind(cliente)
    .execute(&mut *tx)
    .await?
    .rows_affected()
        > 0;
    if quitada {
        sqlx::query(
            "update Acceso.ClientesDirecciones set ClientesDireccionesPrincipal = true \
             where ClientesDireccionesCodigo = ( \
                 select ClientesDireccionesCodigo from Acceso.ClientesDirecciones \
                 where ClientesDireccionesCliente = $1 and ClientesDireccionesEstado = 'A' \
                 order by ClientesDireccionesFechaRegistro, ClientesDireccionesCodigo limit 1) \
               and not exists (select 1 from Acceso.ClientesDirecciones \
                               where ClientesDireccionesCliente = $1 and ClientesDireccionesEstado = 'A' \
                                 and ClientesDireccionesPrincipal)",
        )
        .bind(cliente)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(quitada)
}

// ---------- Personas autorizadas ----------

pub async fn listar_personas(pool: &PgPool, cliente: &str) -> Result<Vec<Persona>, sqlx::Error> {
    sqlx::query_as::<_, Persona>(
        "select PersonasAutorizadasCodigo as codigo, PersonasAutorizadasTipoDocumento as tipo_documento, \
                PersonasAutorizadasNumeroDocumento as numero_documento, PersonasAutorizadasNombre as nombre, \
                PersonasAutorizadasParentesco as parentesco \
         from Acceso.PersonasAutorizadas \
         where PersonasAutorizadasCliente = $1 and PersonasAutorizadasEstado = 'A' \
         order by PersonasAutorizadasFechaAutorizacion, PersonasAutorizadasCodigo",
    )
    .bind(cliente)
    .fetch_all(pool)
    .await
}

// Resultado de intentar autorizar a una persona.
#[derive(Debug, PartialEq, Eq)]
pub enum ResultadoAutorizacion {
    Autorizada,
    YaAutorizada,   // ese documento ya está en la lista del cliente
    LimiteAlcanzado, // ya tiene el máximo de personas
}

// Autoriza a una persona. El conteo, la revisión de duplicados y el guardado van en una sola instrucción,
// para que dos mensajes seguidos no puedan pasarse del máximo.
pub async fn autorizar_persona(
    pool: &PgPool,
    cliente: &str,
    tipo_documento: &str,
    numero_documento: &str,
    nombre: &str,
    parentesco: &str,
    maximo: i64,
) -> Result<ResultadoAutorizacion, sqlx::Error> {
    let ya_esta: bool = sqlx::query_scalar(
        "select exists (select 1 from Acceso.PersonasAutorizadas \
                        where PersonasAutorizadasCliente = $1 and PersonasAutorizadasTipoDocumento = $2 \
                          and PersonasAutorizadasNumeroDocumento = $3 and PersonasAutorizadasEstado = 'A')",
    )
    .bind(cliente)
    .bind(tipo_documento)
    .bind(numero_documento)
    .fetch_one(pool)
    .await?;
    if ya_esta {
        return Ok(ResultadoAutorizacion::YaAutorizada);
    }

    let guardadas = sqlx::query(
        "insert into Acceso.PersonasAutorizadas \
            (PersonasAutorizadasCodigo, PersonasAutorizadasCliente, PersonasAutorizadasTipoDocumento, \
             PersonasAutorizadasNumeroDocumento, PersonasAutorizadasNombre, PersonasAutorizadasParentesco) \
         select $1, $2, $3, $4, $5, $6 \
         where (select count(*) from Acceso.PersonasAutorizadas \
                where PersonasAutorizadasCliente = $2 and PersonasAutorizadasEstado = 'A') < $7",
    )
    .bind(generar_codigo())
    .bind(cliente)
    .bind(tipo_documento)
    .bind(numero_documento)
    .bind(nombre)
    .bind(parentesco)
    .bind(maximo)
    .execute(pool)
    .await?
    .rows_affected();

    Ok(if guardadas > 0 {
        ResultadoAutorizacion::Autorizada
    } else {
        ResultadoAutorizacion::LimiteAlcanzado
    })
}

// Quita la autorización. Devuelve false si esa persona no es de este cliente (o ya estaba quitada).
pub async fn quitar_persona(pool: &PgPool, cliente: &str, persona: &str) -> Result<bool, sqlx::Error> {
    let filas = sqlx::query(
        "update Acceso.PersonasAutorizadas set PersonasAutorizadasEstado = 'E' \
         where PersonasAutorizadasCodigo = $1 and PersonasAutorizadasCliente = $2 and PersonasAutorizadasEstado = 'A'",
    )
    .bind(persona)
    .bind(cliente)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(filas > 0)
}
