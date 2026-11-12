// modelo.rs (perfil): las estructuras del perfil. "Serialize" las envía tal cual a la página como JSON;
// "FromRow" permite que sqlx las llene con el resultado de una consulta (los alias del SQL coinciden con los campos).

use serde::Serialize;
use sqlx::FromRow;

// Una dirección de entrega y recojo.
#[derive(FromRow, Serialize, Clone)]
pub struct Direccion {
    pub codigo: String,
    pub nombre: String, // "Casa", "Oficina"...
    pub direccion: String,
    pub referencia: Option<String>,
    pub principal: bool,
}

// Una persona autorizada a recoger los equipos. El documento viaja completo solo dentro del servidor:
// a la página llega enmascarado (ver rutas.rs).
#[derive(FromRow, Clone)]
pub struct Persona {
    pub codigo: String,
    pub tipo_documento: String,
    pub numero_documento: String,
    pub nombre: String,
    pub parentesco: String,
}

// Datos de la cuenta que no vienen en Acceso::ClienteCompleto.
#[derive(FromRow)]
pub struct DatosCuenta {
    pub codigo: String,
    pub estado: String,                    // 'A' activo
    pub celular_verificado: bool,          // ya confirmó su celular con el código SMS
    pub consentimiento: Option<String>,    // fecha en que aceptó los términos: AAAA-MM-DD (hora de Lima)
}
