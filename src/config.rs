// Este archivo se encarga de leer la configuración de la aplicación desde el archivo .env.

use std::env; // Trae el módulo "env" de la librería estándar, que permite leer variables de entorno.

// Una "struct" es un grupo de datos con nombre. Aquí agrupamos todo lo que la app necesita configurar.
pub struct Config {
    pub database_url: String, // La cadena de conexión a Neon (la que empieza con postgresql://).
    pub port: u16,            // El puerto donde escuchará el servidor (u16 = número entero de 0 a 65535).
}

// "impl" es donde se definen las funciones que pertenecen a la struct Config.
impl Config {
    // Esta función lee el .env y devuelve una Config lista para usar.
    pub fn desde_entorno() -> Config {
        // Carga el archivo .env y pone sus líneas como variables de entorno.
        // ".ok()" ignora el error si el archivo no existe (en Render las variables se configuran en su panel, no en un .env).
        dotenvy::dotenv().ok();

        // Lee la variable DATABASE_URL. Si no existe, "expect" detiene el programa y muestra este mensaje.
        // Es mejor que falle aquí, al arrancar, y no más adelante con un error confuso.
        let database_url = env::var("DATABASE_URL")
            .expect("Falta DATABASE_URL en el archivo .env");

        // Lee la variable PORT. Se hace en cadena:
        let port = env::var("PORT") // intenta leer PORT (devuelve un Result)
            .ok()                    // lo convierte en Option: Some(texto) si existe, None si no
            .and_then(|p| p.parse().ok()) // si existe, intenta convertir el texto a número; si falla, None
            .unwrap_or(3000);        // si no hay PORT o no es un número válido, usa 3000 por defecto

        // Devuelve la Config con los dos valores. Al final no lleva ";" porque es el valor que devuelve la función.
        Config { database_url, port }
    }
}
