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
    YaSoyCliente,  // "ya soy cliente": quiere que lo reconozcamos con su documento
    Servicios,     // "¿qué servicios técnicos tienen?"
    AgendarCita,   // "quiero agendar una cita"
    MiOrden,       // "seguimiento", "ver mi orden"
    CancelarCita,  // "cancelar mi cita"
    Productos,     // "¿qué productos tienen?", "ver productos"
    Apartar,       // "apartar este producto"
    MiApartado,    // "mis apartados"
    CancelarApartado, // "cancelar mi apartado"
    Direcciones,   // "mis direcciones", "añadir una dirección"
    Personas,      // "autorizar a alguien para recoger"
    CambiarCelular, // "cambiar mi celular"
    CorregirDatos, // "corregir mis datos"
    PedirDatos,    // "pedir mis datos"
    EliminarCuenta, // "eliminar mi cuenta"
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
    } else if contiene(&[
        "ya soy cliente",
        "ya estoy registr",
        "ya me registre",
        "ya tengo cuenta",
        "ya tengo registro",
    ]) {
        // Va ANTES que "registr": "ya me registré" es identificarse, no empezar un registro.
        Intencion::YaSoyCliente
    } else if contiene(&[
        "eliminar mi cuenta",
        "eliminar cuenta",
        "borrar mi cuenta",
        "borrar cuenta",
        "dar de baja",
        "eliminar mis datos",
        "borrar mis datos",
    ]) {
        // Mi perfil. Estas frases van ANTES que «datos», «registr» y el resto: son decisiones serias.
        Intencion::EliminarCuenta
    } else if contiene(&[
        "pedir mis datos",
        "pedir una copia",
        "copia de mis datos",
        "descargar mis datos",
        "solicitar mis datos",
        "ver mis datos",
    ]) {
        // Va ANTES que «corregir mis datos»: «pedir mis datos» es solo pedir una copia.
        Intencion::PedirDatos
    } else if contiene(&[
        "corregir",
        "cambiar mis datos",
        "cambiar mi nombre",
        "actualizar mis datos",
        "datos personales",
    ]) {
        Intencion::CorregirDatos
    } else if contiene(&[
        "cambiar mi celular",
        "cambiar el celular",
        "cambiar celular",
        "cambiar mi numero",
        "cambiar mi telefono",
        "cambiar de numero",
        "actualizar mi celular",
        "actualizar mi telefono",
        "nuevo celular",
    ]) {
        Intencion::CambiarCelular
    } else if contiene(&[
        "autoriz",
        "quien puede recoger",
        "que recoja",
        "otra persona recoja",
        "alguien recoja",
        "persona de confianza",
    ]) {
        Intencion::Personas
    } else if contiene(&["direccion", "direcciones", "domicilio"])
        && !contiene(&["donde", "ubicad", "tienda", "local", "taller", "correo"])
    {
        // «Mi dirección», «añadir una dirección», «cambiar mi dirección». Si pregunta DÓNDE queda la tienda, no es esto.
        Intencion::Direcciones
    } else if contiene(&["correo", "email", "e-mail", "mail"]) {
        // Va ANTES que "registr": "registrar mi correo" es agregar un correo, no empezar un registro.
        Intencion::AgregarCorreo
    } else if contiene(&["registr", "inscrib", "crear cuenta", "crear una cuenta"]) {
        Intencion::Registrarse
    } else if contiene(&["cancelar mi apartado", "cancelar el apartado", "cancelar apartado"]) {
        // Va ANTES que "apartar" y que las citas: cancelar un apartado no es apartar ni agendar.
        Intencion::CancelarApartado
    } else if contiene(&["mis apartado", "mi apartado", "ver apartado", "ver mis apartado"]) {
        Intencion::MiApartado
    } else if contiene(&["cancelar mi cita", "cancelar la cita", "cancelar cita", "cancelar mi orden"]) {
        // Va ANTES que "cita": "cancelar mi cita" no es agendar una.
        Intencion::CancelarCita
    } else if contiene(&[
        "seguimiento",
        "mi orden",
        "mis orden",
        "ver mi cita",
        "ver mis cita",
        "estado de mi",
        "como va mi",
    ]) {
        // Va ANTES que "cita" y "servicio". Ojo: no usamos "mi cita" a secas porque
        // "quiero reservar mi cita" es agendar.
        Intencion::MiOrden
    } else if contiene(&[
        "solicitar una reparacion",
        "solicitar reparacion",
        "solicitar un diagnostico",
        "quiero reparar",
        "necesito reparar",
        "reparar mi",
    ]) && !contiene(&["agendar", "reserv", "turno"])
        && !palabras(texto).iter().any(|p| p == "cita" || p == "citas")
    {
        // Va ANTES que las citas: «solicitar» lleva dentro la palabra «cita», y «Reparar mi equipo» (el botón del
        // lienzo) pide ver los servicios, no agendar. Si la persona dice «agendar» o «cita» por su cuenta, gana la cita.
        Intencion::Servicios
    } else if contiene(&["cita", "agendar", "reserv", "turno"]) {
        Intencion::AgendarCita
    } else if contiene(&["apartar", "separar"]) {
        // Va ANTES que "producto": "apartar este producto" es apartar, no mirar el catálogo.
        Intencion::Apartar
    } else if contiene(&[
        "producto",
        "catalogo",
        "comprar",
        "venden",
        "accesorio",
        "tienda",
        "stock",
    ]) {
        Intencion::Productos
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
    fn reconoce_ya_soy_cliente() {
        assert_eq!(detectar_intencion("Ya soy cliente"), Intencion::YaSoyCliente);
        assert_eq!(detectar_intencion("ya me registré antes"), Intencion::YaSoyCliente);
        // Las opciones que ofrece el chat:
        assert_eq!(detectar_intencion("Registrarme"), Intencion::Registrarse);
        assert_eq!(detectar_intencion("Solo estoy de paso"), Intencion::SoloDePaso);
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
    fn reconoce_seguimiento_y_cancelacion() {
        assert_eq!(detectar_intencion("Ver mi orden"), Intencion::MiOrden);
        assert_eq!(detectar_intencion("quiero ver el seguimiento"), Intencion::MiOrden);
        assert_eq!(detectar_intencion("¿Cómo va mi equipo? estado de mi laptop"), Intencion::MiOrden);
        assert_eq!(detectar_intencion("Cancelar mi cita"), Intencion::CancelarCita);
        // Agendar sigue siendo agendar, aunque diga "mi cita".
        assert_eq!(detectar_intencion("quiero reservar mi cita"), Intencion::AgendarCita);
        assert_eq!(detectar_intencion("Agendar una cita"), Intencion::AgendarCita);
    }

    #[test]
    fn reconoce_productos_y_apartados() {
        assert_eq!(detectar_intencion("Ver productos"), Intencion::Productos);
        assert_eq!(detectar_intencion("¿Qué venden?"), Intencion::Productos);
        assert_eq!(detectar_intencion("Ver otros productos"), Intencion::Productos);
        assert_eq!(detectar_intencion("Apartar este producto"), Intencion::Apartar);
        assert_eq!(detectar_intencion("quiero separar un mouse"), Intencion::Apartar);
        assert_eq!(detectar_intencion("Ver mis apartados"), Intencion::MiApartado);
        assert_eq!(detectar_intencion("Cancelar mi apartado"), Intencion::CancelarApartado);
        // Lo anterior no cambia: citas y servicios siguen siendo citas y servicios.
        assert_eq!(detectar_intencion("Cancelar mi cita"), Intencion::CancelarCita);
        assert_eq!(detectar_intencion("Ver servicios"), Intencion::Servicios);
        assert_eq!(detectar_intencion("¿Cuánto cuesta el formateo?"), Intencion::Servicios);
    }

    #[test]
    fn reparar_pide_servicios_y_no_una_cita() {
        // Las frases del lienzo (píldora y menú lateral).
        assert_eq!(detectar_intencion("Quiero reparar mi equipo"), Intencion::Servicios);
        assert_eq!(detectar_intencion("Quiero solicitar una reparación"), Intencion::Servicios);
        // Si pide la cita de forma explícita, sigue siendo una cita.
        assert_eq!(detectar_intencion("Quiero agendar una cita para reparar mi laptop"), Intencion::AgendarCita);
        assert_eq!(detectar_intencion("Quiero reparar mi laptop, ¿tienen cita?"), Intencion::AgendarCita);
    }

    #[test]
    fn reconoce_lo_de_mi_perfil() {
        // Los mensajes que envían los botones «… con Juno» de la página de perfil.
        assert_eq!(detectar_intencion("Quiero corregir mis datos"), Intencion::CorregirDatos);
        assert_eq!(detectar_intencion("Quiero cambiar mi celular"), Intencion::CambiarCelular);
        assert_eq!(detectar_intencion("Quiero cambiar mi dirección"), Intencion::Direcciones);
        assert_eq!(detectar_intencion("Quiero añadir una dirección de entrega"), Intencion::Direcciones);
        assert_eq!(detectar_intencion("Mis direcciones"), Intencion::Direcciones);
        assert_eq!(detectar_intencion("Quiero autorizar a alguien para recoger"), Intencion::Personas);
        assert_eq!(detectar_intencion("Quiero pedir mis datos"), Intencion::PedirDatos);
        assert_eq!(detectar_intencion("Quiero eliminar mi cuenta"), Intencion::EliminarCuenta);
        assert_eq!(detectar_intencion("Quiero eliminar mis datos"), Intencion::EliminarCuenta);
        // Lo de antes no cambia.
        assert_eq!(detectar_intencion("Quiero cambiar mi correo"), Intencion::AgregarCorreo);
        assert_eq!(detectar_intencion("¿Cuál es la dirección de la tienda?"), Intencion::Productos);
        assert_eq!(detectar_intencion("¿Dónde queda el taller? dirección"), Intencion::Desconocida);
        assert_eq!(detectar_intencion("Quiero registrarme"), Intencion::Registrarse);
    }

    #[test]
    fn el_lienzo_consulta_el_estado_del_equipo() {
        assert_eq!(detectar_intencion("Quiero consultar el estado de mi equipo"), Intencion::MiOrden);
        assert_eq!(detectar_intencion("Quiero consultar el estado de mi reparación"), Intencion::MiOrden);
        assert_eq!(detectar_intencion("Muéstrame el catálogo completo"), Intencion::Productos);
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
