// identidad.rs: cómo se presenta el agente (su nombre y su icono).
// Hoy son valores fijos; más adelante podrán venir de una configuración de la plataforma
// (para que H&Devs le ponga otro nombre o icono sin tocar el código).

use serde::Serialize;

// Nombre y icono predeterminados del agente.
pub const NOMBRE_PREDETERMINADO: &str = "Juno";
pub const ICONO_PREDETERMINADO: &str = "/static/img/juno.png";

// Lo que el navegador necesita para presentar al agente.
#[derive(Serialize, Clone)]
pub struct Agente {
    pub nombre: String,
    pub icono: String,
}

// Por ahora siempre devuelve el agente predeterminado.
pub fn actual() -> Agente {
    Agente {
        nombre: NOMBRE_PREDETERMINADO.to_string(),
        icono: ICONO_PREDETERMINADO.to_string(),
    }
}
