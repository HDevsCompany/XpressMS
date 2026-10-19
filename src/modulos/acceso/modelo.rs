// modelo.rs (acceso): estructuras del módulo de acceso.

use sqlx::FromRow;

// Los datos necesarios para registrar un cliente nuevo en Acceso.Clientes.
// Los campos "Option" pueden quedar vacíos (NULL en la base de datos).
pub struct NuevoCliente {
    pub tipo_documento: String,
    pub numero_documento: String,
    pub paterno: Option<String>,
    pub materno: Option<String>,
    pub nombres: Option<String>,
    pub razon_social: Option<String>,
    pub direccion: String,
    pub sexo: Option<String>,             // 'M' o 'F'
    pub fecha_nacimiento: Option<String>, // formato AAAA-MM-DD
    pub telefono: String,
    pub correo: Option<String>,
    pub consentimiento: String, // fecha y hora exactas en que aceptó el tratamiento de sus datos
}

// Un cliente que ya existe, con lo mínimo que necesitamos saber de él.
#[derive(FromRow)]
pub struct ClienteExistente {
    pub codigo: String,
}

// Todos los datos guardados de un cliente (para volver a dibujar su credencial).
// Los nombres de los campos coinciden con los alias ("as tipo_documento") de la consulta SQL.
#[derive(FromRow)]
pub struct ClienteCompleto {
    pub tipo_documento: String,
    pub numero_documento: String,
    pub paterno: Option<String>,
    pub materno: Option<String>,
    pub nombres: Option<String>,
    pub razon_social: Option<String>,
    pub direccion: String,
    pub sexo: Option<String>,
    pub fecha_nacimiento: Option<String>, // AAAA-MM-DD (la base lo convierte a texto)
    pub telefono: String,
    pub correo: Option<String>,
}
