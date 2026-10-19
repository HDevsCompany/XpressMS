// respuesta.rs: las piezas comunes con las que cualquier parte del agente arma sus respuestas.

use super::modelo::Componente;

// Botones del menú principal.
pub const MENU: [&str; 2] = ["Ver servicios", "Agendar una cita"];

// Lo que el agente responde: un texto y, opcionalmente, botones rápidos, un elemento visual especial
// (casilla de términos, credencial, tarjeta de servicio) y un aviso de espera para el próximo mensaje.
pub struct Respuesta {
    pub texto: String,
    pub opciones: Vec<String>,
    pub componente: Option<Componente>,
    // Si tiene valor, el navegador mostrará ese texto con una animación ("Consultando tus datos…")
    // mientras procesa el SIGUIENTE mensaje de la persona (porque sabemos que será lento).
    pub espera: Option<String>,
}

impl Respuesta {
    // Agrega un elemento visual especial a la respuesta.
    pub fn con_componente(mut self, componente: Componente) -> Self {
        self.componente = Some(componente);
        self
    }

    // Agrega el aviso de espera para el próximo mensaje.
    pub fn con_espera(mut self, texto: &str) -> Self {
        self.espera = Some(texto.to_string());
        self
    }
}

// Cada paso devuelve: la respuesta + el nombre del paso en el que queda la conversación.
pub type Paso = (Respuesta, &'static str);

// Atajo para armar una Respuesta a partir de textos fijos.
pub fn r(texto: &str, opciones: &[&str]) -> Respuesta {
    Respuesta {
        texto: texto.to_string(),
        opciones: opciones.iter().map(|o| o.to_string()).collect(),
        componente: None,
        espera: None,
    }
}
