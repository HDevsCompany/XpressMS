// formato.rs: convierte datos en texto presentable para la persona (fechas, sexo, direcciones).
// Son funciones "puras": no tocan la base de datos ni la red, por eso se prueban solas con "cargo test".

// "1995-03-15" -> "15/03/1995". Si el texto no tiene el formato esperado, lo devuelve igual.
pub fn formatear_fecha(iso: &str) -> String {
    let partes: Vec<&str> = iso.split('-').collect();
    if partes.len() == 3 {
        format!("{}/{}/{}", partes[2], partes[1], partes[0])
    } else {
        iso.to_string()
    }
}

// "1995-03-15" -> "••/••/1995": muestra solo el año, para que la persona se reconozca
// sin exponer su fecha de nacimiento completa.
pub fn fecha_enmascarada(iso: &str) -> String {
    match iso.split('-').next() {
        Some(anio) if anio.len() == 4 && anio.chars().all(|c| c.is_ascii_digit()) => {
            format!("••/••/{}", anio)
        }
        _ => "••/••/••••".to_string(),
    }
}

// "AV. ESPAÑA 123 - TRUJILLO" -> "•••••• - TRUJILLO": conserva solo la zona (lo que va después del
// último " - ") y oculta la calle y el número.
pub fn direccion_enmascarada(direccion: &str) -> String {
    match direccion.rsplit_once(" - ") {
        Some((_, zona)) if !zona.trim().is_empty() => format!("•••••• - {}", zona.trim()),
        _ => "••••••".to_string(),
    }
}

// "JUAN CARLOS PEREZ" -> "Juan Carlos Perez": primera letra de cada palabra en mayúscula y el resto en minúscula.
// Sirve para mostrar nombres y direcciones de forma amable (en la base de datos se guardan en mayúsculas).
pub fn a_titulo(texto: &str) -> String {
    texto
        .split_whitespace()
        .map(|palabra| {
            let mut letras = palabra.chars();
            match letras.next() {
                Some(primera) => {
                    primera.to_uppercase().collect::<String>() + &letras.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// "M" -> "Masculino", "F" -> "Femenino".
pub fn nombre_sexo(codigo: &str) -> &'static str {
    match codigo {
        "M" => "Masculino",
        "F" => "Femenino",
        _ => "No indicado",
    }
}

// Un celular con casi todo oculto, para mostrarlo sin exponerlo: "987654321" -> "+51 9•• ••• 321".
// Si no tiene 9 dígitos, devuelve un texto genérico.
pub fn telefono_enmascarado(telefono: &str) -> String {
    if telefono.len() == 9 && telefono.chars().all(|c| c.is_ascii_digit()) {
        format!("+51 {}•• ••• {}", &telefono[0..1], &telefono[6..9])
    } else {
        "tu celular".to_string()
    }
}

// "915054365" -> "+51 915 054 365". Si no son 9 dígitos, lo devuelve tal cual.
pub fn formatear_telefono(telefono: &str) -> String {
    if telefono.len() == 9 && telefono.chars().all(|c| c.is_ascii_digit()) {
        format!(
            "+51 {} {} {}",
            &telefono[0..3],
            &telefono[3..6],
            &telefono[6..9]
        )
    } else {
        telefono.to_string()
    }
}

// "juan.perez@gmail.com" -> "j•••@gmail.com" (para no mostrar el correo completo en el chat).
pub fn correo_enmascarado(correo: &str) -> String {
    match correo.split_once('@') {
        Some((usuario, dominio)) => {
            let inicial: String = usuario.chars().take(1).collect();
            format!("{}•••@{}", inicial, dominio)
        }
        None => "•••".to_string(),
    }
}

// El precio base de un servicio, listo para mostrar: Some("60.00") -> "Desde S/ 60.00"; None -> "A cotizar".
pub fn precio_base(precio: Option<&str>) -> String {
    match precio {
        Some(valor) => format!("Desde S/ {}", valor),
        None => "A cotizar".to_string(),
    }
}

// "12345678" -> "1•••••78": deja visibles el primer dígito y los dos últimos, para que la persona se reconozca
// sin exponer su documento. Si es muy corto, lo oculta todo.
pub fn documento_enmascarado(numero: &str) -> String {
    let letras: Vec<char> = numero.chars().collect();
    if letras.len() < 6 {
        return "••••".to_string();
    }
    let ocultas = "•".repeat(letras.len() - 3);
    let ultimas: String = letras[letras.len() - 2..].iter().collect();
    format!("{}{}{}", letras[0], ocultas, ultimas)
}

// "1" -> "DNI", "6" -> "RUC"... (los nombres de Comun.TiposDocumento).
pub fn nombre_tipo_documento(codigo: &str) -> &'static str {
    match codigo {
        "1" => "DNI",
        "6" => "RUC",
        "4" => "Carnet de extranjería",
        "7" => "Pasaporte",
        _ => "Documento",
    }
}

// "1999-03-14" -> "14 de marzo de 1999". Si no tiene el formato esperado, lo devuelve igual.
pub fn fecha_larga(iso: &str) -> String {
    const MESES: [&str; 12] = [
        "enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre",
        "noviembre", "diciembre",
    ];
    let partes: Vec<&str> = iso.get(..10).unwrap_or(iso).split('-').collect();
    if partes.len() == 3 {
        if let (Ok(dia), Ok(mes), Ok(anio)) = (partes[2].parse::<u32>(), partes[1].parse::<usize>(), partes[0].parse::<u32>()) {
            if (1..=12).contains(&mes) && (1..=31).contains(&dia) {
                return format!("{} de {} de {}", dia, MESES[mes - 1], anio);
            }
        }
    }
    iso.to_string()
}

// "Pedro Alonso Ramos Quispe" -> "PR": la inicial de la primera y de la última palabra (para el círculo del avatar).
pub fn iniciales(nombre: &str) -> String {
    let palabras: Vec<&str> = nombre.split_whitespace().collect();
    let primera = palabras.first().and_then(|p| p.chars().next());
    let ultima = palabras.last().and_then(|p| p.chars().next());
    match (primera, ultima) {
        (Some(a), Some(b)) if palabras.len() > 1 => format!("{}{}", a, b).to_uppercase(),
        (Some(a), _) => a.to_uppercase().to_string(),
        _ => "?".to_string(),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn fechas() {
        assert_eq!(formatear_fecha("1995-03-15"), "15/03/1995");
        assert_eq!(formatear_fecha("sin formato"), "sin formato");
        assert_eq!(fecha_enmascarada("1995-03-15"), "••/••/1995");
        assert_eq!(fecha_enmascarada("raro"), "••/••/••••");
    }

    #[test]
    fn direcciones() {
        assert_eq!(
            direccion_enmascarada("AV. ESPAÑA 123 - TRUJILLO"),
            "•••••• - TRUJILLO"
        );
        assert_eq!(
            direccion_enmascarada("JR. LIMA 45 - URB. SOL - LA ESPERANZA"),
            "•••••• - LA ESPERANZA"
        );
        assert_eq!(direccion_enmascarada("AV. ESPAÑA 123"), "••••••");
    }

    #[test]
    fn titulo() {
        assert_eq!(a_titulo("JUAN CARLOS  PEREZ"), "Juan Carlos Perez");
        assert_eq!(
            a_titulo("AV. ESPAÑA 123 - TRUJILLO"),
            "Av. España 123 - Trujillo"
        );
        assert_eq!(a_titulo("•••••• - TRUJILLO"), "•••••• - Trujillo");
        assert_eq!(a_titulo(""), "");
    }

    #[test]
    fn sexo() {
        assert_eq!(nombre_sexo("M"), "Masculino");
        assert_eq!(nombre_sexo("F"), "Femenino");
        assert_eq!(nombre_sexo("X"), "No indicado");
    }

    #[test]
    fn telefono() {
        assert_eq!(formatear_telefono("915054365"), "+51 915 054 365");
        assert_eq!(formatear_telefono("123"), "123");
        assert_eq!(telefono_enmascarado("987654321"), "+51 9•• ••• 321");
        assert_eq!(telefono_enmascarado("12"), "tu celular");
    }

    #[test]
    fn correo() {
        assert_eq!(correo_enmascarado("juan.perez@gmail.com"), "j•••@gmail.com");
        assert_eq!(correo_enmascarado("raro"), "•••");
    }

    #[test]
    fn precios() {
        assert_eq!(precio_base(Some("60.00")), "Desde S/ 60.00");
        assert_eq!(precio_base(None), "A cotizar");
    }

    #[test]
    fn documentos_y_nombres_del_perfil() {
        assert_eq!(documento_enmascarado("12345678"), "1•••••78");
        assert_eq!(documento_enmascarado("20123456789"), "2••••••••89");
        assert_eq!(documento_enmascarado("123"), "••••");
        assert_eq!(nombre_tipo_documento("1"), "DNI");
        assert_eq!(nombre_tipo_documento("9"), "Documento");
        assert_eq!(fecha_larga("1999-03-14"), "14 de marzo de 1999");
        assert_eq!(fecha_larga("1999-03-14T00:00:00"), "14 de marzo de 1999");
        assert_eq!(fecha_larga("raro"), "raro");
        assert_eq!(iniciales("Pedro Alonso Ramos Quispe"), "PQ");
        assert_eq!(iniciales("Juan"), "J");
        assert_eq!(iniciales(""), "?");
    }
}
