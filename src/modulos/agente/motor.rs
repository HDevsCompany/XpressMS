// motor.rs: el "oído" del agente. Lee lo que escribió la persona y decide QUÉ quiere (su intención).
// Regla de este archivo: NO toca la base de datos ni la red. Solo recibe texto y devuelve una intención.
// Por eso se puede probar solo, con "cargo test", sin necesidad de Neon.

// Las cosas que el agente sabe reconocer por ahora.
// "derive(Debug, PartialEq)" permite imprimir el valor y compararlo (lo necesitan las pruebas de abajo).
#[derive(Debug, PartialEq)]
pub enum Intencion {
    Saludo,        // "hola", "buenas tardes"
    Registrarse,   // "quiero registrarme"
    AgregarCorreo, // "quiero agregar mi correo"
    SoloDePaso,    // "solo estoy de paso"
    Servicios,     // "¿qué servicios técnicos tienen?"
    AgendarCita,   // "quiero agendar una cita"
    Despedida,     // "gracias, chau"
    Desconocida,   // cualquier cosa que no entendamos
}

// Deja el texto en minúsculas y sin tildes, para comparar fácil:
// "¿Qué SERVICIOS tienen?" -> "¿que servicios tienen?"
pub fn normalizar(texto: &str) -> String {
    texto
        .to_lowercase() // todo a minúsculas
        .chars() // recorre letra por letra
        .map(|letra| match letra {
            // Cambia cada vocal con tilde por su versión simple.
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            otra => otra, // cualquier otra letra se queda igual
        })
        .collect() // junta las letras otra vez en un texto
}

// Decide la intención de un mensaje buscando palabras clave.
// OJO con el ORDEN: se revisa de arriba hacia abajo y gana la primera que coincida.
// Por eso "no quiero registrarme" se revisa ANTES que "registr".
pub fn detectar_intencion(texto: &str) -> Intencion {
    let t = normalizar(texto);

    // Mini-función: responde "true" si el mensaje contiene ALGUNA de las palabras dadas.
    let contiene = |palabras: &[&str]| palabras.iter().any(|palabra| t.contains(palabra));

    if contiene(&[
        "de paso",
        "solo miro",
        "solo estoy viendo",
        "no quiero registr",
        "no me quiero registr",
        "sin registr",
    ]) {
        Intencion::SoloDePaso
    } else if contiene(&["correo", "email", "e-mail", "mail"]) {
        // Va ANTES que "registr": "registrar mi correo" es agregar un correo, no empezar un registro.
        Intencion::AgregarCorreo
    } else if contiene(&["registr", "inscrib", "crear cuenta", "crear una cuenta"]) {
        Intencion::Registrarse
    } else if contiene(&["cita", "agendar", "reserv", "turno"]) {
        Intencion::AgendarCita
    } else if contiene(&[
        "servicio",
        "precio",
        "cuanto cuesta",
        "cuanto cobran",
        "reparac",
        "mantenimiento",
        "formateo",
        "tecnico",
    ]) {
        Intencion::Servicios
    } else if contiene(&["adios", "chau", "hasta luego", "gracias"]) {
        Intencion::Despedida
    } else if contiene(&[
        "hola",
        "buenas",
        "buen dia",
        "buenos dias",
        "hey",
        "saludos",
    ]) {
        Intencion::Saludo
    } else {
        Intencion::Desconocida
    }
}

// Separa un texto en palabras sueltas, ya normalizadas: "¡Sí, acepto!" -> ["si", "acepto"].
fn palabras(texto: &str) -> Vec<String> {
    normalizar(texto)
        .split(|c: char| !c.is_alphanumeric()) // corta en todo lo que no sea letra o número
        .filter(|palabra| !palabra.is_empty())
        .map(|palabra| palabra.to_string())
        .collect()
}

// ¿La persona dijo que NO? Se revisa primero porque "no acepto" también contiene "acepto".
pub fn es_negativo(texto: &str) -> bool {
    palabras(texto)
        .iter()
        .any(|p| ["no", "nunca", "rechazo", "negativo", "nel"].contains(&p.as_str()))
}

// ¿La persona dijo que SÍ? Si dijo que no, nunca cuenta como sí.
pub fn es_afirmativo(texto: &str) -> bool {
    !es_negativo(texto)
        && palabras(texto).iter().any(|p| {
            [
                "si",
                "claro",
                "acepto",
                "ok",
                "dale",
                "correcto",
                "correctos",
                "afirmativo",
                "yes",
                "vale",
                "soy",
                "listo",
            ]
            .contains(&p.as_str())
        })
}

// Decide cuál de las opciones mostradas eligió la persona, y devuelve su posición (o None si no coincide).
// Sirve para botones o escritura libre: "Laptop", "laptop" o "la lap" (si es única) eligen "Laptop".
// 1) Coincidencia exacta (sin importar mayúsculas ni tildes).
// 2) Coincidencia parcial: solo si lo escrito tiene 4 letras o más y encaja en UNA sola opción
//    (así "formateo" elige "Formateo e instalación de Windows", pero "pc" o "limpieza" repetido no adivina).
pub fn elegir_opcion(opciones: &[&str], texto: &str) -> Option<usize> {
    let normalizada = normalizar(texto);
    let entrada = normalizada.trim();
    if entrada.is_empty() {
        return None;
    }

    if let Some(posicion) = opciones
        .iter()
        .position(|opcion| normalizar(opcion) == entrada)
    {
        return Some(posicion);
    }

    if entrada.chars().count() >= 4 {
        let parciales: Vec<usize> = opciones
            .iter()
            .enumerate()
            .filter(|(_, opcion)| normalizar(opcion).contains(entrada))
            .map(|(posicion, _)| posicion)
            .collect();
        if parciales.len() == 1 {
            return Some(parciales[0]);
        }
    }
    None
}

// Pruebas automáticas: se ejecutan con "cargo test". Si cambias las palabras clave y rompes algo, avisan.
#[cfg(test)]
mod pruebas {
    use super::*; // trae todo lo de arriba para poder usarlo aquí

    #[test]
    fn reconoce_saludos() {
        assert_eq!(detectar_intencion("Hola, buenas tardes"), Intencion::Saludo);
    }

    #[test]
    fn reconoce_registro() {
        assert_eq!(
            detectar_intencion("Quiero registrarme"),
            Intencion::Registrarse
        );
    }

    #[test]
    fn reconoce_agregar_correo() {
        assert_eq!(
            detectar_intencion("Quiero agregar mi correo"),
            Intencion::AgregarCorreo
        );
        assert_eq!(
            detectar_intencion("Agregar mi correo"),
            Intencion::AgregarCorreo
        );
        assert_eq!(
            detectar_intencion("quiero registrar mi email"),
            Intencion::AgregarCorreo
        );
    }

    #[test]
    fn no_registrarse_es_de_paso() {
        assert_eq!(
            detectar_intencion("No quiero registrarme"),
            Intencion::SoloDePaso
        );
        assert_eq!(
            detectar_intencion("Solo estoy de paso"),
            Intencion::SoloDePaso
        );
    }

    #[test]
    fn cita_gana_sobre_servicio() {
        // Menciona "servicio" Y "cita": queremos que se entienda como agendar.
        assert_eq!(
            detectar_intencion("Quiero una cita para un servicio"),
            Intencion::AgendarCita
        );
    }

    #[test]
    fn reconoce_servicios_con_tildes_y_mayusculas() {
        assert_eq!(
            detectar_intencion("¿QUÉ SERVICIOS técnicos tienen?"),
            Intencion::Servicios
        );
        assert_eq!(
            detectar_intencion("¿Cuánto cuesta el formateo?"),
            Intencion::Servicios
        );
    }

    #[test]
    fn lo_raro_es_desconocido() {
        assert_eq!(detectar_intencion("asdfgh"), Intencion::Desconocida);
    }

    #[test]
    fn si_y_no() {
        assert!(es_afirmativo("Sí, acepto"));
        assert!(es_afirmativo("claro que sí"));
        assert!(es_afirmativo("Sí, soy yo"));
        assert!(es_afirmativo("Sí, son correctos"));
        assert!(es_negativo("No, no es mi empresa"));
        assert!(es_negativo("No, gracias"));
        assert!(es_negativo("no acepto"));
        assert!(!es_afirmativo("no acepto")); // un "no" nunca cuenta como sí
        assert!(!es_afirmativo("quizás")); // ni sí ni no
        assert!(!es_negativo("quizás"));
    }

    #[test]
    fn elige_opciones() {
        let opciones = ["Laptop", "PC de escritorio", "Impresora", "Impresión 3D"];
        assert_eq!(elegir_opcion(&opciones, "Laptop"), Some(0));
        assert_eq!(elegir_opcion(&opciones, "  laptop "), Some(0)); // mayúsculas y espacios no importan
        assert_eq!(elegir_opcion(&opciones, "escritorio"), Some(1)); // parcial y única
        assert_eq!(elegir_opcion(&opciones, "impre"), None); // parcial pero ambigua (Impresora / Impresión)
        assert_eq!(elegir_opcion(&opciones, "pc"), None); // muy corta para adivinar
        assert_eq!(elegir_opcion(&opciones, "celular"), None); // no existe
        assert_eq!(elegir_opcion(&opciones, ""), None);
        assert_eq!(elegir_opcion(&["Instalación"], "instalacion"), Some(0)); // sin tildes
    }
}
