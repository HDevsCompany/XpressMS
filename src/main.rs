// Punto de entrada de XpressMS: aquí arranca todo.

// Declaramos los módulos del proyecto. Cada "mod" busca su archivo (config.rs, db.rs...) dentro de src/.
mod config;
mod db;
mod error;
mod modulos;

use tower_http::services::ServeDir; // Entrega los archivos de la carpeta static/ (estilos, scripts, textos legales).
use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{Html, Response},
    routing::get,
    Router,
}; // Piezas de axum que vamos a usar.
use sqlx::PgPool; // El tipo del grupo de conexiones, para poder recibirlo en el manejador de /salud.

// Esta marca convierte "main" en una función asíncrona: crea el motor de tokio que ejecuta las tareas.
#[tokio::main]
async fn main() {
    // 1. Lee la configuración (DATABASE_URL y PORT) desde el .env.
    let config = config::Config::desde_entorno();

    // 1b. Avisa en la terminal cómo se enviarán los SMS (Textbee o simulado) y si falta algo de su configuración.
    modulos::comun::sms::informar_estado();

    // 1c. Lo mismo para la consulta de DNI/RUC (Factiliza o simulada).
    modulos::acceso::consulta_documento::informar_estado();

    // 2. Conecta con Neon. Si algo falla aquí, el programa se detiene con un mensaje claro.
    let pool = db::crear_pool(&config.database_url).await;

    // 3. Si se escribió un comando (por ejemplo "cargo run -- crear-admin"), se ejecuta y el programa termina
    //    sin abrir el servidor. Los comandos son para quien administra el sistema, no para el público.
    if let Some(comando) = std::env::args().nth(1) {
        if !modulos::acceso::personal::comandos::ejecutar(&pool, &comando).await {
            eprintln!("Comando desconocido: {}", comando);
            eprintln!("Comandos disponibles: crear-admin, restablecer-clave");
        }
        return;
    }

    // 4. Define las rutas de la aplicación.
    let app = Router::new()
        .route("/", get(lienzo))     // el lienzo de Juno: la pantalla principal (templates/lienzo.html)
        .route("/ingreso", get(ingreso)) // la tarjeta de ingreso (cliente o equipo); el lienzo la abre en una ventana
        .route("/servicios", get(servicios)) // servicios técnicos: la página solo muestra; Juno solicita y agenda
        .route("/catalogo", get(catalogo)) // la tienda: la página solo muestra; Juno (la burbuja flotante) filtra y aparta
        .route("/perfil", get(perfil)) // «Mi perfil y datos»: la página solo muestra; lo que cambia algo se lo pide a Juno
        .route("/chat", get(lienzo)) // la misma pantalla: a ella se llega al iniciar sesión o al registrarse con Juno
        .route("/salud", get(salud)) // cuando alguien abra /salud, se ejecuta la función "salud"
        .nest_service("/static", ServeDir::new("static")) // /static/css/..., /static/js/... salen de la carpeta static/
        .merge(modulos::publico::rutas::rutas()) // datos públicos del catálogo: /publico/inicio (el lienzo ya no los usa; quedan para una futura página de catálogo)
        .merge(modulos::perfil::rutas::rutas()) // los datos de «Mi perfil y datos»: /perfil/datos
        .merge(modulos::agente::rutas::rutas()) // suma las rutas del agente: /agente/iniciar y /agente/mensaje
        .merge(modulos::agente::rutas_panel::rutas()) // Juno en el panel de servicios: /panel/api/juno/servicios (solo Administrador)
        .merge(modulos::acceso::personal::rutas::rutas()) // ingreso del personal: /ingresar y /acceso/personal/...
        .merge(modulos::acceso::personal::rutas_gestion::rutas()) // administración del personal: /panel/api/personal
        .merge(modulos::panel::rutas::rutas()) // panel interno: /panel
        .merge(modulos::servicio_tecnico::rutas::rutas()) // administración de servicios y equipos: /panel/api/...
        .merge(modulos::servicio_tecnico::rutas_ordenes::rutas()) // órdenes de servicio: /panel/api/ordenes
        .merge(modulos::catalogo::rutas::rutas()) // productos, categorías y apartados: /panel/api/productos, ...
        .layer(middleware::from_fn(sin_cache)) // que el navegador revise static/ en cada visita (así ve los cambios al instante)
        .with_state(pool);           // comparte el pool con todas las rutas para que puedan consultar la base

    // 5. Arma la dirección donde escuchará el servidor. 0.0.0.0 significa "acepta conexiones desde cualquier lugar" (lo que pide Render).
    let direccion = format!("0.0.0.0:{}", config.port);

    // 6. Abre el puerto. Si está ocupado por otro programa, falla con este mensaje.
    let listener = tokio::net::TcpListener::bind(&direccion)
        .await
        .expect("No se pudo abrir el puerto (¿ya hay otro programa usándolo?)");

    // 7. Avisa en la terminal que todo está listo.
    println!("XpressMS escuchando en http://localhost:{}", config.port);

    // 8. Pone el servidor a atender peticiones. Esta línea no termina hasta que cierres el programa (Ctrl + C).
    axum::serve(listener, app)
        .await
        .expect("El servidor se detuvo por un error");
}

// Pide al navegador que, antes de usar un archivo de /static (estilos y scripts), pregunte al servidor si cambió.
// Si no cambió, el servidor responde "304" casi sin enviar nada; si cambió, entrega el nuevo.
// Sin esto, el navegador puede seguir usando un script viejo y no mostrar las funciones nuevas.
async fn sin_cache(peticion: Request, siguiente: Next) -> Response {
    let es_estatico = peticion.uri().path().starts_with("/static/");
    let mut respuesta = siguiente.run(peticion).await;
    if es_estatico {
        respuesta
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    }
    respuesta
}

// El encabezado y el menú lateral son los mismos en el lienzo y en «Mi perfil y datos»: viven en un solo archivo
// (templates/parciales/encabezado_menu.html) y se meten en cada página en lugar de la marca «<!--ENCABEZADO_MENU-->».
const ENCABEZADO_MENU: &str = include_str!("../templates/parciales/encabezado_menu.html");
const MARCA_ENCABEZADO_MENU: &str = "<!--ENCABEZADO_MENU-->";

fn con_encabezado_y_menu(pagina: &str) -> Html<String> {
    Html(pagina.replace(MARCA_ENCABEZADO_MENU, ENCABEZADO_MENU))
}

// Manejador de GET / y GET /chat: entrega el lienzo de Juno (templates/lienzo.html). Es una sola pantalla: Juno al centro
// para empezar y, en cuanto la persona escribe, la conversación; con el menú lateral y, arriba, «Iniciar sesión» o el avatar.
// "include_str!" mete el contenido del archivo dentro del programa al compilar.
// Por eso, si editas una plantilla, hay que reiniciar "cargo run" para ver el cambio.
async fn lienzo() -> Html<String> {
    con_encabezado_y_menu(include_str!("../templates/lienzo.html"))
}

// Manejador de GET /catalogo: la tienda (templates/catalogo.html). Página propia, pública (la ve cualquiera): la cuadrícula de
// productos la pide a /publico/catalogo y Juno vive como una burbuja flotante que conversa sin salir de la página.
async fn catalogo() -> Html<String> {
    con_encabezado_y_menu(include_str!("../templates/catalogo.html"))
}

// Manejador de GET /servicios: los servicios técnicos (templates/servicios.html). Página propia y pública, como /catalogo: las tarjetas
// las pide a /publico/servicios y Juno vive como una burbuja flotante que solicita el servicio o agenda la cita de diagnóstico.
async fn servicios() -> Html<String> {
    con_encabezado_y_menu(include_str!("../templates/servicios.html"))
}

// Manejador de GET /perfil: «Mi perfil y datos» (templates/perfil.html). La página pide sus datos a /perfil/datos con el
// token del navegador; quien no es un cliente reconocido ve una invitación a iniciar sesión.
async fn perfil() -> Html<String> {
    con_encabezado_y_menu(include_str!("../templates/perfil.html"))
}

// Manejador de GET /ingreso: entrega la tarjeta de ingreso (templates/acceso/ingresar.html), la misma que sirve
// /ingresar; la diferencia es que aquí no se redirige a nadie al panel y se ve primero el lado de cliente.
// El lienzo la muestra dentro de una ventana (un marco) cuando se pulsa «Iniciar sesión».
async fn ingreso() -> Html<&'static str> {
    Html(include_str!("../templates/acceso/ingresar.html"))
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
