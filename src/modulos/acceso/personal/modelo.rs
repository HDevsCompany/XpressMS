// modelo.rs (acceso/personal): las estructuras que viajan entre la base de datos, la lógica y el navegador.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// La persona que está usando el panel en este momento. Se envía al navegador (por eso es Serialize),
// así que NUNCA lleva la clave ni datos internos.
#[derive(Serialize, Clone, Debug)]
pub struct UsuarioActual {
    pub codigo: String,            // código del usuario (char(9))
    pub usuario: String,           // nombre de usuario ("pedro.guevara")
    pub nombre: String,            // para saludar: "Pedro Guevara"
    pub roles: Vec<String>,        // códigos de rol: ["ADM", "TEC"]
    pub debe_cambiar_clave: bool,  // true mientras tenga una clave temporal
}

// Fila de la base al reconocer una sesión (todavía sin los roles, que se consultan aparte).
#[derive(FromRow)]
pub struct FilaSesion {
    pub codigo: String,
    pub usuario: String,
    pub nombres: String,
    pub paterno: String,
    pub debe_cambiar_clave: bool,
}

// Fila de la base al validar usuario y clave.
#[derive(FromRow)]
pub struct FilaCredencial {
    pub codigo: String,
    pub clave_hash: String,
    pub debe_cambiar_clave: bool,
    pub bloqueado: bool, // true si BloqueadoHasta todavía no pasó
}

// Los datos para dar de alta a una persona del personal con su cuenta.
pub struct NuevoPersonal {
    pub tipo_documento: String,
    pub numero_documento: String,
    pub paterno: String,
    pub materno: Option<String>,
    pub nombres: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub usuario: String,
    pub clave_hash: String,
    pub roles: Vec<String>,
}

// ---------- Estructuras del panel de personal (solo las usa el Administrador) ----------

// Una persona del personal con su cuenta, tal como se muestra en la lista del panel.
// Nunca lleva la clave ni su resultado Argon2.
#[derive(FromRow, Serialize)]
pub struct FilaPersonal {
    pub codigo: String, // código del USUARIO: con él se editan, restablecen y eliminan
    pub tipo_documento: String,
    pub numero_documento: String,
    pub paterno: String,
    pub materno: Option<String>,
    pub nombres: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub usuario: String,
    pub debe_cambiar_clave: bool,
    pub bloqueado: bool,
    pub ultimo_acceso: Option<String>, // ya formateado en hora de Lima: "02/11/2026 14:03"
    pub roles: Vec<String>,
}

// Lo que envía el navegador al dar de alta a una persona (todavía sin revisar).
#[derive(Deserialize)]
pub struct EntradaAlta {
    pub tipo_documento: String,
    pub numero_documento: String,
    pub paterno: String,
    pub materno: Option<String>,
    pub nombres: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub usuario: String,
    pub roles: Vec<String>,
}

// Lo que envía el navegador al editar a una persona. El documento y el usuario no se cambian.
#[derive(Deserialize)]
pub struct EntradaEdicion {
    pub paterno: String,
    pub materno: Option<String>,
    pub nombres: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub roles: Vec<String>,
}

// La clave temporal que se entrega UNA sola vez (al dar de alta o al restablecer).
#[derive(Serialize)]
pub struct ClaveTemporal {
    pub usuario: String,
    pub clave_temporal: String,
}

// Cómo terminó una operación sobre el personal en la base.
pub enum ResultadoPersonal {
    Hecho,
    DocumentoRepetido, // ya hay una persona activa con ese documento
    UsuarioRepetido,   // ya hay una cuenta activa con ese nombre de usuario
    UltimoAdmin,       // la operación dejaría al sistema sin ningún administrador
    NoEncontrado,      // la persona no existe o ya fue eliminada
}
