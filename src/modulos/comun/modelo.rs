// modelo.rs (comun): estructuras de las tablas del esquema Comun.

use sqlx::FromRow;

// Una fila de Comun.TiposDocumento (DNI, RUC, Carnet de extranjería, Pasaporte).
#[derive(FromRow)]
pub struct TipoDocumento {
    pub codigo: String, // código SUNAT: "1" DNI, "6" RUC...
    pub nombre: String, // lo que ve la persona: "DNI", "RUC"...
}
