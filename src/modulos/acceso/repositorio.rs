// repositorio.rs (acceso): consultas SQL de clientes. Ningún otro módulo escribe SQL sobre Acceso.Clientes.

use sqlx::PgPool;

use super::modelo::{ClienteCompleto, ClienteExistente, NuevoCliente};
use crate::modulos::comun::codigos::generar_codigo;

// Busca un cliente por su tipo y número de documento. Devuelve None si no está registrado.
pub async fn buscar_cliente_por_documento(
    pool: &PgPool,
    tipo: &str,
    numero: &str,
) -> Result<Option<ClienteExistente>, sqlx::Error> {
    sqlx::query_as::<_, ClienteExistente>(
        "select ClientesCodigo as codigo \
         from Acceso.Clientes \
         where ClientesTipoDocumento = $1 and ClientesNumeroDocumento = $2 and ClientesEstado = 'A'",
    )
    .bind(tipo)
    .bind(numero)
    .fetch_optional(pool)
    .await
}

// Devuelve el teléfono guardado de un cliente. Solo se usa para verificar identidad
// (comparar los últimos dígitos); nunca se le muestra completo a nadie.
pub async fn obtener_telefono(pool: &PgPool, codigo: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>("select ClientesTelefono from Acceso.Clientes where ClientesCodigo = $1")
        .bind(codigo)
        .fetch_optional(pool)
        .await
}

// Registra un cliente nuevo y devuelve su código de 9 caracteres.
// Nota sobre "$10::text::date": enviamos la fecha como texto y la base la convierte a fecha.
// "as_deref()" convierte Option<String> en Option<&str>, que es lo que sqlx necesita para enviar NULL.
pub async fn crear_cliente(pool: &PgPool, cliente: &NuevoCliente) -> Result<String, sqlx::Error> {
    let codigo = generar_codigo();

    sqlx::query(
        "insert into Acceso.Clientes \
         (ClientesCodigo, ClientesTipoDocumento, ClientesNumeroDocumento, ClientesPaterno, ClientesMaterno, \
          ClientesNombres, ClientesRazonSocial, ClientesDireccion, ClientesSexo, ClientesFechaNacimiento, \
          ClientesTelefono, ClientesCorreo, ClientesFechaConsentimiento) \
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::text::date, $11, $12, $13::text::timestamptz)",
    )
    .bind(&codigo)
    .bind(&cliente.tipo_documento)
    .bind(&cliente.numero_documento)
    .bind(cliente.paterno.as_deref())
    .bind(cliente.materno.as_deref())
    .bind(cliente.nombres.as_deref())
    .bind(cliente.razon_social.as_deref())
    .bind(&cliente.direccion)
    .bind(cliente.sexo.as_deref())
    .bind(cliente.fecha_nacimiento.as_deref())
    .bind(&cliente.telefono)
    .bind(cliente.correo.as_deref())
    .bind(&cliente.consentimiento)
    .execute(pool)
    .await?;

    Ok(codigo)
}

// Devuelve todos los datos de un cliente (o None si no existe). "fecha::text" entrega la fecha como AAAA-MM-DD.
pub async fn obtener_cliente(pool: &PgPool, codigo: &str) -> Result<Option<ClienteCompleto>, sqlx::Error> {
    sqlx::query_as::<_, ClienteCompleto>(
        "select ClientesTipoDocumento as tipo_documento, ClientesNumeroDocumento as numero_documento, \
                ClientesPaterno as paterno, ClientesMaterno as materno, ClientesNombres as nombres, \
                ClientesRazonSocial as razon_social, ClientesDireccion as direccion, ClientesSexo as sexo, \
                ClientesFechaNacimiento::text as fecha_nacimiento, ClientesTelefono as telefono, \
                ClientesCorreo as correo \
         from Acceso.Clientes where ClientesCodigo = $1",
    )
    .bind(codigo)
    .fetch_optional(pool)
    .await
}

// Guarda el correo de un cliente que aún no tenía uno. Devuelve true si se guardó, y false si ya tenía correo
// (la condición está en el SQL para que un correo ya registrado nunca se pise por accidente).
pub async fn agregar_correo(pool: &PgPool, codigo: &str, correo: &str) -> Result<bool, sqlx::Error> {
    let resultado = sqlx::query(
        "update Acceso.Clientes set ClientesCorreo = $1 \
         where ClientesCodigo = $2 and coalesce(ClientesCorreo, '') = ''",
    )
    .bind(correo)
    .bind(codigo)
    .execute(pool)
    .await?;

    Ok(resultado.rows_affected() > 0)
}
