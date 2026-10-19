// modelo.rs: las estructuras que representan los datos del agente en Rust.

use serde::Serialize;
use sqlx::FromRow;

// Una conversación guardada en Agente.Conversaciones.
// "FromRow" permite que sqlx llene esta estructura directamente con el resultado de una consulta.
// Los nombres de los campos deben coincidir con los alias ("as codigo") que se usan en el SQL.
#[derive(FromRow)]
pub struct Conversacion {
    pub codigo: String, // código de 9 caracteres (clave primaria)
    pub token: String,  // código secreto que guarda el navegador para reconocer la conversación
    #[allow(dead_code)] // todavía no lo leemos; lo usará el flujo de registro y de citas
    pub paso: String,   // en qué punto va la conversación ('inicio', luego los pasos del registro...)
}

// Un mensaje del chat, tal como se guarda en Agente.Mensajes.
// Además de FromRow, lleva "Serialize" porque se envía tal cual al navegador como JSON.
#[derive(FromRow, Serialize)]
pub struct MensajeGuardado {
    pub emisor: String, // 'P' = lo escribió la persona, 'A' = lo escribió el agente
    pub texto: String,  // el contenido del mensaje
}
