// codigo_otp.rs: las piezas "puras" del código de verificación de un solo uso (OTP) de 6 dígitos:
// generarlo, guardar solo su huella, compararlo y armar el texto del SMS. No toca la base de datos ni internet,
// así se prueba con "cargo test". El que las une con la base y con el envío es verificacion_telefono.rs.
//
// Requiere la librería "sha2" (ya instalada para las claves y sesiones del personal).

use sha2::{Digest, Sha256};

use crate::modulos::comun::codigos::generar_token;

// Cuántos dígitos tiene el código.
pub const LARGO_CODIGO: usize = 6;

// Cuántos minutos vale un código desde que se envía. 10 da margen por si el SMS tarda (el teléfono pasarela
// puede demorarse) y sigue siendo corto.
pub const VIGENCIA_MINUTOS: i32 = 10;

// Un número aleatorio entre 0 y 999 999, SIN favoritos: si usáramos "resto de dividir entre 1 000 000" a secas,
// los números bajos saldrían un poquito más seguido. Por eso se descartan los valores del final del rango que no
// completan una vuelta entera ("muestreo por rechazo"). Recibe los 4 bytes aleatorios como dato para poder probarse.
fn codigo_desde_bytes(bytes: [u8; 4]) -> Option<String> {
    const MODULO: u64 = 1_000_000;
    const TOPE: u64 = (u32::MAX as u64 + 1) / MODULO * MODULO; // 4 294 000 000: el último múltiplo completo
    let valor = u32::from_be_bytes(bytes) as u64;
    if valor >= TOPE {
        None // se descarta y se vuelve a sortear
    } else {
        Some(format!("{:0ancho$}", valor % MODULO, ancho = LARGO_CODIGO)) // con ceros a la izquierda: "004821"
    }
}

// Crea un código nuevo de 6 dígitos con números aleatorios seguros del sistema operativo
// (los mismos que usan los tokens de sesión: ver comun/codigos.rs).
pub fn generar_codigo_otp() -> String {
    loop {
        // generar_token() entrega 32 caracteres hexadecimales; los 8 primeros son 4 bytes completamente aleatorios.
        let token = generar_token();
        if let Some(bytes) = bytes_iniciales(&token) {
            if let Some(codigo) = codigo_desde_bytes(bytes) {
                return codigo;
            }
        }
    }
}

// Convierte los primeros 8 caracteres hexadecimales de un token en 4 bytes.
fn bytes_iniciales(token: &str) -> Option<[u8; 4]> {
    let mut bytes = [0u8; 4];
    for (posicion, byte) in bytes.iter_mut().enumerate() {
        let trozo = token.get(posicion * 2..posicion * 2 + 2)?;
        *byte = u8::from_str_radix(trozo, 16).ok()?;
    }
    Some(bytes)
}

// La "sal" propia de cada código: texto aleatorio que se mezcla antes de calcular la huella.
// Así, dos códigos iguales tienen huellas distintas y no sirven tablas de huellas ya calculadas.
pub fn generar_sal() -> String {
    generar_token() // 32 caracteres hexadecimales
}

// La huella (SHA-256 en hexadecimal, 64 caracteres) del código con su sal. Es lo único que se guarda:
// el código en sí solo existe en el SMS y en la mente de la persona.
pub fn huella(sal: &str, codigo: &str) -> String {
    format!("{:x}", Sha256::digest(format!("{}:{}", sal, codigo).as_bytes()))
}

// Compara dos huellas tardando lo mismo se parezcan o no (así no se filtra, por el tiempo de respuesta,
// cuántos caracteres iniciales coinciden).
pub fn huellas_iguales(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes().zip(b.bytes()).fold(0u8, |acumulado, (x, y)| acumulado | (x ^ y)) == 0
}

// Si lo que escribió la persona parece el código ("123456", "123 456", "123-456"), devuelve solo los 6 dígitos.
pub fn codigo_escrito(texto: &str) -> Option<String> {
    let digitos: String = texto.chars().filter(|c| c.is_ascii_digit()).collect();
    let solo_separadores = texto.chars().all(|c| c.is_ascii_digit() || c == ' ' || c == '-');
    if digitos.len() == LARGO_CODIGO && solo_separadores {
        Some(digitos)
    } else {
        None
    }
}

// El texto del SMS. Va SIN tildes ni "ñ" a propósito: con un solo carácter fuera del alfabeto básico de los SMS
// (como la "ó" de "código") el mensaje pasa a otra codificación y cabe menos por SMS (70 en lugar de 160),
// y puede partirse en dos y gastar el doble del paquete del teléfono.
pub fn mensaje_sms(codigo: &str) -> String {
    format!(
        "XpressMS: tu codigo de verificacion es {}. Vence en {} minutos. No lo compartas con nadie.",
        codigo, VIGENCIA_MINUTOS
    )
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_codigo_tiene_6_digitos() {
        for _ in 0..2000 {
            let codigo = generar_codigo_otp();
            assert_eq!(codigo.len(), LARGO_CODIGO);
            assert!(codigo.chars().all(|c| c.is_ascii_digit()), "{codigo}");
        }
    }

    #[test]
    fn los_codigos_varian() {
        let mut vistos = std::collections::HashSet::new();
        for _ in 0..200 {
            vistos.insert(generar_codigo_otp());
        }
        assert!(vistos.len() > 150, "salieron muy pocos códigos distintos: {}", vistos.len());
    }

    #[test]
    fn los_bytes_se_convierten_siempre_igual() {
        assert_eq!(codigo_desde_bytes([0, 0, 0, 0]), Some("000000".to_string()));
        assert_eq!(codigo_desde_bytes([0, 0, 0, 7]), Some("000007".to_string()));
        assert_eq!(codigo_desde_bytes([0, 0, 0x12, 0xD6]), Some("004822".to_string())); // 0x12D6 = 4822
        assert_eq!(codigo_desde_bytes([0, 0x0F, 0x42, 0x3F]), Some("999999".to_string())); // 999 999
        assert_eq!(codigo_desde_bytes([0, 0x0F, 0x42, 0x40]), Some("000000".to_string())); // 1 000 000 vuelve a 0
    }

    #[test]
    fn el_final_del_rango_se_descarta_para_no_tener_favoritos() {
        // 4 294 000 000 es el primer valor que ya no completa una vuelta: se descarta.
        assert_eq!(codigo_desde_bytes(4_294_000_000u32.to_be_bytes()), None);
        assert_eq!(codigo_desde_bytes(u32::MAX.to_be_bytes()), None);
        // El último valor aceptado.
        assert_eq!(codigo_desde_bytes(3_999_999_999u32.to_be_bytes()), Some("999999".to_string()));
        assert!(codigo_desde_bytes(4_293_999_999u32.to_be_bytes()).is_some());
    }

    #[test]
    fn los_bytes_se_leen_del_token() {
        assert_eq!(bytes_iniciales("00ff10a5deadbeef00000000deadbeef"), Some([0x00, 0xFF, 0x10, 0xA5]));
        assert_eq!(bytes_iniciales("zz"), None);
        assert_eq!(bytes_iniciales("abc"), None); // muy corto
    }

    #[test]
    fn la_huella_es_sha256_con_sal() {
        let h = huella("sal", "123456");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(h, huella("sal", "123456")); // siempre igual con los mismos datos
        assert_ne!(h, huella("otra", "123456")); // la sal cambia la huella
        assert_ne!(h, huella("sal", "123457")); // el código cambia la huella
        assert!(!h.contains("123456"));
    }

    #[test]
    fn las_sales_son_distintas() {
        let a = generar_sal();
        let b = generar_sal();
        assert_eq!(a.len(), 32);
        assert_ne!(a, b);
    }

    #[test]
    fn comparar_huellas() {
        let h = huella("s", "111111");
        assert!(huellas_iguales(&h, &huella("s", "111111")));
        assert!(!huellas_iguales(&h, &huella("s", "111112")));
        assert!(!huellas_iguales(&h, "corta"));
        assert!(huellas_iguales("", ""));
    }

    #[test]
    fn lo_que_escribe_la_persona() {
        assert_eq!(codigo_escrito("123456"), Some("123456".to_string()));
        assert_eq!(codigo_escrito("  123 456 "), Some("123456".to_string()));
        assert_eq!(codigo_escrito("123-456"), Some("123456".to_string()));
        assert_eq!(codigo_escrito("000123"), Some("000123".to_string()));
        assert_eq!(codigo_escrito("12345"), None); // faltan dígitos
        assert_eq!(codigo_escrito("1234567"), None); // sobran
        assert_eq!(codigo_escrito("mi codigo es 123456"), None); // no es solo el código
        assert_eq!(codigo_escrito("reenviar"), None);
    }

    #[test]
    fn el_sms_es_corto_y_sin_tildes() {
        let mensaje = mensaje_sms("123456");
        assert!(mensaje.contains("123456"));
        assert!(mensaje.is_ascii(), "debe ser solo ASCII para no partirse en dos SMS: {mensaje}");
        assert!(mensaje.len() <= 160);
        assert!(mensaje.contains("10 minutos"));
    }
}
