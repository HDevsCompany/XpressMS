// cookies.rs: leer y armar la cookie de sesión. Son funciones puras (solo texto), se prueban con "cargo test".
//
// La cookie lleva el token secreto de la sesión con estas protecciones:
//   HttpOnly        -> ningún JavaScript de la página puede leerla (protege contra robo por scripts).
//   SameSite=Strict -> el navegador no la envía desde otros sitios (protege contra peticiones falsas).
//   Secure          -> solo viaja por HTTPS (se activa en Render; en localhost no hace falta).

// Nombre de la cookie en el navegador.
pub const NOMBRE: &str = "xms_sesion";

// Cuánto dura una sesión: una jornada de trabajo. Pasado ese tiempo hay que ingresar de nuevo.
pub const HORAS_SESION: i64 = 8;

// Busca una cookie por nombre dentro del encabezado "Cookie" ("a=1; b=2; xms_sesion=abc").
pub fn leer(encabezado: &str, nombre: &str) -> Option<String> {
    encabezado.split(';').find_map(|par| {
        let (clave, valor) = par.trim().split_once('=')?;
        if clave == nombre && !valor.is_empty() {
            Some(valor.to_string())
        } else {
            None
        }
    })
}

// Arma el texto de la cookie de sesión para guardarla en el navegador.
pub fn de_sesion(token: &str, segura: bool) -> String {
    armar(token, segura, HORAS_SESION * 3600)
}

// Arma una cookie vacía que vence al instante: así el navegador borra la de sesión.
pub fn de_cierre(segura: bool) -> String {
    armar("", segura, 0)
}

fn armar(valor: &str, segura: bool, segundos: i64) -> String {
    let mut cookie = format!(
        "{}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}",
        NOMBRE, valor, segundos
    );
    if segura {
        cookie.push_str("; Secure");
    }
    cookie
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn lee_la_cookie_entre_otras() {
        assert_eq!(leer("a=1; xms_sesion=abc123; b=2", NOMBRE), Some("abc123".to_string()));
        assert_eq!(leer("xms_sesion=solo", NOMBRE), Some("solo".to_string()));
    }

    #[test]
    fn no_confunde_nombres_parecidos() {
        assert_eq!(leer("otra_xms_sesion=1; xms_sesion2=2", NOMBRE), None);
        assert_eq!(leer("xms_sesion=", NOMBRE), None); // vacía
        assert_eq!(leer("", NOMBRE), None);
    }

    #[test]
    fn la_cookie_lleva_sus_protecciones() {
        let cookie = de_sesion("tok", false);
        assert!(cookie.starts_with("xms_sesion=tok;"));
        assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Strict"));
        assert!(cookie.contains("Max-Age=28800")); // 8 horas
        assert!(!cookie.contains("Secure"));
        assert!(de_sesion("tok", true).ends_with("; Secure"));
    }

    #[test]
    fn la_cookie_de_cierre_vence_al_instante() {
        let cookie = de_cierre(true);
        assert!(cookie.starts_with("xms_sesion=;"));
        assert!(cookie.contains("Max-Age=0"));
    }
}
