// consulta_documento.rs: obtiene los datos de una persona o empresa a partir de su DNI o RUC.
//
// IMPORTANTE: por ahora esta consulta es SIMULADA (datos inventados) para poder construir y probar el
// registro sin gastar consultas. Cuando conectemos Factiliza, solo se reemplaza el cuerpo de
// "consultar": el resto del sistema no cambia, porque siempre recibe una "PersonaConsultada".
// Mientras sea simulada, NO se debe usar con clientes reales: guardaría datos falsos.

use crate::modulos::comun::validaciones::{TIPO_DNI, TIPO_RUC};

// Los datos que devuelve la consulta. Todo es opcional porque la fuente puede no traerlo.
pub struct PersonaConsultada {
    pub nombres: Option<String>,
    pub paterno: Option<String>,
    pub materno: Option<String>,
    pub razon_social: Option<String>,
    pub direccion: Option<String>,
    pub sexo: Option<String>,             // 'M' o 'F'
    pub fecha_nacimiento: Option<String>, // AAAA-MM-DD
}

// Consulta un documento. Devuelve None si no se encontró (la persona entonces escribe sus datos a mano).
// Es "async" porque la versión real esperará la respuesta de internet.
pub async fn consultar(tipo: &str, numero: &str) -> Option<PersonaConsultada> {
    // Regla de la simulación: los documentos que terminan en 0 "no se encuentran".
    // Sirve para probar el camino de ingreso manual.
    if numero.ends_with('0') {
        return None;
    }

    if tipo == TIPO_DNI {
        Some(PersonaConsultada {
            nombres: Some("JUAN CARLOS".to_string()),
            paterno: Some("PEREZ".to_string()),
            materno: Some("GOMEZ".to_string()),
            razon_social: None,
            direccion: Some("AV. ESPAÑA 123 - TRUJILLO".to_string()),
            sexo: Some("M".to_string()),
            fecha_nacimiento: Some("1995-03-15".to_string()),
        })
    } else if tipo == TIPO_RUC {
        Some(PersonaConsultada {
            nombres: None,
            paterno: None,
            materno: None,
            razon_social: Some("EMPRESA DE PRUEBA S.A.C.".to_string()),
            direccion: Some("CAL. LOS PINOS 456 - TRUJILLO".to_string()),
            sexo: None,
            fecha_nacimiento: None,
        })
    } else {
        None // carnet de extranjería y pasaporte no tienen consulta pública
    }
}
