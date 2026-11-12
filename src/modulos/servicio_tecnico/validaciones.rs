// validaciones.rs (servicio_tecnico): reglas para revisar lo que escribe el personal al crear o editar
// servicios y tipos de equipo. Son funciones "puras" (reciben texto y devuelven un resultado, sin tocar
// la base de datos), por eso se prueban solas con "cargo test". La base solo guarda lo que ya pasó por aquí.

use std::collections::HashSet;

// Un servicio ya revisado y limpio, listo para guardarse.
pub struct DatosServicio {
    pub nombre: String,
    pub descripcion: String,
    pub precio_base: Option<String>, // "60.00"; None = "a cotizar"
    pub tiempo_estimado: Option<String>,
    pub equipos: Vec<String>, // códigos de tipo de equipo, sin repetidos
    pub aviso_precio: Option<bool>, // «El precio puede cambiar»; None = no tocarlo (al editar) o apagado (al crear)
}

// ---------- Imagen del servicio ----------

// Lo máximo que pesa una imagen ya reducida por el navegador (1.5 MB). Una foto normal reducida pesa unos 150 KB.
pub const MAXIMO_IMAGEN: usize = 1_500_000;

// Revisa los bytes de una imagen y devuelve su tipo ("image/jpeg", "image/png" o "image/webp"). No se confía en lo que diga el
// navegador: se mira el comienzo del propio archivo.
pub fn validar_imagen(bytes: &[u8]) -> Result<&'static str, String> {
    if bytes.is_empty() {
        return Err("No llegó ninguna imagen.".to_string());
    }
    if bytes.len() > MAXIMO_IMAGEN {
        return Err("La imagen pesa demasiado (máximo 1.5 MB). Prueba con una más pequeña.".to_string());
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok("image/jpeg");
    }
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Ok("image/png");
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Ok("image/webp");
    }
    Err("La imagen debe ser JPG, PNG o WEBP.".to_string())
}

// Limpia un texto: quita espacios de los bordes y espacios repetidos ("  a   b " -> "a b").
// Devuelve None si queda fuera del rango de caracteres permitido.
pub fn limpiar_texto(texto: &str, minimo: usize, maximo: usize) -> Option<String> {
    let limpio = texto.split_whitespace().collect::<Vec<_>>().join(" ");
    let largo = limpio.chars().count();
    if (minimo..=maximo).contains(&largo) {
        Some(limpio)
    } else {
        None
    }
}

// Precio base: acepta "60", "60.5", "60,50" y "S/ 60". Devuelve el precio con 2 decimales ("60.00"),
// o None si se dejó vacío (el servicio será "a cotizar"). Se trabaja con texto, sin decimales de
// computadora (f64), para que "60.10" nunca se convierta en "60.0999...".
pub fn normalizar_precio(texto: &str) -> Result<Option<String>, String> {
    let mensaje = "El precio debe ser un número (por ejemplo 60 o 60.50). Déjalo vacío si el servicio se cotiza.";
    let mut limpio = texto.trim();
    if limpio.len() >= 2 && limpio[..2].eq_ignore_ascii_case("s/") {
        limpio = limpio[2..].trim();
    }
    if limpio.is_empty() {
        return Ok(None);
    }
    let limpio = limpio.replace(',', ".");
    let (entero, decimales) = match limpio.split_once('.') {
        Some((entero, decimales)) => (entero, decimales),
        None => (limpio.as_str(), ""),
    };
    let solo_digitos = |t: &str| t.chars().all(|c| c.is_ascii_digit());
    // La columna es numeric(10,2): hasta 8 dígitos enteros y 2 decimales.
    if entero.is_empty() || entero.len() > 8 || decimales.len() > 2 || !solo_digitos(entero) || !solo_digitos(decimales) {
        return Err(mensaje.to_string());
    }
    let entero = entero.trim_start_matches('0');
    let entero = if entero.is_empty() { "0" } else { entero };
    Ok(Some(format!("{}.{:0<2}", entero, decimales)))
}

// Revisa todos los campos de un servicio. Si hay un error, devuelve el mensaje para mostrarle a la persona.
pub fn validar_servicio(
    nombre: &str,
    descripcion: &str,
    precio_base: Option<&str>,
    tiempo_estimado: Option<&str>,
    equipos: &[String],
) -> Result<DatosServicio, String> {
    let nombre = limpiar_texto(nombre, 3, 100)
        .ok_or("El nombre del servicio debe tener entre 3 y 100 caracteres.")?;
    let descripcion = limpiar_texto(descripcion, 10, 500)
        .ok_or("La descripción debe tener entre 10 y 500 caracteres.")?;
    let precio_base = normalizar_precio(precio_base.unwrap_or(""))?;
    let tiempo_estimado = match tiempo_estimado.unwrap_or("").trim() {
        "" => None,
        texto => Some(
            limpiar_texto(texto, 1, 40)
                .ok_or("El tiempo estimado no puede pasar de 40 caracteres.")?,
        ),
    };

    // Los equipos: al menos uno, códigos de 9 caracteres, sin repetir (se conserva el orden).
    let mut vistos = HashSet::new();
    let equipos: Vec<String> = equipos
        .iter()
        .map(|codigo| codigo.trim().to_string())
        .filter(|codigo| vistos.insert(codigo.clone()))
        .collect();
    if equipos.is_empty() {
        return Err("Elige al menos un tipo de equipo.".to_string());
    }
    if equipos.len() > 50 || equipos.iter().any(|codigo| codigo.chars().count() != 9) {
        return Err("Hay un tipo de equipo que no es válido. Recarga la página e inténtalo de nuevo.".to_string());
    }

    Ok(DatosServicio {
        nombre,
        descripcion,
        precio_base,
        tiempo_estimado,
        equipos,
        aviso_precio: None, // lo decide gestion.rs con lo que envió la persona
    })
}

// Nombre de un tipo de equipo: entre 2 y 60 caracteres.
pub fn validar_nombre_equipo(nombre: &str) -> Result<String, String> {
    limpiar_texto(nombre, 2, 60)
        .ok_or_else(|| "El nombre del tipo de equipo debe tener entre 2 y 60 caracteres.".to_string())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn equipos(lista: &[&str]) -> Vec<String> {
        lista.iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn precios_validos() {
        assert_eq!(normalizar_precio("60"), Ok(Some("60.00".to_string())));
        assert_eq!(normalizar_precio("60.5"), Ok(Some("60.50".to_string())));
        assert_eq!(normalizar_precio("60,50"), Ok(Some("60.50".to_string())));
        assert_eq!(normalizar_precio(" S/ 80 "), Ok(Some("80.00".to_string())));
        assert_eq!(normalizar_precio("007.1"), Ok(Some("7.10".to_string())));
        assert_eq!(normalizar_precio("0"), Ok(Some("0.00".to_string())));
        assert_eq!(normalizar_precio("60."), Ok(Some("60.00".to_string())));
    }

    #[test]
    fn precio_vacio_es_a_cotizar() {
        assert_eq!(normalizar_precio(""), Ok(None));
        assert_eq!(normalizar_precio("   "), Ok(None));
        assert_eq!(normalizar_precio("S/"), Ok(None));
    }

    #[test]
    fn precios_invalidos() {
        assert!(normalizar_precio("abc").is_err());
        assert!(normalizar_precio("-5").is_err());
        assert!(normalizar_precio("1.234").is_err()); // 3 decimales
        assert!(normalizar_precio("1.2.3").is_err());
        assert!(normalizar_precio(".5").is_err());
        assert!(normalizar_precio("123456789").is_err()); // 9 dígitos enteros
        assert!(normalizar_precio("6 0").is_err());
    }

    #[test]
    fn servicio_correcto_se_limpia() {
        let datos = validar_servicio(
            "  Mantenimiento   preventivo ",
            "Limpieza interna y cambio de pasta térmica del equipo.",
            Some("60"),
            Some(" Mismo día "),
            &equipos(&["AAAAAAAA2", "AAAAAAAA2", "BBBBBBBB3"]),
        )
        .unwrap();
        assert_eq!(datos.nombre, "Mantenimiento preventivo");
        assert_eq!(datos.precio_base, Some("60.00".to_string()));
        assert_eq!(datos.tiempo_estimado, Some("Mismo día".to_string()));
        assert_eq!(datos.equipos, equipos(&["AAAAAAAA2", "BBBBBBBB3"])); // sin repetidos
    }

    #[test]
    fn servicio_a_cotizar_sin_tiempo() {
        let datos = validar_servicio("Reparación de placa", "Diagnóstico y reparación de placa madre.", None, Some(""), &equipos(&["AAAAAAAA2"])).unwrap();
        assert_eq!(datos.precio_base, None);
        assert_eq!(datos.tiempo_estimado, None);
    }

    #[test]
    fn servicio_con_errores() {
        let ok_equipos = equipos(&["AAAAAAAA2"]);
        let descripcion = "Descripción suficientemente larga.";
        assert!(validar_servicio("ab", descripcion, None, None, &ok_equipos).is_err()); // nombre corto
        assert!(validar_servicio("Servicio", "corta", None, None, &ok_equipos).is_err());
        assert!(validar_servicio("Servicio", descripcion, Some("xx"), None, &ok_equipos).is_err());
        assert!(validar_servicio("Servicio", descripcion, None, Some(&"x".repeat(41)), &ok_equipos).is_err());
        assert!(validar_servicio("Servicio", descripcion, None, None, &[]).is_err()); // sin equipos
        assert!(validar_servicio("Servicio", descripcion, None, None, &equipos(&["corto"])).is_err());
        assert!(validar_servicio(&"a".repeat(101), descripcion, None, None, &ok_equipos).is_err());
    }

    #[test]
    fn imagenes_validas_e_invalidas() {
        assert_eq!(validar_imagen(&[0xFF, 0xD8, 0xFF, 0xE0, 0, 0]), Ok("image/jpeg"));
        assert_eq!(validar_imagen(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0]), Ok("image/png"));
        let mut webp = b"RIFF".to_vec();
        webp.extend_from_slice(&[1, 2, 3, 4]);
        webp.extend_from_slice(b"WEBPVP8 ");
        assert_eq!(validar_imagen(&webp), Ok("image/webp"));
        assert!(validar_imagen(&[]).is_err());
        assert!(validar_imagen(b"GIF89a....").is_err());
        assert!(validar_imagen(b"<svg></svg>").is_err());
        let mut grande = vec![0xFF, 0xD8, 0xFF];
        grande.resize(MAXIMO_IMAGEN + 1, 0);
        assert!(validar_imagen(&grande).is_err());
    }

    #[test]
    fn nombre_de_equipo() {
        assert_eq!(validar_nombre_equipo("  PC   de escritorio "), Ok("PC de escritorio".to_string()));
        assert!(validar_nombre_equipo("a").is_err());
        assert!(validar_nombre_equipo("   ").is_err());
        assert!(validar_nombre_equipo(&"a".repeat(61)).is_err());
    }
}
