// comandos.rs: comandos que se ejecutan desde la terminal, sin abrir el servidor.
//
//   cargo run -- crear-admin         crea al PRIMER administrador (pregunta sus datos y entrega una clave temporal)
//   cargo run -- restablecer-clave   entrega una clave temporal nueva a quien olvidó la suya
//
// Son la "llave maestra" de la plataforma: solo funcionan con acceso a la base (el .env), o sea, para
// quien administra el sistema. En Render no se usan; se corren desde tu computadora, que apunta a la misma base.

use std::io::{self, Write};

use sqlx::PgPool;

use super::modelo::{NuevoPersonal, ResultadoPersonal};
use super::roles::ROL_ADMIN;
use super::{claves, repositorio};
use crate::modulos::comun::validaciones::{
    limpiar_nombre, normalizar_usuario, validar_documento, TIPO_DNI,
};

// Ejecuta el comando pedido. Devuelve false si no existe.
pub async fn ejecutar(pool: &PgPool, comando: &str) -> bool {
    match comando {
        "crear-admin" => crear_admin(pool).await,
        "restablecer-clave" => restablecer_clave(pool).await,
        _ => return false,
    }
    true
}

// Hace una pregunta en la terminal y devuelve lo que se escribió (sin espacios sobrantes).
fn preguntar(texto: &str) -> String {
    print!("{}", texto);
    io::stdout().flush().ok(); // para que la pregunta aparezca antes de esperar la respuesta
    let mut linea = String::new();
    io::stdin().read_line(&mut linea).ok();
    linea.trim().to_string()
}

// Repite la pregunta hasta que la respuesta sea válida. "revisar" devuelve el valor limpio o un mensaje de error.
fn preguntar_valido<T>(texto: &str, revisar: impl Fn(&str) -> Result<T, String>) -> T {
    loop {
        match revisar(&preguntar(texto)) {
            Ok(valor) => return valor,
            Err(mensaje) => println!("  {}", mensaje),
        }
    }
}

async fn crear_admin(pool: &PgPool) {
    match repositorio::existe_administrador(pool).await {
        Ok(true) => {
            println!("Ya existe un administrador. Este comando solo sirve para crear el primero.");
            println!("Si olvidó su clave, usa:  cargo run -- restablecer-clave");
            return;
        }
        Ok(false) => {}
        Err(error) => {
            println!("No se pudo consultar la base de datos: {}", error);
            println!("¿Ya ejecutaste 04_acceso_personal.sql en Neon?");
            return;
        }
    }

    println!("Crear el primer administrador de XpressMS\n");
    let numero_documento = preguntar_valido("DNI (8 números): ", |texto| {
        validar_documento(TIPO_DNI, texto)
    });
    let paterno = preguntar_valido("Apellido paterno: ", |texto| {
        limpiar_nombre(texto).ok_or("Escribe solo letras.".to_string())
    });
    let materno = match preguntar("Apellido materno (Enter si no tiene): ").as_str() {
        "" => None,
        texto => limpiar_nombre(texto),
    };
    let nombres = preguntar_valido("Nombres: ", |texto| {
        limpiar_nombre(texto).ok_or("Escribe solo letras.".to_string())
    });
    let usuario = preguntar_valido("Nombre de usuario (4 a 30 caracteres, sin espacios, ej. pedro.guevara): ", |texto| {
        normalizar_usuario(texto).ok_or(
            "Usa de 4 a 30 letras sin tilde, números, punto, guion o guion bajo, empezando con letra.".to_string(),
        )
    });

    let clave = claves::generar_clave_temporal();
    let clave_hash = match claves::hashear_async(clave.clone()).await {
        Ok(hash) => hash,
        Err(mensaje) => {
            println!("{}", mensaje);
            return;
        }
    };

    let nuevo = NuevoPersonal {
        tipo_documento: TIPO_DNI.to_string(),
        numero_documento,
        paterno,
        materno,
        nombres,
        telefono: None,
        correo: None,
        usuario: usuario.clone(),
        clave_hash,
        roles: vec![ROL_ADMIN.to_string()],
    };
    match repositorio::crear_personal(pool, &nuevo).await {
        Ok(ResultadoPersonal::Hecho) => {
            println!("\nListo. Administrador creado.");
            println!("  Usuario:        {}", usuario);
            println!("  Clave temporal: {}", clave);
            println!("Anótala ahora: no se vuelve a mostrar. Se te pedirá cambiarla al ingresar en /ingresar.");
        }
        Ok(ResultadoPersonal::DocumentoRepetido) => println!("\nYa existe una persona activa con ese documento."),
        Ok(ResultadoPersonal::UsuarioRepetido) => println!("\nYa existe una cuenta con ese nombre de usuario."),
        Ok(_) => println!("\nNo se pudo crear la cuenta."),
        Err(error) => {
            println!("\nNo se pudo crear: {}", error);
            println!("¿Ya ejecutaste los scripts 04 y 06 en Neon?");
        }
    }
}

async fn restablecer_clave(pool: &PgPool) {
    let usuario = preguntar_valido("Nombre de usuario a restablecer: ", |texto| {
        normalizar_usuario(texto).ok_or("Ese nombre de usuario no tiene un formato válido.".to_string())
    });
    let clave = claves::generar_clave_temporal();
    let clave_hash = match claves::hashear_async(clave.clone()).await {
        Ok(hash) => hash,
        Err(mensaje) => {
            println!("{}", mensaje);
            return;
        }
    };
    match repositorio::restablecer_clave(pool, &usuario, &clave_hash).await {
        Ok(true) => {
            println!("\nListo. Se cerraron sus sesiones y se quitó cualquier bloqueo.");
            println!("  Clave temporal: {}", clave);
            println!("Se le pedirá cambiarla al ingresar.");
        }
        Ok(false) => println!("No existe un usuario llamado \"{}\".", usuario),
        Err(error) => println!("No se pudo restablecer: {}", error),
    }
}
