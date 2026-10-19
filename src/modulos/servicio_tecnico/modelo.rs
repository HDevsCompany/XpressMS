// modelo.rs (servicio_tecnico): estructuras del módulo de servicios técnicos.

use sqlx::FromRow;

// Un servicio que ofrece la empresa (fila de ServicioTecnico.Servicios).
// Los nombres de los campos coinciden con los alias ("as nombre") de las consultas SQL.
#[derive(FromRow)]
pub struct Servicio {
    pub codigo: String,
    pub nombre: String,
    pub descripcion: String,
    pub precio_base: Option<String>, // el precio llega como texto ("60.00"); vacío = "a cotizar"
    pub tiempo_estimado: Option<String>,
}

// Un tipo de equipo que atiende la empresa (fila de ServicioTecnico.TiposEquipo): Laptop, Impresora...
#[derive(FromRow)]
pub struct TipoEquipo {
    pub codigo: String,
    pub nombre: String,
}
