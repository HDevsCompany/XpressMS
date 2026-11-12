// validaciones.rs (acceso/personal): reglas para revisar lo que escribe el Administrador al dar de alta o
// editar al personal. Son funciones "puras" (reciben texto y devuelven un resultado, sin tocar la base),
// por eso se prueban solas con "cargo test". Las reglas de documento, teléfono, correo, nombres y usuario
// se reutilizan de comun/validaciones.rs; aquí solo se combinan.

use std::collections::HashSet;

use super::roles::{ROL_ADMIN, ROL_TECNICO, ROL_VENDEDOR};
use crate::modulos::comun::validaciones::{
    correo_valido, limpiar_nombre, normalizar_telefono, normalizar_usuario, validar_documento,
};

// Tipos de documento que acepta el personal: DNI, carnet de extranjería y pasaporte (no RUC).
const TIPOS_DOCUMENTO: [&str; 3] = ["1", "4", "7"];

// Los datos personales ya revisados y limpios.
pub struct DatosPersonales {
    pub paterno: String,
    pub materno: Option<String>,
    pub nombres: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
}

// Documento: el tipo debe ser uno de los permitidos y el número válido para ese tipo.
// Devuelve (tipo, número limpio).
pub fn validar_documento_personal(tipo: &str, numero: &str) -> Result<(String, String), String> {
    if !TIPOS_DOCUMENTO.contains(&tipo) {
        return Err("Elige un tipo de documento válido.".to_string());
    }
    let numero = validar_documento(tipo, numero)?;
    Ok((tipo.to_string(), numero))
}

// Apellidos, nombres, celular y correo. El apellido materno, el celular y el correo son opcionales.
pub fn validar_datos_personales(
    paterno: &str,
    materno: &str,
    nombres: &str,
    telefono: &str,
    correo: &str,
) -> Result<DatosPersonales, String> {
    let paterno = limpiar_nombre(paterno)
        .ok_or("El apellido paterno solo puede tener letras (y estar escrito).")?;
    let materno = match materno.trim() {
        "" => None,
        texto => Some(
            limpiar_nombre(texto).ok_or("El apellido materno solo puede tener letras.")?,
        ),
    };
    let nombres = limpiar_nombre(nombres).ok_or("Los nombres solo pueden tener letras (y estar escritos).")?;
    let telefono = match telefono.trim() {
        "" => None,
        texto => Some(
            normalizar_telefono(texto).ok_or("El celular debe tener 9 números y empezar con 9.")?,
        ),
    };
    let correo = match correo.trim() {
        "" => None,
        texto if correo_valido(texto) => Some(texto.to_lowercase()),
        _ => return Err("El correo no parece válido. Revísalo (ejemplo: nombre@empresa.com).".to_string()),
    };
    Ok(DatosPersonales {
        paterno,
        materno,
        nombres,
        telefono,
        correo,
    })
}

// Nombre de usuario: reutiliza la regla común, con un mensaje claro.
pub fn validar_usuario(texto: &str) -> Result<String, String> {
    normalizar_usuario(texto).ok_or_else(|| {
        "El usuario debe tener de 4 a 30 caracteres: letras sin tilde, números, punto, guion o guion bajo, empezando con una letra.".to_string()
    })
}

// Roles: al menos uno, solo los que existen, sin repetir (se conserva el orden).
pub fn validar_roles(roles: &[String]) -> Result<Vec<String>, String> {
    let mut vistos = HashSet::new();
    let roles: Vec<String> = roles
        .iter()
        .map(|rol| rol.trim().to_string())
        .filter(|rol| vistos.insert(rol.clone()))
        .collect();
    if roles.is_empty() {
        return Err("Elige al menos un rol.".to_string());
    }
    if roles
        .iter()
        .any(|rol| ![ROL_ADMIN, ROL_TECNICO, ROL_VENDEDOR].contains(&rol.as_str()))
    {
        return Err("Hay un rol que no es válido. Recarga la página e inténtalo de nuevo.".to_string());
    }
    Ok(roles)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn lista(roles: &[&str]) -> Vec<String> {
        roles.iter().map(|r| r.to_string()).collect()
    }

    #[test]
    fn documentos() {
        assert_eq!(validar_documento_personal("1", "12 345 678"), Ok(("1".to_string(), "12345678".to_string())));
        assert!(validar_documento_personal("1", "123").is_err());
        assert!(validar_documento_personal("6", "20615067131").is_err()); // RUC no permitido
        assert!(validar_documento_personal("4", "ab123456").is_ok()); // carnet de extranjería
        assert!(validar_documento_personal("9", "12345678").is_err());
    }

    #[test]
    fn datos_correctos_se_limpian() {
        let datos = validar_datos_personales(" guevara ", "", "pedro  juan", "+51 987 654 321", "Pedro@Gmail.com").unwrap();
        assert_eq!(datos.paterno, "GUEVARA");
        assert_eq!(datos.materno, None);
        assert_eq!(datos.nombres, "PEDRO JUAN");
        assert_eq!(datos.telefono, Some("987654321".to_string()));
        assert_eq!(datos.correo, Some("pedro@gmail.com".to_string()));
    }

    #[test]
    fn opcionales_vacios() {
        let datos = validar_datos_personales("Alvarado", "Rojas", "Ana", "  ", "").unwrap();
        assert_eq!(datos.materno, Some("ROJAS".to_string()));
        assert_eq!(datos.telefono, None);
        assert_eq!(datos.correo, None);
    }

    #[test]
    fn datos_con_errores() {
        assert!(validar_datos_personales("", "", "Ana", "", "").is_err()); // sin paterno
        assert!(validar_datos_personales("Rojas", "", "Ana1", "", "").is_err()); // número en nombre
        assert!(validar_datos_personales("Rojas", "R0jas", "Ana", "", "").is_err());
        assert!(validar_datos_personales("Rojas", "", "Ana", "12345", "").is_err()); // celular corto
        assert!(validar_datos_personales("Rojas", "", "Ana", "", "sin-arroba").is_err());
    }

    #[test]
    fn usuarios() {
        assert_eq!(validar_usuario(" Ana.Rojas "), Ok("ana.rojas".to_string()));
        assert!(validar_usuario("ab").is_err());
        assert!(validar_usuario("ana rojas").is_err());
    }

    #[test]
    fn roles() {
        assert_eq!(validar_roles(&lista(&["TEC", "TEC", "VEN"])), Ok(lista(&["TEC", "VEN"])));
        assert!(validar_roles(&[]).is_err());
        assert!(validar_roles(&lista(&["ADM", "JEFE"])).is_err());
        assert!(validar_roles(&lista(&["adm"])).is_err()); // los códigos van en mayúscula
    }
}
