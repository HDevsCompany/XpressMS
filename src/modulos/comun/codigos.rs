use uuid::Uuid; // Uuid::new_v4() crea 16 bytes aleatorios.

// Los 32 símbolos permitidos: letras y números, SIN I, O, 0 y 1 (se confunden al leer o dictar).
// Son exactamente 32 porque 256 (valores de un byte) se divide justo entre 32: todos los
// símbolos tienen la misma probabilidad de salir (no hay "favoritos").
const ALFABETO: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

// Posiciones de los bytes que usaremos para armar el código. Se saltan la 6 y la 8 porque en un
// UUID esos bytes llevan bits fijos (marcan la versión) y no son completamente aleatorios.
const POSICIONES: [usize; 9] = [0, 1, 2, 3, 4, 5, 7, 9, 10];

// Crea un código nuevo de 9 caracteres, por ejemplo "K7TQ3M9XA".
// Lo usarán todas las tablas cuyo Codigo es char(9): Clientes, Conversaciones, Mensajes...
// Si dos códigos coincidieran (probabilidad ínfima: 1 en 35 billones), la clave primaria de la
// base rechazaría el duplicado y bastaría con reintentar.
pub fn generar_codigo() -> String {
    let aleatorio = Uuid::new_v4(); // 16 bytes aleatorios
    codigo_desde_bytes(aleatorio.as_bytes())
}

// Crea el token secreto de una conversación: 32 caracteres hexadecimales (a-f, 0-9).
// Se guarda en el navegador de la persona para reconocer su conversación. Nadie debe poder adivinarlo.
pub fn generar_token() -> String {
    Uuid::new_v4().simple().to_string() // "simple" = el UUID sin guiones (32 caracteres)
}

// Convierte bytes en un código usando el ALFABETO. Está separada de generar_codigo() para poder
// probarla con bytes conocidos (los aleatorios no se pueden predecir en una prueba).
// "% 32" convierte cada byte (0-255) en una posición válida del alfabeto (0-31).
fn codigo_desde_bytes(bytes: &[u8; 16]) -> String {
    POSICIONES
        .iter() // recorre las 9 posiciones elegidas
        .map(|&posicion| ALFABETO[(bytes[posicion] % 32) as usize] as char) // byte -> símbolo
        .collect() // junta los 9 símbolos en un texto
}


