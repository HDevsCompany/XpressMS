// repositorio.rs (agente): aquí están TODAS las consultas SQL del módulo agente.
// Regla de arquitectura: ningún otro archivo del agente escribe SQL. Si algún día cambia la
// base de datos, solo se toca este archivo.
//
// Detalles de sqlx que se repiten abajo:
//   - "$1", "$2" son huecos que se rellenan con ".bind(...)". Así evitamos la inyección SQL
//     (nunca pegamos el texto de la persona dentro de la consulta).
//   - "as codigo" le pone nombre a la columna para que coincida con el campo de la estructura.

use sqlx::PgPool;

use super::modelo::{Conversacion, Datos, MensajeGuardado};
use crate::modulos::comun::codigos::{generar_codigo, generar_token};

// Crea una conversación nueva con código y token generados por el backend.
pub async fn crear_conversacion(pool: &PgPool) -> Result<Conversacion, sqlx::Error> {
    let codigo = generar_codigo(); // por ejemplo "K7TQ3M9XA"
    let token = generar_token();   // 32 caracteres secretos

    // El paso inicial y las fechas los pone la base de datos con sus valores por defecto.
    sqlx::query("insert into Agente.Conversaciones (ConversacionesCodigo, ConversacionesToken) values ($1, $2)")
        .bind(&codigo) // $1
        .bind(&token)  // $2
        .execute(pool) // ejecuta sin esperar filas de vuelta
        .await?;       // el "?" devuelve el error si algo falla

    Ok(Conversacion {
        codigo,
        token,
        paso: "inicio".to_string(),
        cliente: None,
        datos: "{}".to_string(),
    })
}

// Busca una conversación activa por su token. Devuelve "None" si no existe.
pub async fn buscar_por_token(pool: &PgPool, token: &str) -> Result<Option<Conversacion>, sqlx::Error> {
    sqlx::query_as::<_, Conversacion>(
        "select ConversacionesCodigo as codigo, ConversacionesToken as token, ConversacionesPaso as paso, \
                ConversacionesCliente as cliente, ConversacionesDatos::text as datos \
         from Agente.Conversaciones \
         where ConversacionesToken = $1 and ConversacionesEstado = 'A'",
    )
    .bind(token)
    .fetch_optional(pool) // 0 o 1 fila: devuelve Option
    .await
}

// Guarda un mensaje y marca la conversación como "activa ahora".
// "emisor" es 'P' (persona) o 'A' (agente).
pub async fn guardar_mensaje(
    pool: &PgPool,
    conversacion: &str,
    emisor: &str,
    texto: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "insert into Agente.Mensajes (MensajesCodigo, MensajesConversacion, MensajesEmisor, MensajesTexto) \
         values ($1, $2, $3, $4)",
    )
    .bind(generar_codigo()) // cada mensaje también lleva su código de 9 caracteres
    .bind(conversacion)
    .bind(emisor)
    .bind(texto)
    .execute(pool)
    .await?;

    // Actualiza la última actividad de la conversación (sirve para medir tiempos y abandonos).
    sqlx::query("update Agente.Conversaciones set ConversacionesFechaActividad = now() where ConversacionesCodigo = $1")
        .bind(conversacion)
        .execute(pool)
        .await?;

    Ok(())
}

// Devuelve todos los mensajes de una conversación, del más antiguo al más nuevo.
pub async fn listar_mensajes(pool: &PgPool, conversacion: &str) -> Result<Vec<MensajeGuardado>, sqlx::Error> {
    sqlx::query_as::<_, MensajeGuardado>(
        "select MensajesEmisor as emisor, MensajesTexto as texto \
         from Agente.Mensajes \
         where MensajesConversacion = $1 \
         order by MensajesFecha, MensajesCodigo",
    )
    .bind(conversacion)
    .fetch_all(pool) // todas las filas, en un Vec
    .await
}

// Guarda en qué paso va la conversación y lo que se ha recogido hasta ahora.
// "serde_json::to_string" convierte la estructura Datos en texto JSON; "$2::text::jsonb" lo convierte
// a JSON dentro de la base de datos.
pub async fn guardar_estado(
    pool: &PgPool,
    conversacion: &str,
    paso: &str,
    datos: &Datos,
) -> Result<(), sqlx::Error> {
    let json = serde_json::to_string(datos).unwrap_or_else(|_| "{}".to_string());

    sqlx::query(
        "update Agente.Conversaciones \
         set ConversacionesPaso = $1, ConversacionesDatos = $2::text::jsonb, ConversacionesFechaActividad = now() \
         where ConversacionesCodigo = $3",
    )
    .bind(paso)
    .bind(json)
    .bind(conversacion)
    .execute(pool)
    .await?;

    Ok(())
}

// Asocia la conversación con un cliente (porque se registró o porque verificó su identidad).
// "resultado" es opcional: al registrarse se marca 'registro'; si solo se identifica, se deja como está.
pub async fn vincular_cliente(
    pool: &PgPool,
    conversacion: &str,
    cliente: &str,
    resultado: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "update Agente.Conversaciones \
         set ConversacionesCliente = $1, ConversacionesResultado = coalesce($2::text, ConversacionesResultado) \
         where ConversacionesCodigo = $3",
    )
    .bind(cliente)
    .bind(resultado)
    .bind(conversacion)
    .execute(pool)
    .await?;

    Ok(())
}
