// flujo.rs: lo que el agente DICE en cada momento. Aquí irán los pasos de la conversación
// (registro, consentimiento, citas). Por ahora responde según la intención detectada.
// Regla: este archivo no escribe SQL; cuando necesite datos, llamará a funciones de otros módulos.

use super::motor::{detectar_intencion, Intencion};

// Lo que el agente responde: un texto y, opcionalmente, botones rápidos.
pub struct Respuesta {
    pub texto: String,
    pub opciones: Vec<String>,
}

// Atajo para armar una Respuesta a partir de textos fijos.
fn respuesta(texto: &str, opciones: &[&str]) -> Respuesta {
    Respuesta {
        texto: texto.to_string(),
        opciones: opciones.iter().map(|o| o.to_string()).collect(),
    }
}

// El primer mensaje de cualquier conversación nueva.
pub fn saludo_inicial() -> Respuesta {
    respuesta(
        "¡Hola! 👋 Soy el asistente de H&Devs. Puedo ayudarte con nuestro servicio técnico. \
         ¿Quieres que te registre para atenderte mejor, o solo estás de paso? \
         Si es así, no hay problema: puedes preguntarme lo que desees.",
        &["Quiero registrarme", "Solo estoy de paso"],
    )
}

// Decide qué responder a lo que escribió la persona.
// Por ahora son respuestas de prueba; en los próximos pasos las reemplaza el flujo real.
pub fn responder(texto: &str) -> Respuesta {
    match detectar_intencion(texto) {
        Intencion::Saludo => respuesta("¡Hola! ¿En qué te puedo ayudar con el servicio técnico?", &["Ver servicios", "Agendar una cita"]),
        Intencion::Registrarse => respuesta("Perfecto, quiero registrarte. (El registro lo construimos en el siguiente paso.)", &[]),
        Intencion::SoloDePaso => respuesta("¡Sin problema! Pregúntame lo que quieras sobre nuestros servicios.", &["Ver servicios", "Agendar una cita"]),
        Intencion::Servicios => respuesta("Con gusto te cuento sobre nuestros servicios. (La lista real la conectaremos a la base de datos.)", &["Agendar una cita"]),
        Intencion::AgendarCita => respuesta("Claro, vamos a agendar tu cita. (El agendamiento lo construimos más adelante.)", &[]),
        Intencion::Despedida => respuesta("¡Gracias a ti! Aquí estaré si necesitas algo más. 😊", &[]),
        Intencion::Desconocida => respuesta("Mmm, no estoy seguro de haber entendido. ¿Puedes decirlo de otra forma?", &["Ver servicios", "Agendar una cita"]),
    }
}
