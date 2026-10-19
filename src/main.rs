// Punto de entrada de XpressMS: aquí arranca todo.

// Declaramos los módulos del proyecto. Cada "mod" busca su archivo (config.rs, db.rs...) dentro de src/.
mod config;
mod db;
mod error;
mod modulos;

use axum::{extract::State, http::StatusCode, response::Html, routing::get, Router}; // Piezas de axum que vamos a usar.
use sqlx::PgPool; // El tipo del grupo de conexiones, para poder recibirlo en el manejador de /salud.

// Esta marca convierte "main" en una función asíncrona: crea el motor de tokio que ejecuta las tareas.
#[tokio::main]
async fn main() {
    // 1. Lee la configuración (DATABASE_URL y PORT) desde el .env.
    let config = config::Config::desde_entorno();

    // 2. Conecta con Neon. Si algo falla aquí, el programa se detiene con un mensaje claro.
    let pool = db::crear_pool(&config.database_url).await;

    // 3. Define las rutas de la aplicación.
    let app = Router::new()
        .route("/", get(inicio))     // la página principal, donde aparece el agente
        .route("/salud", get(salud)) // cuando alguien abra /salud, se ejecuta la función "salud"
        .merge(modulos::agente::rutas::rutas()) // suma las rutas del agente: /agente/saludo y /agente/mensaje
        .with_state(pool);           // comparte el pool con todas las rutas para que puedan consultar la base

    // 4. Arma la dirección donde escuchará el servidor. 0.0.0.0 significa "acepta conexiones desde cualquier lugar" (lo que pide Render).
    let direccion = format!("0.0.0.0:{}", config.port);

    // 5. Abre el puerto. Si está ocupado por otro programa, falla con este mensaje.
    let listener = tokio::net::TcpListener::bind(&direccion)
        .await
        .expect("No se pudo abrir el puerto (¿ya hay otro programa usándolo?)");

    // 6. Avisa en la terminal que todo está listo.
    println!("XpressMS escuchando en http://localhost:{}", config.port);

    // 7. Pone el servidor a atender peticiones. Esta línea no termina hasta que cierres el programa (Ctrl + C).
    axum::serve(listener, app)
        .await
        .expect("El servidor se detuvo por un error");
}

// Manejador de GET /: entrega la página principal (templates/inicio.html).
// "include_str!" mete el contenido del archivo dentro del programa al compilar.
// Por eso, si editas inicio.html, hay que reiniciar "cargo run" para ver el cambio.
async fn inicio() -> Html<&'static str> {
    Html(include_str!("../templates/inicio.html"))
}

// Manejador de GET /salud: comprueba que la aplicación llega a la base de datos.
// Recibe el pool compartido con "State(pool)" y devuelve un código HTTP más un texto.
// Más adelante lo moveremos a su propio módulo; por ahora vive aquí para probar la conexión.
async fn salud(State(pool): State<PgPool>) -> (StatusCode, String) {
    // Pregunta a la base de datos cuántas filas tiene la tabla de tipos de documento.
    // query_scalar devuelve un solo valor; i64 es el tipo del resultado de count(*).
    let resultado = sqlx::query_scalar::<_, i64>("select count(*) from Comun.TiposDocumento")
        .fetch_one(&pool) // ejecuta la consulta y espera exactamente una fila
        .await;           // espera la respuesta de Neon

    // "match" revisa si salió bien (Ok) o mal (Err) y decide qué responder.
    match resultado {
        // Si salió bien: código 200 y un mensaje con el número de filas (debe ser 4).
        Ok(cantidad) => (
            StatusCode::OK,
            format!("OK: hay {} tipos de documento en la base de datos", cantidad),
        ),
        // Si salió mal: código 500 y el texto del error, para que sepas qué pasó.
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error al consultar la base de datos: {}", error),
        ),
    }
}
