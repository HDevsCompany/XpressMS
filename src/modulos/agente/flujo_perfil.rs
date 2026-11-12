// flujo_perfil.rs: la parte del agente que cambia lo que muestra «Mi perfil y datos»: las direcciones de entrega y
// recojo y las personas autorizadas a recoger los equipos. La página de perfil solo MUESTRA; aquí es donde se EJECUTA.
//
//   libre -> "mis direcciones" -> dir_accion -> dir_tipo -> [dir_nombre] -> dir_texto -> dir_referencia -> [dir_principal] -> libre
//                              -> dir_accion -> dir_cual -> libre                      (hacer principal)
//                              -> dir_accion -> dir_cual -> dir_quitar_confirmar -> libre   (quitar)
//   libre -> "autorizar a alguien" -> per_accion -> per_nombre -> per_documento -> per_parentesco -> per_confirmar
//                                  -> per_codigo (código SMS al celular del cliente) -> libre (persona autorizada)
//                                  -> per_accion -> per_quitar_cual -> per_quitar_confirmar -> libre
//   libre -> «cambiar mi celular», «corregir mis datos», «pedir mis datos», «eliminar mi cuenta»: respuestas informativas
//            (esos cambios se hacen en tienda con el documento; ver los textos abajo).
//
// Seguridad: solo un cliente reconocido llega aquí; autorizar a una persona exige un código que se envía por SMS al
// celular ya registrado del cliente (el mismo sistema de verificación del registro); el documento de la persona se
// guarda completo pero solo se muestra enmascarado. Regla: este archivo NO escribe SQL; usa perfil/repositorio.rs.

use sqlx::PgPool;

use super::modelo::{Conversacion, Datos};
use super::motor::{elegir_opcion, es_afirmativo, es_negativo, normalizar};
use super::pasos::{
    DIR_ACCION, DIR_CUAL, DIR_NOMBRE, DIR_PRINCIPAL, DIR_QUITAR_CONFIRMAR, DIR_REFERENCIA, DIR_TEXTO, DIR_TIPO,
    LIBRE, PER_ACCION, PER_CODIGO, PER_CONFIRMAR, PER_DOCUMENTO, PER_NOMBRE, PER_PARENTESCO, PER_QUITAR_CONFIRMAR,
    PER_QUITAR_CUAL,
};
use super::respuesta::{r, Paso, Respuesta, MENU};
use crate::modulos::acceso::repositorio as acceso_repo;
use crate::modulos::acceso::verificacion_telefono::{self, Resultado as ResultadoCodigo, Solicitud};
use crate::modulos::acceso::codigo_otp;
use crate::modulos::comun::{formato, validaciones};
use crate::modulos::perfil::modelo::{Direccion, Persona};
use crate::modulos::perfil::repositorio::{self as repo, ResultadoAutorizacion};
use crate::modulos::perfil::rutas::{MAX_DIRECCIONES, MAX_PERSONAS};

const CANCELAR: &str = "Cancelar";
const VOLVER: &str = "Volver";
const OPCIONES_ANONIMO: [&str; 3] = ["Ya soy cliente", "Registrarme", "Solo estoy de paso"];

const ANADIR_DIRECCION: &str = "Añadir dirección";
const HACER_PRINCIPAL: &str = "Hacer principal";
const QUITAR_DIRECCION: &str = "Quitar dirección";
const TIPOS_DIRECCION: [&str; 4] = ["Casa", "Oficina", "Otra", CANCELAR];
const OPCIONES_PRINCIPAL: [&str; 2] = ["Sí, principal", "No, solo guardarla"];
const OPCIONES_REFERENCIA: [&str; 2] = ["Omitir", CANCELAR];
const OPCIONES_QUITAR: [&str; 2] = ["Sí, quitar", "No, mantenerla"];

const AUTORIZAR_PERSONA: &str = "Autorizar a alguien";
const QUITAR_PERSONA: &str = "Quitar a alguien";
const PARENTESCOS: [&str; 7] = ["Madre", "Padre", "Pareja", "Hermano/a", "Hijo/a", "Amistad", "Otro"];
const OPCIONES_CONFIRMAR_PERSONA: [&str; 2] = ["Enviar código", CANCELAR];
const OPCIONES_CODIGO: [&str; 2] = ["Reenviar código", CANCELAR];

const MAX_NOMBRE_DIRECCION: usize = 40;
const MIN_DIRECCION: usize = 8;
const MAX_DIRECCION: usize = 120;
const MAX_PARENTESCO: usize = 30;

// ---------- Piezas de apoyo ----------

// Una respuesta cuyos botones se calcularon en el momento.
fn con_botones(texto: &str, botones: Vec<String>) -> Respuesta {
    let referencias: Vec<&str> = botones.iter().map(|b| b.as_str()).collect();
    r(texto, &referencias)
}

fn a_botones(opciones: &[&str]) -> Vec<String> {
    opciones.iter().map(|o| o.to_string()).collect()
}

// ¿La persona quiere dejar el trámite? Solo con palabras claras (las respuestas libres, como una dirección, nunca se
// toman por «no»).
fn es_cancelar(texto: &str) -> bool {
    matches!(normalizar(texto).trim(), "cancelar" | "volver" | "salir")
}

// Quien no está reconocido no puede tocar direcciones ni personas autorizadas.
fn pedir_identificarse() -> Paso {
    (
        r(
            "Para eso necesito saber quién eres. ¿Ya eres cliente o quieres registrarte?",
            &OPCIONES_ANONIMO,
        ),
        LIBRE,
    )
}

// Olvida todo lo recogido para direcciones y personas.
fn limpiar(datos: &mut Datos) {
    datos.dir_accion = None;
    datos.dir_nombre = None;
    datos.dir_texto = None;
    datos.dir_referencia = None;
    datos.dir_objetivo = None;
    datos.per_nombre = None;
    datos.per_documento = None;
    datos.per_parentesco = None;
    datos.per_objetivo = None;
}

fn salir(datos: &mut Datos) -> Paso {
    limpiar(datos);
    (r("Sin problema. ¿En qué te puedo ayudar?", &MENU), LIBRE)
}

// Los nombres como botones. Si dos se llaman igual, el segundo lleva (2), el tercero (3)...
fn etiquetas(nombres: &[String]) -> Vec<String> {
    let mut vistas: Vec<String> = Vec::new();
    nombres
        .iter()
        .map(|nombre| {
            let repetidas = vistas.iter().filter(|v| *v == nombre).count();
            vistas.push(nombre.clone());
            if repetidas == 0 {
                nombre.clone()
            } else {
                format!("{} ({})", nombre, repetidas + 1)
            }
        })
        .collect()
}

// Texto de la persona escrito con comodidad: espacios repetidos fuera.
fn compactar(texto: &str) -> String {
    texto.split_whitespace().collect::<Vec<_>>().join(" ")
}

// Un texto libre corto y sin caracteres raros (nombres de direcciones, referencias).
fn texto_valido(texto: &str, minimo: usize, maximo: usize) -> Option<String> {
    let limpio = compactar(texto);
    let largo = limpio.chars().count();
    if largo >= minimo && largo <= maximo && !limpio.chars().any(|c| c.is_control()) {
        Some(limpio)
    } else {
        None
    }
}

// ---------- Respuestas informativas ----------
// Lo que todavía no se hace por chat (se cambia en la tienda con el documento): se dice con claridad, sin prometer más.

pub fn cambiar_celular(conversacion: &Conversacion) -> Paso {
    if conversacion.cliente.is_none() {
        return pedir_identificarse();
    }
    (
        r(
            "Por seguridad, el celular de tu cuenta se cambia en la tienda, presentando tu documento: así nadie más puede quedarse con tu cuenta. Mientras tanto, tu número actual sigue recibiendo los avisos de tus equipos.",
            &MENU,
        ),
        LIBRE,
    )
}

pub fn corregir_datos(conversacion: &Conversacion) -> Paso {
    if conversacion.cliente.is_none() {
        return pedir_identificarse();
    }
    (
        r(
            "Tu nombre, documento y fecha de nacimiento vienen de tu documento de identidad, por eso no se cambian por chat. Si algo no coincide, acércate a la tienda con tu documento y lo corregimos. Lo que sí puedo hacer ahora: actualizar tus direcciones y las personas autorizadas a recoger tus equipos.",
            &["Mis direcciones", "Autorizar a una persona"],
        ),
        LIBRE,
    )
}

pub fn pedir_datos(conversacion: &Conversacion) -> Paso {
    if conversacion.cliente.is_none() {
        return pedir_identificarse();
    }
    (
        r(
            "Todo lo que guardamos de ti lo ves en «Mi perfil y datos», en el menú. Si necesitas una copia completa por escrito, pídela en la tienda con tu documento y el equipo te la entrega.",
            &MENU,
        ),
        LIBRE,
    )
}

pub fn eliminar_cuenta(conversacion: &Conversacion) -> Paso {
    if conversacion.cliente.is_none() {
        return pedir_identificarse();
    }
    (
        r(
            "Entiendo. Por seguridad, la eliminación de una cuenta se solicita en la tienda con tu documento: el equipo revisa que no tengas citas ni apartados pendientes y la procesa. Hasta entonces tu cuenta sigue activa y tus datos protegidos.",
            &MENU,
        ),
        LIBRE,
    )
}

// ---------- Direcciones ----------

fn resumen_direcciones(lista: &[Direccion]) -> String {
    lista
        .iter()
        .map(|d| format!("• {}{}: {}", d.nombre, if d.principal { " (principal)" } else { "" }, d.direccion))
        .collect::<Vec<_>>()
        .join("\n")
}

// El menú de direcciones: lo que se puede hacer según cuántas tiene.
fn menu_direcciones(lista: &[Direccion], aviso: Option<&str>) -> Paso {
    let mut botones = Vec::new();
    if lista.len() < MAX_DIRECCIONES {
        botones.push(ANADIR_DIRECCION.to_string());
    }
    if lista.len() > 1 {
        botones.push(HACER_PRINCIPAL.to_string());
    }
    if !lista.is_empty() {
        botones.push(QUITAR_DIRECCION.to_string());
    }
    botones.push(VOLVER.to_string());

    let mut texto = String::new();
    if let Some(aviso) = aviso {
        texto.push_str(aviso);
        texto.push_str("\n\n");
    }
    if lista.is_empty() {
        texto.push_str("Todavía no tienes direcciones guardadas. ¿Quieres añadir una?");
    } else {
        texto.push_str(&format!(
            "Estas son tus direcciones de entrega y recojo:\n{}\n\n¿Qué quieres hacer?",
            resumen_direcciones(lista)
        ));
    }
    (con_botones(&texto, botones), DIR_ACCION)
}

// «Mis direcciones», «cambiar mi dirección»...
pub async fn iniciar_direcciones(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    limpiar(datos);
    repo::asegurar_direccion_inicial(pool, cliente).await?;
    let lista = repo::listar_direcciones(pool, cliente).await?;
    Ok(menu_direcciones(&lista, None))
}

// Elige qué hacer: añadir, hacer principal o quitar.
pub async fn paso_dir_accion(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    if es_cancelar(texto) {
        return Ok(salir(datos));
    }
    let lista = repo::listar_direcciones(pool, cliente).await?;
    let t = normalizar(texto);

    if t.contains("anadir") || t.contains("agregar") || t.contains("nueva") {
        if lista.len() >= MAX_DIRECCIONES {
            let aviso = format!("Ya tienes {} direcciones, que es el máximo. Quita una para añadir otra.", MAX_DIRECCIONES);
            return Ok(menu_direcciones(&lista, Some(&aviso)));
        }
        return Ok((con_botones("¿Cómo la llamamos?", a_botones(&TIPOS_DIRECCION)), DIR_TIPO));
    }
    if t.contains("principal") && lista.len() > 1 {
        datos.dir_accion = Some("principal".to_string());
        return Ok(preguntar_cual_direccion(&lista, "principal"));
    }
    if t.contains("quitar") || t.contains("eliminar") || t.contains("borrar") {
        if lista.is_empty() {
            return Ok(menu_direcciones(&lista, None));
        }
        datos.dir_accion = Some("quitar".to_string());
        return Ok(preguntar_cual_direccion(&lista, "quitar"));
    }
    Ok(menu_direcciones(&lista, Some("No entendí cuál de esas opciones quieres. Elige una:")))
}

// Las direcciones entre las que se puede elegir: para «principal», solo las que todavía no lo son.
fn candidatas<'a>(lista: &'a [Direccion], accion: &str) -> Vec<&'a Direccion> {
    lista.iter().filter(|d| accion != "principal" || !d.principal).collect()
}

fn preguntar_cual_direccion(lista: &[Direccion], accion: &str) -> Paso {
    let nombres: Vec<String> = candidatas(lista, accion).iter().map(|d| d.nombre.clone()).collect();
    let mut botones = etiquetas(&nombres);
    botones.push(VOLVER.to_string());
    let pregunta = if accion == "principal" {
        "¿Cuál dejo como tu dirección principal?"
    } else {
        "¿Cuál dirección quitas?"
    };
    (con_botones(pregunta, botones), DIR_CUAL)
}

// Eligió cuál (para hacerla principal o para quitarla).
pub async fn paso_dir_cual(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    if es_cancelar(texto) {
        let lista = repo::listar_direcciones(pool, cliente).await?;
        limpiar(datos);
        return Ok(menu_direcciones(&lista, None));
    }

    let accion = datos.dir_accion.clone().unwrap_or_default();
    let lista = repo::listar_direcciones(pool, cliente).await?;
    let elegibles = candidatas(&lista, &accion);
    let nombres: Vec<String> = elegibles.iter().map(|d| d.nombre.clone()).collect();
    let botones = etiquetas(&nombres);
    let referencias: Vec<&str> = botones.iter().map(|b| b.as_str()).collect();

    let Some(posicion) = elegir_opcion(&referencias, texto) else {
        return Ok(preguntar_cual_direccion(&lista, &accion));
    };
    let elegida = elegibles[posicion];

    if accion == "principal" {
        let hecho = repo::hacer_principal(pool, cliente, &elegida.codigo).await?;
        limpiar(datos);
        let texto = if hecho {
            format!("Listo, «{}» ahora es tu dirección principal. Puedes verla en «Mi perfil y datos».", elegida.nombre)
        } else {
            "No pude cambiarla porque esa dirección ya no existe. Revisa tu lista.".to_string()
        };
        return Ok((r(&texto, &MENU), LIBRE));
    }

    datos.dir_objetivo = Some(elegida.codigo.clone());
    Ok((
        r(
            &format!("¿Quito «{}» ({})?", elegida.nombre, elegida.direccion),
            &OPCIONES_QUITAR,
        ),
        DIR_QUITAR_CONFIRMAR,
    ))
}

pub async fn paso_dir_quitar_confirmar(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    let objetivo = datos.dir_objetivo.clone().unwrap_or_default();

    if es_negativo(texto) || es_cancelar(texto) {
        limpiar(datos);
        return Ok((r("Perfecto, la dejo como está. ¿En qué más te ayudo?", &MENU), LIBRE));
    }
    if !es_afirmativo(texto) {
        return Ok((r("¿Quito esa dirección? Responde «Sí, quitar» o «No, mantenerla».", &OPCIONES_QUITAR), DIR_QUITAR_CONFIRMAR));
    }

    let quitada = repo::quitar_direccion(pool, cliente, &objetivo).await?;
    limpiar(datos);
    let mensaje = if quitada {
        "Listo, quité esa dirección. Si era la principal, otra de tus direcciones pasó a serlo."
    } else {
        "Esa dirección ya no estaba en tu lista."
    };
    Ok((r(mensaje, &MENU), LIBRE))
}

// Añadir: ¿Casa, Oficina u otra?
pub fn paso_dir_tipo(datos: &mut Datos, texto: &str) -> Paso {
    if es_cancelar(texto) {
        return salir(datos);
    }
    match elegir_opcion(&TIPOS_DIRECCION[..3], texto) {
        Some(0) | Some(1) => {
            datos.dir_nombre = Some(TIPOS_DIRECCION[elegir_opcion(&TIPOS_DIRECCION[..3], texto).unwrap_or(0)].to_string());
            pedir_direccion()
        }
        Some(_) => (
            r("¿Qué nombre le ponemos? Por ejemplo: «Casa de mi mamá» o «Taller».", &[CANCELAR]),
            DIR_NOMBRE,
        ),
        None => (con_botones("Elige cómo la llamamos:", a_botones(&TIPOS_DIRECCION)), DIR_TIPO),
    }
}

fn pedir_direccion() -> Paso {
    (
        r(
            "Escribe la dirección completa: calle, número y distrito. Por ejemplo: Av. España 1020, Trujillo.",
            &[CANCELAR],
        ),
        DIR_TEXTO,
    )
}

pub fn paso_dir_nombre(datos: &mut Datos, texto: &str) -> Paso {
    if es_cancelar(texto) {
        return salir(datos);
    }
    match texto_valido(texto, 2, MAX_NOMBRE_DIRECCION) {
        Some(nombre) => {
            datos.dir_nombre = Some(nombre);
            pedir_direccion()
        }
        None => (
            r(
                &format!("El nombre debe tener entre 2 y {} caracteres. Inténtalo de nuevo.", MAX_NOMBRE_DIRECCION),
                &[CANCELAR],
            ),
            DIR_NOMBRE,
        ),
    }
}

pub fn paso_dir_texto(datos: &mut Datos, texto: &str) -> Paso {
    if es_cancelar(texto) {
        return salir(datos);
    }
    match texto_valido(texto, MIN_DIRECCION, MAX_DIRECCION) {
        Some(direccion) => {
            datos.dir_texto = Some(direccion);
            (
                r(
                    "¿Quieres añadir una referencia para ubicarla? Por ejemplo: «frente al parque principal». Es opcional.",
                    &OPCIONES_REFERENCIA,
                ),
                DIR_REFERENCIA,
            )
        }
        None => (
            r(
                &format!(
                    "Escribe la dirección completa, de {} a {} caracteres (calle, número y distrito).",
                    MIN_DIRECCION, MAX_DIRECCION
                ),
                &[CANCELAR],
            ),
            DIR_TEXTO,
        ),
    }
}

pub async fn paso_dir_referencia(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    if es_cancelar(texto) {
        return Ok(salir(datos));
    }
    if normalizar(texto).trim() == "omitir" {
        datos.dir_referencia = None;
    } else {
        match texto_valido(texto, 2, MAX_DIRECCION) {
            Some(referencia) => datos.dir_referencia = Some(referencia),
            None => {
                return Ok((
                    r("La referencia puede tener hasta 120 caracteres. Escríbela de nuevo o pulsa «Omitir».", &OPCIONES_REFERENCIA),
                    DIR_REFERENCIA,
                ))
            }
        }
    }

    // Si ya tiene otras direcciones, pregunta si esta pasa a ser la principal; si es la primera, lo es.
    let existentes = repo::listar_direcciones(pool, cliente).await?;
    if existentes.is_empty() {
        return guardar_direccion(pool, cliente, datos, true).await;
    }
    Ok((r("¿La dejo como tu dirección principal?", &OPCIONES_PRINCIPAL), DIR_PRINCIPAL))
}

pub async fn paso_dir_principal(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    if es_cancelar(texto) {
        return Ok(salir(datos));
    }
    if es_negativo(texto) {
        guardar_direccion(pool, cliente, datos, false).await
    } else if es_afirmativo(texto) {
        guardar_direccion(pool, cliente, datos, true).await
    } else {
        Ok((r("¿La dejo como tu dirección principal?", &OPCIONES_PRINCIPAL), DIR_PRINCIPAL))
    }
}

async fn guardar_direccion(
    pool: &PgPool,
    cliente: &str,
    datos: &mut Datos,
    principal: bool,
) -> Result<Paso, sqlx::Error> {
    let nombre = datos.dir_nombre.clone().unwrap_or_else(|| "Otra".to_string());
    let direccion = datos.dir_texto.clone().unwrap_or_default();
    let referencia = datos.dir_referencia.clone();
    limpiar(datos);

    if direccion.is_empty() {
        return Ok((r("Se perdió el dato de la dirección. Empecemos de nuevo cuando quieras.", &MENU), LIBRE));
    }
    // Por si dos mensajes seguidos intentaron pasarse del máximo.
    if repo::listar_direcciones(pool, cliente).await?.len() >= MAX_DIRECCIONES {
        return Ok((r("Ya tienes el máximo de direcciones guardadas. Quita una para añadir otra.", &MENU), LIBRE));
    }
    repo::crear_direccion(pool, cliente, &nombre, &direccion, referencia.as_deref(), principal).await?;
    Ok((
        r(
            &format!(
                "Listo, guardé «{}»: {}{}. Puedes verla en «Mi perfil y datos».",
                nombre,
                direccion,
                if principal { " (principal)" } else { "" }
            ),
            &MENU,
        ),
        LIBRE,
    ))
}

// ---------- Personas autorizadas ----------

fn linea_persona(persona: &Persona) -> String {
    format!(
        "• {} ({}) · {} {}",
        formato::a_titulo(&persona.nombre),
        persona.parentesco,
        formato::nombre_tipo_documento(&persona.tipo_documento),
        formato::documento_enmascarado(&persona.numero_documento)
    )
}

fn menu_personas(lista: &[Persona], aviso: Option<&str>) -> Paso {
    let mut botones = Vec::new();
    if lista.len() < MAX_PERSONAS {
        botones.push(AUTORIZAR_PERSONA.to_string());
    }
    if !lista.is_empty() {
        botones.push(QUITAR_PERSONA.to_string());
    }
    botones.push(VOLVER.to_string());

    let mut texto = String::new();
    if let Some(aviso) = aviso {
        texto.push_str(aviso);
        texto.push_str("\n\n");
    }
    if lista.is_empty() {
        texto.push_str("Todavía no tienes personas autorizadas. Una persona autorizada puede recoger tus equipos del taller si tú no puedes ir. ¿Autorizamos a alguien?");
    } else {
        texto.push_str(&format!(
            "Estas personas pueden recoger tus equipos ({} de {}):\n{}\n\n¿Qué quieres hacer?",
            lista.len(),
            MAX_PERSONAS,
            lista.iter().map(linea_persona).collect::<Vec<_>>().join("\n")
        ));
    }
    (con_botones(&texto, botones), PER_ACCION)
}

// «Autorizar a una persona», «quién puede recoger mi equipo»...
pub async fn iniciar_personas(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    limpiar(datos);
    let lista = repo::listar_personas(pool, cliente).await?;
    Ok(menu_personas(&lista, None))
}

pub async fn paso_per_accion(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    if es_cancelar(texto) {
        return Ok(salir(datos));
    }
    let lista = repo::listar_personas(pool, cliente).await?;
    let t = normalizar(texto);

    if t.contains("autoriz") || t.contains("anadir") || t.contains("agregar") || t.contains("nueva") {
        if lista.len() >= MAX_PERSONAS {
            let aviso = format!("Ya tienes {} personas autorizadas, que es el máximo. Quita a una para autorizar a otra.", MAX_PERSONAS);
            return Ok(menu_personas(&lista, Some(&aviso)));
        }
        return Ok((
            r("Dime el nombre completo de la persona, como figura en su DNI.", &[CANCELAR]),
            PER_NOMBRE,
        ));
    }
    if t.contains("quitar") || t.contains("eliminar") || t.contains("borrar") {
        if lista.is_empty() {
            return Ok(menu_personas(&lista, None));
        }
        return Ok(preguntar_cual_persona(&lista));
    }
    Ok(menu_personas(&lista, Some("No entendí cuál de esas opciones quieres. Elige una:")))
}

fn preguntar_cual_persona(lista: &[Persona]) -> Paso {
    let nombres: Vec<String> = lista.iter().map(|p| formato::a_titulo(&p.nombre)).collect();
    let mut botones = etiquetas(&nombres);
    botones.push(VOLVER.to_string());
    (con_botones("¿A quién le quitas la autorización?", botones), PER_QUITAR_CUAL)
}

pub fn paso_per_nombre(datos: &mut Datos, texto: &str) -> Paso {
    if es_cancelar(texto) {
        return salir(datos);
    }
    match validaciones::limpiar_nombre(texto) {
        Some(nombre) if nombre.split_whitespace().count() >= 2 => {
            datos.per_nombre = Some(nombre);
            (r("Escribe su DNI (8 números).", &[CANCELAR]), PER_DOCUMENTO)
        }
        _ => (
            r(
                "Escribe su nombre y su apellido, solo con letras. Por ejemplo: Rosa Quispe Vargas.",
                &[CANCELAR],
            ),
            PER_NOMBRE,
        ),
    }
}

pub async fn paso_per_documento(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    if es_cancelar(texto) {
        return Ok(salir(datos));
    }
    let numero = match validaciones::validar_documento("1", texto) {
        Ok(numero) => numero,
        Err(mensaje) => return Ok((r(&mensaje, &[CANCELAR]), PER_DOCUMENTO)),
    };

    // No tiene sentido autorizarse a uno mismo.
    if let Some(propio) = acceso_repo::obtener_cliente(pool, cliente).await? {
        if propio.tipo_documento == "1" && propio.numero_documento == numero {
            return Ok((
                r("Ese es tu propio DNI: tú ya puedes recoger tus equipos. Escribe el DNI de la otra persona.", &[CANCELAR]),
                PER_DOCUMENTO,
            ));
        }
    }
    // Tampoco repetirla.
    let lista = repo::listar_personas(pool, cliente).await?;
    if lista.iter().any(|p| p.tipo_documento == "1" && p.numero_documento == numero) {
        limpiar(datos);
        return Ok(menu_personas(&lista, Some("Esa persona ya está autorizada.")));
    }

    datos.per_documento = Some(numero);
    Ok((con_botones("¿Qué parentesco o relación tiene contigo? Elige una opción o escríbela.", {
        let mut botones = a_botones(&PARENTESCOS);
        botones.push(CANCELAR.to_string());
        botones
    }), PER_PARENTESCO))
}

pub async fn paso_per_parentesco(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    if es_cancelar(texto) {
        return Ok(salir(datos));
    }
    let parentesco = match elegir_opcion(&PARENTESCOS, texto) {
        Some(posicion) => PARENTESCOS[posicion].to_string(),
        None => match validaciones::limpiar_nombre(texto) {
            Some(propio) if propio.chars().count() <= MAX_PARENTESCO => formato::a_titulo(&propio),
            _ => {
                let mut botones = a_botones(&PARENTESCOS);
                botones.push(CANCELAR.to_string());
                return Ok((con_botones("Elige una opción o escribe la relación con solo letras (hasta 30).", botones), PER_PARENTESCO));
            }
        },
    };
    datos.per_parentesco = Some(parentesco.clone());

    let celular = acceso_repo::obtener_telefono(pool, cliente).await?.unwrap_or_default();
    let resumen = format!(
        "Vas a autorizar a:\n• {}\n• DNI {}\n• Relación: {}\n\nPara confirmarlo te envío un código por SMS a tu celular ({}).",
        formato::a_titulo(datos.per_nombre.as_deref().unwrap_or("")),
        formato::documento_enmascarado(datos.per_documento.as_deref().unwrap_or("")),
        parentesco,
        formato::telefono_enmascarado(&celular)
    );
    Ok((r(&resumen, &OPCIONES_CONFIRMAR_PERSONA), PER_CONFIRMAR))
}

pub async fn paso_per_confirmar(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
    red: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    if es_cancelar(texto) || es_negativo(texto) {
        return Ok(salir(datos));
    }
    let t = normalizar(texto);
    if t.contains("enviar") || t.contains("codigo") || es_afirmativo(texto) {
        return enviar_codigo(pool, conversacion, cliente, red).await;
    }
    Ok((r("Pulsa «Enviar código» para confirmar o «Cancelar» para dejarlo.", &OPCIONES_CONFIRMAR_PERSONA), PER_CONFIRMAR))
}

// Pide el código al celular registrado del cliente y le cuenta qué pasó.
async fn enviar_codigo(
    pool: &PgPool,
    conversacion: &Conversacion,
    cliente: &str,
    red: &str,
) -> Result<Paso, sqlx::Error> {
    let celular = acceso_repo::obtener_telefono(pool, cliente).await?.unwrap_or_default();

    let texto = match verificacion_telefono::solicitar_codigo(pool, &conversacion.codigo, &celular, red).await? {
        Solicitud::Enviado => format!(
            "Te enviamos un código de 6 dígitos por SMS al {}. Escríbelo aquí; vale {} minutos. Si no te llega, pulsa «Reenviar código».",
            formato::telefono_enmascarado(&celular),
            codigo_otp::VIGENCIA_MINUTOS
        ),
        Solicitud::LimiteExcedido => format!(
            "Ya se pidieron {} códigos en los últimos {} minutos. Por seguridad, espera un poco y pulsa «Reenviar código».",
            verificacion_telefono::MAX_SOLICITUDES,
            verificacion_telefono::VENTANA_MINUTOS
        ),
        Solicitud::Demorado => {
            "El SMS está tardando más de lo normal. Espera un momento: si te llega, escribe el código aquí; si no, pulsa «Reenviar código».".to_string()
        }
        Solicitud::NoSePudoEnviar => {
            "No pudimos enviar el SMS en este momento. Espera un minuto y pulsa «Reenviar código».".to_string()
        }
        Solicitud::NumeroInvalido => {
            return Ok((
                r("No pude usar el celular de tu cuenta para enviarte el código. Acércate a la tienda para actualizarlo.", &MENU),
                LIBRE,
            ))
        }
    };
    Ok((r(&texto, &OPCIONES_CODIGO), PER_CODIGO))
}

pub async fn paso_per_codigo(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
    red: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    let t = normalizar(texto);
    if t.trim() == "cancelar" || t.trim() == "volver" {
        return Ok(salir(datos));
    }
    if t.contains("reenviar") || t.contains("otro codigo") || t.contains("nuevo codigo") {
        return enviar_codigo(pool, conversacion, cliente, red).await;
    }

    let Some(codigo) = codigo_otp::codigo_escrito(texto) else {
        return Ok((
            r("Escribe los 6 números del código que te llegó por SMS. Si no te llegó, pulsa «Reenviar código».", &OPCIONES_CODIGO),
            PER_CODIGO,
        ));
    };

    let celular = acceso_repo::obtener_telefono(pool, cliente).await?.unwrap_or_default();
    let mensaje = match verificacion_telefono::verificar_codigo(pool, &conversacion.codigo, &celular, &codigo).await? {
        ResultadoCodigo::Verificado => return guardar_persona(pool, cliente, datos).await,
        ResultadoCodigo::Incorrecto { restantes } => format!(
            "Ese código no coincide. Te {} {} {}. Revísalo e inténtalo de nuevo.",
            if restantes == 1 { "queda" } else { "quedan" },
            restantes,
            if restantes == 1 { "intento" } else { "intentos" }
        ),
        ResultadoCodigo::Vencido => "Ese código ya venció. Pulsa «Reenviar código» para recibir uno nuevo.".to_string(),
        ResultadoCodigo::DemasiadosIntentos => {
            "Agotaste los intentos de ese código. Pulsa «Reenviar código» para recibir uno nuevo.".to_string()
        }
        ResultadoCodigo::SinCodigo => {
            "No tengo un código activo para tu número. Pulsa «Reenviar código» para recibir uno.".to_string()
        }
    };
    Ok((r(&mensaje, &OPCIONES_CODIGO), PER_CODIGO))
}

// El código era correcto: se guarda la autorización.
async fn guardar_persona(pool: &PgPool, cliente: &str, datos: &mut Datos) -> Result<Paso, sqlx::Error> {
    let nombre = datos.per_nombre.clone().unwrap_or_default();
    let documento = datos.per_documento.clone().unwrap_or_default();
    let parentesco = datos.per_parentesco.clone().unwrap_or_default();
    limpiar(datos);

    if nombre.is_empty() || documento.is_empty() || parentesco.is_empty() {
        return Ok((r("Se perdieron los datos de la persona. Empecemos de nuevo cuando quieras.", &MENU), LIBRE));
    }

    let texto = match repo::autorizar_persona(pool, cliente, "1", &documento, &nombre, &parentesco, MAX_PERSONAS as i64).await? {
        ResultadoAutorizacion::Autorizada => format!(
            "Listo, {} quedó autorizada para recoger tus equipos. Recuerda: por seguridad, deberá presentar su DNI físico al momento del recojo en tienda. La ves en «Mi perfil y datos».",
            formato::a_titulo(&nombre)
        ),
        ResultadoAutorizacion::YaAutorizada => "Esa persona ya estaba autorizada.".to_string(),
        ResultadoAutorizacion::LimiteAlcanzado => format!(
            "Ya tienes {} personas autorizadas, que es el máximo. Quita a una para autorizar a otra.",
            MAX_PERSONAS
        ),
    };
    Ok((r(&texto, &MENU), LIBRE))
}

// Quitar: eligió a quién.
pub async fn paso_per_quitar_cual(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    let lista = repo::listar_personas(pool, cliente).await?;
    if es_cancelar(texto) {
        limpiar(datos);
        return Ok(menu_personas(&lista, None));
    }

    let nombres: Vec<String> = lista.iter().map(|p| formato::a_titulo(&p.nombre)).collect();
    let botones = etiquetas(&nombres);
    let referencias: Vec<&str> = botones.iter().map(|b| b.as_str()).collect();
    let Some(posicion) = elegir_opcion(&referencias, texto) else {
        return Ok(preguntar_cual_persona(&lista));
    };

    datos.per_objetivo = Some(lista[posicion].codigo.clone());
    Ok((
        r(
            &format!("¿Quito la autorización de {}?", nombres[posicion]),
            &OPCIONES_QUITAR,
        ),
        PER_QUITAR_CONFIRMAR,
    ))
}

pub async fn paso_per_quitar_confirmar(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.as_deref() else {
        return Ok(pedir_identificarse());
    };
    let objetivo = datos.per_objetivo.clone().unwrap_or_default();

    if es_negativo(texto) || es_cancelar(texto) {
        limpiar(datos);
        return Ok((r("Perfecto, la dejo autorizada. ¿En qué más te ayudo?", &MENU), LIBRE));
    }
    if !es_afirmativo(texto) {
        return Ok((r("¿Quito esa autorización? Responde «Sí, quitar» o «No, mantenerla».", &OPCIONES_QUITAR), PER_QUITAR_CONFIRMAR));
    }

    let quitada = repo::quitar_persona(pool, cliente, &objetivo).await?;
    limpiar(datos);
    let mensaje = if quitada {
        "Listo, esa persona ya no puede recoger tus equipos."
    } else {
        "Esa persona ya no estaba en tu lista."
    };
    Ok((r(mensaje, &MENU), LIBRE))
}

// Los botones que se vuelven a mostrar si la persona recarga la página en medio de un paso con botones fijos.
// (Los pasos cuyos botones salen de la base de datos no se repiten: basta escribir lo que quiere o «volver».)
pub fn opciones_de_reanudacion(paso: &str) -> Vec<String> {
    match paso {
        DIR_TIPO => a_botones(&TIPOS_DIRECCION),
        DIR_REFERENCIA => a_botones(&OPCIONES_REFERENCIA),
        DIR_PRINCIPAL => a_botones(&OPCIONES_PRINCIPAL),
        DIR_QUITAR_CONFIRMAR | PER_QUITAR_CONFIRMAR => a_botones(&OPCIONES_QUITAR),
        DIR_NOMBRE | DIR_TEXTO | PER_NOMBRE | PER_DOCUMENTO => vec![CANCELAR.to_string()],
        PER_PARENTESCO => {
            let mut botones = a_botones(&PARENTESCOS);
            botones.push(CANCELAR.to_string());
            botones
        }
        PER_CONFIRMAR => a_botones(&OPCIONES_CONFIRMAR_PERSONA),
        PER_CODIGO => a_botones(&OPCIONES_CODIGO),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn los_repetidos_llevan_numero() {
        let nombres = vec!["Casa".to_string(), "Oficina".to_string(), "Casa".to_string(), "Casa".to_string()];
        assert_eq!(etiquetas(&nombres), vec!["Casa", "Oficina", "Casa (2)", "Casa (3)"]);
    }

    #[test]
    fn cancelar_solo_con_palabras_claras() {
        assert!(es_cancelar("Cancelar"));
        assert!(es_cancelar(" volver "));
        assert!(!es_cancelar("Av. España 1020, Trujillo"));
        assert!(!es_cancelar("no tengo referencia"));
    }

    #[test]
    fn textos_libres_validos() {
        assert_eq!(texto_valido("  Av.   España  1020 ", 8, 120).as_deref(), Some("Av. España 1020"));
        assert!(texto_valido("Av. 1", 8, 120).is_none());
        assert!(texto_valido(&"x".repeat(121), 8, 120).is_none());
    }

    #[test]
    fn solo_se_ofrecen_las_que_no_son_principales() {
        let lista = vec![
            Direccion { codigo: "A".into(), nombre: "Casa".into(), direccion: "x".into(), referencia: None, principal: true },
            Direccion { codigo: "B".into(), nombre: "Oficina".into(), direccion: "y".into(), referencia: None, principal: false },
        ];
        assert_eq!(candidatas(&lista, "principal").len(), 1);
        assert_eq!(candidatas(&lista, "quitar").len(), 2);
    }
}
