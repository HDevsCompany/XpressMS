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

#[cfg(test)]
mod pruebas {
    use super::*;

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
