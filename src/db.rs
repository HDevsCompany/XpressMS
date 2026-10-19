// Este archivo crea la conexión a la base de datos de Neon.

use sqlx::{postgres::PgPoolOptions, PgPool}; // PgPool = grupo de conexiones a PostgreSQL; PgPoolOptions = para configurarlo.

// "async" significa que la función puede esperar (por ejemplo, la red) sin bloquear el programa.
// Recibe la cadena de conexión como texto prestado (&str) y devuelve el grupo de conexiones.
pub async fn crear_pool(database_url: &str) -> PgPool {
    // Un "pool" es un grupo de conexiones abiertas que se reutilizan, en lugar de abrir una nueva en cada consulta.
    PgPoolOptions::new()           // empieza a configurar un pool nuevo
        .max_connections(5)        // como máximo 5 conexiones a la vez (suficiente para empezar y cuida el plan gratuito)
        .connect(database_url)     // conecta a Neon usando la cadena del .env
        .await                     // espera a que termine de conectar (la primera vez Neon puede tardar unos segundos en despertar)
        .expect("No se pudo conectar a la base de datos de Neon") // si falla, detiene el programa con este mensaje
}
