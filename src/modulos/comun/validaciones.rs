// validaciones.rs: reglas para revisar lo que escribe la persona (documento, teléfono, correo, nombres).
// Son funciones "puras": reciben texto y devuelven un resultado, sin tocar base de datos ni red.
// Por eso se prueban solas con "cargo test". El backend es quien decide qué es válido; la base de datos
// solo guarda lo que ya pasó por aquí.

// Códigos de tipo de documento (catálogo 06 de SUNAT) que tienen reglas propias.
pub const TIPO_DNI: &str = "1";
pub const TIPO_RUC: &str = "6";

// ¿El texto está formado solo por dígitos (0-9) y no está vacío?
fn solo_digitos(texto: &str) -> bool {
    !texto.is_empty() && texto.chars().all(|c| c.is_ascii_digit())
}

// DNI: exactamente 8 dígitos.
fn dni_valido(numero: &str) -> bool {
    numero.len() == 8 && solo_digitos(numero)
}

// RUC: 11 dígitos, empieza con 10, 15, 16, 17 o 20, y el último dígito debe ser el "dígito verificador".
// El verificador se calcula multiplicando los 10 primeros dígitos por [5,4,3,2,7,6,5,4,3,2],
// sumando, y restando a 11 el resto de dividir esa suma entre 11 (10 pasa a 0 y 11 pasa a 1).
fn ruc_valido(numero: &str) -> bool {
    if numero.len() != 11 || !solo_digitos(numero) {
        return false;
    }
    if !["10", "15", "16", "17", "20"].contains(&&numero[..2]) {
        return false;
    }
    let digitos: Vec<u32> = numero.chars().filter_map(|c| c.to_digit(10)).collect();
    let pesos = [5, 4, 3, 2, 7, 6, 5, 4, 3, 2];
    let suma: u32 = pesos.iter().zip(digitos.iter()).map(|(peso, digito)| peso * digito).sum();
    let verificador = match 11 - (suma % 11) {
        10 => 0,
        11 => 1,
        otro => otro,
    };
    verificador == digitos[10]
}

// Carnet de extranjería y pasaporte: de 6 a 12 letras o números.
fn extranjero_valido(numero: &str) -> bool {
    (6..=12).contains(&numero.len()) && numero.chars().all(|c| c.is_ascii_alphanumeric())
}

// Revisa el número de documento según su tipo. Si es válido, lo devuelve limpio (sin espacios y en
// mayúsculas); si no, devuelve el mensaje que se le mostrará a la persona.
pub fn validar_documento(tipo: &str, texto: &str) -> Result<String, String> {
    // Quita espacios y guiones, y pasa a mayúsculas: "1234 5678" -> "12345678".
    let numero: String = texto
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect::<String>()
        .to_uppercase();

    if tipo == TIPO_DNI {
        if dni_valido(&numero) {
            Ok(numero)
        } else {
            Err("El DNI debe tener exactamente 8 números. Revísalo y escríbelo de nuevo.".to_string())
        }
    } else if tipo == TIPO_RUC {
        if ruc_valido(&numero) {
            Ok(numero)
        } else {
            Err("Ese RUC no parece válido. Debe tener 11 números y empezar con 10 o 20. Revísalo y escríbelo de nuevo.".to_string())
        }
    } else if extranjero_valido(&numero) {
        Ok(numero)
    } else {
        Err("El número debe tener entre 6 y 12 letras o números, sin símbolos. Revísalo y escríbelo de nuevo.".to_string())
    }
}

// Entiende "1995-03-15", "1995-03-15T00:00:00", "15/03/1995", "15-03-1995", "15.03.1995" y "15 03 1995", y
// devuelve "1995-03-15". Si la fecha no existe en el calendario (por ejemplo 31 de febrero) devuelve None:
// la base de datos la rechazaría.
pub fn normalizar_fecha(texto: &str) -> Option<String> {
    let t = texto.trim();
    let t = t.get(..10).unwrap_or(t); // descarta una hora si viniera pegada
    let partes: Vec<&str> = t.split(|c| c == '-' || c == '/' || c == '.' || c == ' ').collect();
    if partes.len() != 3 || partes.iter().any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    let (anio, mes, dia) = if partes[0].len() == 4 {
        (partes[0], partes[1], partes[2])
    } else if partes[2].len() == 4 {
        (partes[2], partes[1], partes[0])
    } else {
        return None;
    };
    let (anio, mes, dia): (u32, u32, u32) = (anio.parse().ok()?, mes.parse().ok()?, dia.parse().ok()?);
    let bisiesto = (anio % 4 == 0 && anio % 100 != 0) || anio % 400 == 0;
    let dias_del_mes = match mes {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if bisiesto => 29,
        2 => 28,
        _ => return None,
    };
    if !(1900..=2100).contains(&anio) || dia == 0 || dia > dias_del_mes {
        return None;
    }
    Some(format!("{:04}-{:02}-{:02}", anio, mes, dia))
}

// La fecha de nacimiento que escribe la persona: debe existir, no puede ser futura ni de hace más de 120 años.
// "hoy_iso" es el día de hoy como "2026-11-04" (se recibe como dato para poder probarla).
pub fn fecha_nacimiento_valida(texto: &str, hoy_iso: &str) -> Option<String> {
    let iso = normalizar_fecha(texto)?;
    let anio_hoy: i32 = hoy_iso.get(..4)?.parse().ok()?;
    let anio: i32 = iso.get(..4)?.parse().ok()?;
    if iso.as_str() > hoy_iso || anio < anio_hoy - 120 {
        None
    } else {
        Some(iso)
    }
}

// Celular peruano: 9 dígitos que empiezan con 9. Acepta "+51", espacios y guiones.
// Devuelve solo los 9 dígitos, o None si no es válido.
pub fn normalizar_telefono(texto: &str) -> Option<String> {
    let mut digitos: String = texto.chars().filter(|c| c.is_ascii_digit()).collect();
    // Si trae el código de país (51) delante de los 9 dígitos, se lo quitamos.
    if digitos.len() == 11 && digitos.starts_with("51") {
        digitos = digitos[2..].to_string();
    }
    if digitos.len() == 9 && digitos.starts_with('9') {
        Some(digitos)
    } else {
        None
    }
}

// Celular peruano en formato internacional E.164: "+51" y los 9 dígitos, sin espacios ("+51987654321").
// Es el formato que exigen los servicios de SMS. Acepta lo mismo que normalizar_telefono (9 dígitos, con
// "+51", espacios o guiones) y devuelve None si no es un celular válido.
pub fn a_e164(texto: &str) -> Option<String> {
    normalizar_telefono(texto).map(|nacional| format!("+51{}", nacional))
}

// Revisión básica de un correo: algo@dominio.ext, sin espacios y con una sola arroba.
pub fn correo_valido(texto: &str) -> bool {
    let correo = texto.trim();
    if correo.len() > 150 || correo.contains(char::is_whitespace) {
        return false;
    }
    let partes: Vec<&str> = correo.split('@').collect();
    if partes.len() != 2 {
        return false;
    }
    let (usuario, dominio) = (partes[0], partes[1]);
    !usuario.is_empty()
        && dominio.contains('.')
        && !dominio.starts_with('.')
        && !dominio.ends_with('.')
        && !dominio.contains("..")
}

// Nombres y apellidos: solo letras (con tildes y ñ), espacios, apóstrofes, puntos y guiones; máximo 100.
// Devuelve el texto limpio en mayúsculas, o None si no es válido.
pub fn limpiar_nombre(texto: &str) -> Option<String> {
    let limpio = texto.split_whitespace().collect::<Vec<_>>().join(" "); // quita espacios repetidos
    let valido = !limpio.is_empty()
        && limpio.chars().count() <= 100
        && limpio.chars().any(|c| c.is_alphabetic())
        && limpio
            .chars()
            .all(|c| c.is_alphabetic() || c == ' ' || c == '\'' || c == '.' || c == '-');
    if valido {
        Some(limpio.to_uppercase())
    } else {
        None
    }
}

// Nombre de usuario del personal: de 4 a 30 caracteres, solo letras sin tilde, números, punto, guion
// y guion bajo, y empieza con una letra. Devuelve el nombre en minúsculas (así "Pedro" y "pedro" son la
// misma cuenta), o None si no es válido.
pub fn normalizar_usuario(texto: &str) -> Option<String> {
    let usuario = texto.trim().to_lowercase();
    let largo = usuario.chars().count();
    let valido = (4..=30).contains(&largo)
        && usuario.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && usuario
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-' || c == '_');
    if valido {
        Some(usuario)
    } else {
        None
    }
}

// Clave del personal: de 10 a 128 caracteres, con al menos una letra y un número.
// Devuelve Ok(()) si cumple, o el mensaje que se le mostrará a la persona.
pub fn clave_valida(clave: &str) -> Result<(), String> {
    let largo = clave.chars().count();
    if largo < 10 {
        return Err("La clave debe tener al menos 10 caracteres.".to_string());
    }
    if largo > 128 {
        return Err("La clave no puede pasar de 128 caracteres.".to_string());
    }
    if !clave.chars().any(|c| c.is_alphabetic()) || !clave.chars().any(|c| c.is_ascii_digit()) {
        return Err("La clave debe tener al menos una letra y un número.".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod pruebas {

    #[test]
    fn fechas_escritas_de_varias_formas() {
        assert_eq!(normalizar_fecha("1995-03-15").as_deref(), Some("1995-03-15"));
        assert_eq!(normalizar_fecha("1995-03-15T00:00:00.000Z").as_deref(), Some("1995-03-15"));
        assert_eq!(normalizar_fecha("15/03/1995").as_deref(), Some("1995-03-15"));
        assert_eq!(normalizar_fecha("15.03.1995").as_deref(), Some("1995-03-15"));
        assert_eq!(normalizar_fecha(" 15 03 1995 ").as_deref(), Some("1995-03-15"));
        assert_eq!(normalizar_fecha("5-3-1995").as_deref(), Some("1995-03-05"));
        assert_eq!(normalizar_fecha("29/02/2000").as_deref(), Some("2000-02-29")); // bisiesto
        assert_eq!(normalizar_fecha("29/02/1900"), None); // 1900 no fue bisiesto
        assert_eq!(normalizar_fecha("31/02/1995"), None);
        assert_eq!(normalizar_fecha("1995-13-01"), None);
        assert_eq!(normalizar_fecha("1995-00-10"), None);
        assert_eq!(normalizar_fecha("0000-00-00"), None);
        assert_eq!(normalizar_fecha(""), None);
        assert_eq!(normalizar_fecha("ayer"), None);
        assert_eq!(normalizar_fecha("95-03-15"), None);
        assert_eq!(normalizar_fecha("15031995"), None);
    }

    #[test]
    fn fecha_de_nacimiento_razonable() {
        let hoy = "2026-11-04";
        assert_eq!(fecha_nacimiento_valida("15/03/1995", hoy).as_deref(), Some("1995-03-15"));
        assert_eq!(fecha_nacimiento_valida("04/11/2026", hoy).as_deref(), Some("2026-11-04")); // hoy mismo
        assert_eq!(fecha_nacimiento_valida("05/11/2026", hoy), None); // futura
        assert_eq!(fecha_nacimiento_valida("01/01/1900", hoy), None); // más de 120 años
        assert_eq!(fecha_nacimiento_valida("01/01/1906", hoy).as_deref(), Some("1906-01-01"));
        assert_eq!(fecha_nacimiento_valida("31/02/1990", hoy), None);
        assert_eq!(fecha_nacimiento_valida("hola", hoy), None);
    }
    use super::*;

    #[test]
    fn usuarios() {
        assert_eq!(normalizar_usuario("  Pedro.Guevara "), Some("pedro.guevara".to_string()));
        assert_eq!(normalizar_usuario("tec_01"), Some("tec_01".to_string()));
        assert_eq!(normalizar_usuario("abc"), None); // muy corto
        assert_eq!(normalizar_usuario("1pedro"), None); // empieza con número
        assert_eq!(normalizar_usuario("pedro luis"), None); // espacio
        assert_eq!(normalizar_usuario("péter"), None); // tilde
        assert_eq!(normalizar_usuario(&"a".repeat(31)), None); // muy largo
    }

    #[test]
    fn claves() {
        assert!(clave_valida("Taller2026ok").is_ok());
        assert!(clave_valida("corta1").is_err()); // menos de 10
        assert!(clave_valida("sololetrasaqui").is_err()); // sin número
        assert!(clave_valida("1234567890123").is_err()); // sin letra
        assert!(clave_valida(&format!("a1{}", "x".repeat(127))).is_err()); // más de 128
    }

    #[test]
    fn dni() {
        assert_eq!(validar_documento("1", "12 345-678"), Ok("12345678".to_string()));
        assert!(validar_documento("1", "1234567").is_err()); // 7 dígitos
        assert!(validar_documento("1", "1234567a").is_err()); // letra
    }

    #[test]
    fn ruc_con_digito_verificador() {
        assert!(validar_documento("6", "20615067131").is_ok()); // RUC real de H&Devs
        assert!(validar_documento("6", "20615067130").is_err()); // verificador mal
        assert!(validar_documento("6", "30615067131").is_err()); // prefijo inválido
        assert!(validar_documento("6", "2061506713").is_err()); // 10 dígitos
    }

    #[test]
    fn carnet_y_pasaporte() {
        assert_eq!(validar_documento("7", "ab123456"), Ok("AB123456".to_string()));
        assert!(validar_documento("4", "123").is_err()); // muy corto
        assert!(validar_documento("7", "AB 123 456 789 01").is_err()); // muy largo
        assert!(validar_documento("7", "AB12#456").is_err()); // símbolo
    }

    #[test]
    fn telefonos() {
        assert_eq!(normalizar_telefono("987 654 321"), Some("987654321".to_string()));
        assert_eq!(normalizar_telefono("+51 987-654-321"), Some("987654321".to_string()));
        assert_eq!(normalizar_telefono("887654321"), None); // no empieza con 9
    }

    #[test]
    fn telefonos_en_formato_internacional() {
        assert_eq!(a_e164("987654321"), Some("+51987654321".to_string()));
        assert_eq!(a_e164("987 654 321"), Some("+51987654321".to_string()));
        assert_eq!(a_e164("+51 987-654-321"), Some("+51987654321".to_string()));
        assert_eq!(a_e164("51987654321"), Some("+51987654321".to_string()));
        assert_eq!(a_e164("887654321"), None); // un celular peruano empieza con 9
        assert_eq!(a_e164("98765432"), None); // falta un dígito
        assert_eq!(a_e164("+1 987 654 321"), None); // otro país: no es un celular peruano
        assert_eq!(a_e164(""), None);
        assert_eq!(normalizar_telefono("98765"), None); // corto
    }

    #[test]
    fn correos() {
        assert!(correo_valido("pedro@gmail.com"));
        assert!(correo_valido("  pedro.g@empresa.com.pe "));
        assert!(!correo_valido("pedro@gmail"));
        assert!(!correo_valido("pedro@@gmail.com"));
        assert!(!correo_valido("pedro gmail.com"));
        assert!(!correo_valido("@gmail.com"));
    }

    #[test]
    fn nombres() {
        assert_eq!(limpiar_nombre("  josé   luis "), Some("JOSÉ LUIS".to_string()));
        assert_eq!(limpiar_nombre("O'Brien-Núñez"), Some("O'BRIEN-NÚÑEZ".to_string()));
        assert_eq!(limpiar_nombre("Pedro123"), None);
        assert_eq!(limpiar_nombre("   "), None);
    }
}
