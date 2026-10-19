// repositorio.rs (comun): consultas SQL del esquema Comun.

use sqlx::PgPool;

use super::modelo::TipoDocumento;

// Lista los tipos de documento activos, para ofrecerlos como opciones en el chat.
// Si mañana agregas "Cédula diplomática" en la tabla, aparece solo: no hay que tocar el código.
pub async fn listar_tipos_documento(pool: &PgPool) -> Result<Vec<TipoDocumento>, sqlx::Error> {
    sqlx::query_as::<_, TipoDocumento>(
        "select TiposDocumentoCodigo as codigo, TiposDocumentoNombre as nombre \
         from Comun.TiposDocumento \
         where TiposDocumentoEstado = 'A' \
         order by TiposDocumentoNombre",
    )
    .fetch_all(pool)
    .await
}

// Devuelve la fecha y hora actuales de la base de datos como texto. Se usa para dejar constancia
// del momento exacto en que la persona dio su consentimiento.
pub async fn ahora_texto(pool: &PgPool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar::<_, String>("select now()::text").fetch_one(pool).await
}
