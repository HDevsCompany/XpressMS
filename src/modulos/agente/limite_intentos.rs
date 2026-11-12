// limite_intentos.rs: frena los intentos repetidos al verificar la identidad de un cliente.
//
// Sin este freno, alguien podría probar las 1000 combinaciones de «últimos 3 dígitos del celular»
// de un documento hasta acertar. Cada intento fallido se anota con su "clave" (por ejemplo el documento
// o la dirección de internet de quien intenta); si en la ventana de tiempo hay demasiados fallos,
// se bloquea esa clave hasta que pasen los minutos.
//
// Los intentos se guardan en la memoria del servidor (no en la base de datos): si el servidor se reinicia,
// el contador vuelve a cero. Es suficiente como freno; si algún día hace falta algo más estricto,
// se puede guardar en la base de datos sin cambiar a quien usa este módulo.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

// Tope de claves guardadas a la vez, para que la memoria no crezca sin límite.
const MAX_CLAVES: usize = 10_000;
// Pasado este tiempo, un fallo se olvida aunque nadie vuelva a intentar (la ventana más larga que se usa).
const OLVIDO: Duration = Duration::from_secs(60 * 60);

// El contador en sí. Recibe la hora actual como dato para poder probarse sin esperar de verdad.
pub struct Contador {
    fallos: HashMap<String, Vec<Instant>>,
}

impl Contador {
    pub fn nuevo() -> Self {
        Contador { fallos: HashMap::new() }
    }

    // ¿Esta clave ya hizo "maximo" fallos dentro de la "ventana"?
    pub fn bloqueado(&mut self, clave: &str, maximo: usize, ventana: Duration, ahora: Instant) -> bool {
        match self.fallos.get_mut(clave) {
            None => false,
            Some(lista) => {
                // Los fallos más viejos que la ventana ya no cuentan.
                lista.retain(|momento| ahora.duration_since(*momento) < ventana);
                lista.len() >= maximo
            }
        }
    }

    // Anota un fallo de esta clave.
    pub fn anotar(&mut self, clave: &str, ahora: Instant) {
        if self.fallos.len() >= MAX_CLAVES {
            // Limpieza: se olvidan las claves cuyos fallos ya son muy viejos.
            self.fallos.retain(|_, lista| {
                lista.retain(|momento| ahora.duration_since(*momento) < OLVIDO);
                !lista.is_empty()
            });
        }
        self.fallos.entry(clave.to_string()).or_default().push(ahora);
    }

    // Olvida los fallos de una clave (por ejemplo, cuando la persona sí acertó).
    pub fn limpiar(&mut self, clave: &str) {
        self.fallos.remove(clave);
    }
}

// El contador único de todo el servidor.
fn global() -> &'static Mutex<Contador> {
    static CONTADOR: OnceLock<Mutex<Contador>> = OnceLock::new();
    CONTADOR.get_or_init(|| Mutex::new(Contador::nuevo()))
}

// Si otro hilo falló mientras tenía el candado, seguimos usando lo guardado en lugar de caernos.
fn tomar() -> std::sync::MutexGuard<'static, Contador> {
    global().lock().unwrap_or_else(|error| error.into_inner())
}

pub fn bloqueado(clave: &str, maximo: usize, ventana: Duration) -> bool {
    tomar().bloqueado(clave, maximo, ventana, Instant::now())
}

pub fn anotar_fallo(clave: &str) {
    tomar().anotar(clave, Instant::now());
}

pub fn limpiar(clave: &str) {
    tomar().limpiar(clave);
}

#[cfg(test)]
mod pruebas {
    use super::*;

    const VENTANA: Duration = Duration::from_secs(15 * 60);

    #[test]
    fn bloquea_al_llegar_al_maximo_y_se_libera_con_el_tiempo() {
        let mut c = Contador::nuevo();
        let inicio = Instant::now();
        for _ in 0..4 {
            c.anotar("doc:1:12345678", inicio);
        }
        assert!(!c.bloqueado("doc:1:12345678", 5, VENTANA, inicio)); // 4 fallos: todavía puede
        c.anotar("doc:1:12345678", inicio);
        assert!(c.bloqueado("doc:1:12345678", 5, VENTANA, inicio)); // 5 fallos: bloqueado
        // Pasada la ventana, los fallos se olvidan.
        assert!(!c.bloqueado("doc:1:12345678", 5, VENTANA, inicio + VENTANA + Duration::from_secs(1)));
    }

    #[test]
    fn cada_clave_cuenta_por_separado_y_acertar_limpia() {
        let mut c = Contador::nuevo();
        let ahora = Instant::now();
        for _ in 0..5 {
            c.anotar("a", ahora);
        }
        assert!(c.bloqueado("a", 5, VENTANA, ahora));
        assert!(!c.bloqueado("b", 5, VENTANA, ahora));
        c.limpiar("a");
        assert!(!c.bloqueado("a", 5, VENTANA, ahora));
    }

    #[test]
    fn la_memoria_no_crece_sin_limite() {
        let mut c = Contador::nuevo();
        let inicio = Instant::now();
        for i in 0..MAX_CLAVES {
            c.anotar(&format!("k{i}"), inicio);
        }
        // Mucho tiempo después, al llegar al tope se limpian las claves viejas.
        c.anotar("nueva", inicio + OLVIDO + Duration::from_secs(1));
        assert_eq!(c.fallos.len(), 1);
    }
}
