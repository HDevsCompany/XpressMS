// modelo.rs: las estructuras que representan los datos del agente en Rust.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Una conversación guardada en Agente.Conversaciones.
// "FromRow" permite que sqlx llene esta estructura directamente con el resultado de una consulta.
// Los nombres de los campos deben coincidir con los alias ("as codigo") que se usan en el SQL.
#[derive(FromRow)]
pub struct Conversacion {
    pub codigo: String,          // código de 9 caracteres (clave primaria)
    pub token: String, // código secreto que guarda el navegador para reconocer la conversación
    pub paso: String,  // en qué punto va ('inicio', 'consentimiento', 'telefono'...)
    pub cliente: Option<String>, // código del cliente, si la persona ya se registró o se identificó
    pub datos: String, // lo recogido hasta ahora (JSON guardado como texto)
}

// Lo que el agente va recogiendo durante la conversación. Se guarda en la columna ConversacionesDatos.
// Los datos del registro se borran al terminar (los definitivos viven en Acceso.Clientes);
// el servicio y el equipo elegidos se conservan para la futura cita.
// "Default" crea una versión vacía; "serde(default)" permite leer JSON antiguos aunque falten campos.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Datos {
    pub consentimiento: Option<String>, // fecha y hora exactas en que aceptó el tratamiento de sus datos
    pub tipo_documento: Option<String>, // código SUNAT: "1", "6"...
    pub numero_documento: Option<String>,
    pub paterno: Option<String>,
    pub materno: Option<String>,
    pub nombres: Option<String>,
    pub razon_social: Option<String>,
    pub direccion: Option<String>,
    pub sexo: Option<String>,
    pub fecha_nacimiento: Option<String>,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub cliente_existente: Option<String>, // si el documento ya estaba registrado: su código
    pub intentos: u8,                      // intentos fallidos al verificar identidad
    pub servicio_elegido: Option<String>,  // código del servicio que la persona está viendo
    pub equipo_elegido: Option<String>,    // código del tipo de equipo elegido para ese servicio
}

// Un mensaje del chat, tal como se guarda en Agente.Mensajes.
// Además de FromRow, lleva "Serialize" porque se envía tal cual al navegador como JSON.
#[derive(FromRow, Serialize)]
pub struct MensajeGuardado {
    pub emisor: String, // 'P' = lo escribió la persona, 'A' = lo escribió el agente
    pub texto: String,  // el contenido del mensaje
}

// Estado de verificación de un dato de contacto (se dibuja como un icono junto al dato).
// "Fallido" y "Verificado" todavía no se usan: se activarán cuando exista el envío de SMS de confirmación.
#[allow(dead_code)]
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verificacion {
    Verificado, // check verde: la persona confirmó el dato
    Pendiente,  // reloj amarillo: aún no se confirma
    Fallido,    // x roja: no se pudo confirmar
}

// Una fila de la tarjeta de datos: "Nombres" -> "JUAN CARLOS".
// "verificacion" es opcional: solo los datos de contacto (celular) la llevan; si es None, no se envía en el JSON.
#[derive(Serialize)]
pub struct Campo {
    pub etiqueta: String,
    pub valor: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verificacion: Option<Verificacion>,
}

// Elementos visuales especiales que el agente puede pedirle al navegador que dibuje,
// además de los mensajes de texto. El JSON lleva un campo "tipo" que dice cuál es:
//   {"tipo":"consentimiento"}  -> casilla de términos y condiciones
//   {"tipo":"identidad", ...}  -> credencial (carnet) con los datos encontrados
//   {"tipo":"servicio", ...}   -> tarjeta con el detalle de un servicio técnico
#[derive(Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Componente {
    Consentimiento,
    Identidad {
        variante: String, // "persona" o "empresa": cambia el icono y el color de la credencial
        nombre: String,   // lo que va en grande: "Juan Carlos Perez Gomez"
        subtitulo: String, // lo que va debajo del nombre: "Persona natural"
        campos: Vec<Campo>, // el resto de datos (documento, sexo, nacimiento, dirección)
        estado: String, // "Pendiente de confirmar" (tarjeta de confirmación) o "Activo" (cliente ya registrado)
        // Código de cliente: solo existe en la credencial final. Si falta, no se envía el campo en el JSON.
        #[serde(skip_serializing_if = "Option::is_none")]
        codigo: Option<String>,
    },
    Servicio {
        nombre: String,         // "Formateo e instalación de Windows"
        descripcion: String,    // qué incluye el servicio
        precio: String,         // ya listo para mostrar: "Desde S/ 60.00" o "A cotizar"
        nota: String, // aviso sobre el precio (el costo final se confirma al revisar el equipo)
        tiempo: Option<String>, // "1 día" (si el servicio lo tiene)
        equipo: Option<String>, // el tipo de equipo elegido, "Laptop" (si corresponde)
    },
}
