// pasos.rs: los nombres de los PASOS de la conversación (el "mapa" del agente).
// La conversación siempre está en uno de estos pasos (se guarda en la base de datos) y, según el paso,
// lo que escribe la persona significa una cosa u otra. Están aquí, juntos, para ver el mapa completo
// de un vistazo y no equivocarnos al escribirlos.

pub const LIBRE: &str = "libre"; // conversación abierta: la persona pregunta lo que quiera

// Registro de un cliente nuevo.
pub const CONSENTIMIENTO: &str = "consentimiento";
pub const TIPO_DOCUMENTO: &str = "tipo_documento";
pub const NUMERO_DOCUMENTO: &str = "numero_documento";
pub const VERIFICAR_TELEFONO: &str = "verificar_telefono";
pub const CONFIRMAR_IDENTIDAD: &str = "confirmar_identidad";
pub const MANUAL_NOMBRES: &str = "manual_nombres";
pub const MANUAL_PATERNO: &str = "manual_paterno";
pub const MANUAL_MATERNO: &str = "manual_materno";
pub const MANUAL_RAZON_SOCIAL: &str = "manual_razon_social";
pub const DIRECCION: &str = "direccion";
pub const TELEFONO: &str = "telefono";
pub const CORREO: &str = "correo";
pub const AGREGAR_CORREO: &str = "agregar_correo"; // un cliente ya registrado agrega su correo después

// Consulta de servicios técnicos.
pub const ELEGIR_SERVICIO: &str = "elegir_servicio"; // la persona escoge un servicio de la lista
pub const ELEGIR_EQUIPO: &str = "elegir_equipo"; // la persona escoge para qué equipo lo necesita
