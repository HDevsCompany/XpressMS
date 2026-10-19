// rutas.rs (agente): las "puertas" del chat. Aquí solo se reciben y se devuelven mensajes;
// la inteligencia vive en motor.rs (qué quiso decir) y, más adelante, en flujo.rs (qué sigue).

use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// "super" significa "el módulo de arriba" (agente); de allí tomamos motor.rs.
use super::motor::{detectar_intencion, Intencion};

// Lo que nos envía el navegador cuando la persona escribe. Ejemplo JSON: {"texto": "hola"}
// "Deserialize" = convertir ese JSON en esta estructura de Rust.
#[derive(Deserialize)]
struct MensajeEntrada {
    texto: String,
}

// Lo que le respondemos al navegador. Ejemplo JSON: {"respuesta": "...", "opciones": ["..."]}
// "Serialize" = convertir esta estructura de Rust en JSON.
// "opciones" son botones rápidos que el navegador mostrará debajo del mensaje.
#[derive(Serialize)]
struct MensajeSalida {
    respuesta: String,
    opciones: Vec<String>,
}

// Agrupa las rutas del agente. main.rs la usa con ".merge(...)" para sumarlas a la aplicación.
// Router<PgPool> significa "estas rutas viven dentro de una app que comparte el pool de la base".
pub fn rutas() -> Router<PgPool> {
    Router::new()
        .route("/agente/saludo", get(saludo))      // el navegador la llama al abrir la página
        .route("/agente/mensaje", post(mensaje))   // el navegador la llama cada vez que la persona escribe
}

// GET /agente/saludo: el primer mensaje del agente, con dos botones.
async fn saludo() -> Json<MensajeSalida> {
    Json(MensajeSalida {
        respuesta: "¡Hola! 👋 Soy el asistente de H&Devs. Puedo ayudarte con nuestro servicio técnico. \
                    ¿Quieres que te registre para atenderte mejor, o solo estás de paso? \
                    Si es así, no hay problema: puedes preguntarme lo que desees."
            .to_string(),
        opciones: vec!["Quiero registrarme".to_string(), "Solo estoy de paso".to_string()],
    })
}

// POST /agente/mensaje: recibe lo que escribió la persona y responde según su intención.
// "Json(entrada): Json<MensajeEntrada>" abre el JSON recibido y deja "entrada.texto" listo para usar.
async fn mensaje(Json(entrada): Json<MensajeEntrada>) -> Json<MensajeSalida> {
    // 1. El motor entiende qué quiso decir.
    let intencion = detectar_intencion(&entrada.texto);

    // 2. Elegimos la respuesta. Por ahora son textos de prueba: en el siguiente paso
    //    el flujo real (registro, citas) reemplazará estas respuestas.
    let (respuesta, opciones): (&str, Vec<&str>) = match intencion {
        Intencion::Saludo => ("¡Hola! ¿En qué te puedo ayudar con el servicio técnico?", vec!["Ver servicios", "Agendar una cita"]),
        Intencion::Registrarse => ("Perfecto, quiero registrarte. (El registro lo construimos en el siguiente paso.)", vec![]),
        Intencion::SoloDePaso => ("¡Sin problema! Pregúntame lo que quieras sobre nuestros servicios.", vec!["Ver servicios", "Agendar una cita"]),
        Intencion::Servicios => ("Con gusto te cuento sobre nuestros servicios. (La lista real la conectaremos a la base de datos.)", vec!["Agendar una cita"]),
        Intencion::AgendarCita => ("Claro, vamos a agendar tu cita. (El agendamiento lo construimos más adelante.)", vec![]),
        Intencion::Despedida => ("¡Gracias a ti! Aquí estaré si necesitas algo más. 😊", vec![]),
        Intencion::Desconocida => ("Mmm, no estoy seguro de haber entendido. ¿Puedes decirlo de otra forma?", vec!["Ver servicios", "Agendar una cita"]),
    };

    // 3. Armamos la respuesta, convirtiendo los textos fijos (&str) en textos propios (String).
    Json(MensajeSalida {
        respuesta: respuesta.to_string(),
        opciones: opciones.into_iter().map(|o| o.to_string()).collect(),
    })
}
