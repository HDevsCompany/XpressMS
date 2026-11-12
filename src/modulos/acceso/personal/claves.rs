// claves.rs: todo lo que protege las claves y los tokens de sesión.
//
// Requiere las librerías "argon2" y "sha2" (se instalan con: cargo add argon2@0.5 sha2@0.10).
//   - Argon2: guarda la clave de forma que NO se puede recuperar (solo comprobar si coincide).
//     Es lento a propósito: así probar millones de claves tarda demasiado para un atacante.
//   - SHA-256: calcula la "huella" del token de sesión; en la base solo se guarda la huella.

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::modulos::comun::validaciones::clave_valida;

// Símbolos de las claves temporales: sin I, O, 0 y 1 (se confunden al leerlas o dictarlas).
const ALFABETO: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

// Posiciones de los bytes del UUID que usamos (se saltan la 6 y la 8, que llevan bits fijos).
const POSICIONES: [usize; 12] = [0, 1, 2, 3, 4, 5, 7, 9, 10, 11, 12, 13];

// Convierte una clave en su resultado Argon2 (con "sal" aleatoria incluida), listo para guardar.
// Dos claves iguales dan resultados distintos: la sal evita que se note quién repite clave.
pub fn hashear(clave: &str) -> Result<String, String> {
    // La sal son 16 bytes aleatorios (los mismos que usa uuid para sus códigos, seguros del sistema operativo).
    let sal = SaltString::encode_b64(Uuid::new_v4().as_bytes())
        .map_err(|error| format!("No se pudo preparar la clave: {}", error))?;
    Argon2::default()
        .hash_password(clave.as_bytes(), &sal)
        .map(|hash| hash.to_string())
        .map_err(|error| format!("No se pudo proteger la clave: {}", error))
}

// ¿Esta clave corresponde a ese resultado Argon2 guardado?
pub fn verificar(clave: &str, hash_guardado: &str) -> bool {
    match PasswordHash::new(hash_guardado) {
        Ok(hash) => Argon2::default()
            .verify_password(clave.as_bytes(), &hash)
            .is_ok(),
        Err(_) => false,
    }
}

// Argon2 usa bastante procesador y memoria. Se ejecuta en un hilo aparte ("spawn_blocking") para
// que, mientras una persona ingresa, el servidor siga atendiendo a los demás sin congelarse.
pub async fn hashear_async(clave: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || hashear(&clave))
        .await
        .map_err(|error| format!("Falló el cálculo de la clave: {}", error))?
}

pub async fn verificar_async(clave: String, hash_guardado: String) -> bool {
    tokio::task::spawn_blocking(move || verificar(&clave, &hash_guardado))
        .await
        .unwrap_or(false)
}

// La huella SHA-256 del token, en 64 caracteres hexadecimales. Es lo único que se guarda en la base.
pub fn huella_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

// Crea una clave temporal de 12 caracteres (por ejemplo "K7TQ3M9XA2HP") que cumple las reglas de
// clave_valida: se entrega una sola vez y la persona debe cambiarla en su primer ingreso.
pub fn generar_clave_temporal() -> String {
    loop {
        let clave = clave_desde_bytes(Uuid::new_v4().as_bytes());
        // Casi siempre cumple (letras y números); si por azar salió sin números, se genera otra.
        if clave_valida(&clave).is_ok() {
            return clave;
        }
    }
}

// Convierte bytes en una clave usando el ALFABETO. Está separada para poder probarla con bytes conocidos.
fn clave_desde_bytes(bytes: &[u8; 16]) -> String {
    POSICIONES
        .iter()
        .map(|&posicion| ALFABETO[(bytes[posicion] % 32) as usize] as char)
        .collect()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn la_clave_se_guarda_y_se_comprueba() {
        let hash = hashear("Taller2026ok").unwrap();
        assert!(hash.starts_with("$argon2")); // formato estándar: lleva sal y parámetros dentro
        assert!(verificar("Taller2026ok", &hash));
        assert!(!verificar("Taller2026OK", &hash)); // una mayúscula distinta ya no sirve
    }

    #[test]
    fn la_misma_clave_da_resultados_distintos() {
        assert_ne!(hashear("Taller2026ok").unwrap(), hashear("Taller2026ok").unwrap());
    }

    #[test]
    fn un_hash_dañado_nunca_acepta() {
        assert!(!verificar("lo-que-sea", "esto-no-es-un-hash"));
        assert!(!verificar("lo-que-sea", ""));
    }

    #[test]
    fn la_huella_es_sha256() {
        // Valor oficial de SHA-256 para "abc".
        assert_eq!(
            huella_token("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(huella_token("otro").len(), 64);
    }

    #[test]
    fn la_clave_temporal_es_valida_y_legible() {
        for _ in 0..200 {
            let clave = generar_clave_temporal();
            assert_eq!(clave.len(), 12);
            assert!(clave_valida(&clave).is_ok());
            assert!(!clave.contains(['I', 'O', '0', '1']));
        }
    }

    #[test]
    fn los_bytes_se_convierten_siempre_igual() {
        // 0 -> 'A', 1 -> 'B', 31 -> '9', 32 -> 'A'; las posiciones 6 y 8 (valor 255) se saltan.
        let bytes = [0, 1, 2, 3, 4, 5, 255, 31, 255, 32, 0, 0, 0, 0, 0, 0];
        assert_eq!(clave_desde_bytes(&bytes), "ABCDEF9AAAAA");
    }
}
