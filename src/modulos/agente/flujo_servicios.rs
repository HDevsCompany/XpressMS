// flujo_servicios.rs: la parte del agente que muestra los servicios técnicos.
//
//   libre -> elegir_servicio -> (el servicio sirve para varios equipos) elegir_equipo -> libre
//                            -> (sirve para uno solo, o ninguno)                       -> libre
//
// En el paso final el agente muestra la tarjeta del servicio (precio base, tiempo estimado, equipo)
// y ofrece agendar una cita. Regla: este archivo NO escribe SQL; usa el repositorio de servicio_tecnico.

use sqlx::PgPool;

use super::modelo::{Componente, Datos};
use super::motor::{detectar_intencion, elegir_opcion, es_negativo, normalizar, Intencion};
use super::pasos::{ELEGIR_EQUIPO, ELEGIR_SERVICIO, LIBRE};
use super::respuesta::{r, Paso, Respuesta, MENU};
use crate::modulos::comun::formato;
use crate::modulos::servicio_tecnico::modelo::{Servicio, TipoEquipo};
use crate::modulos::servicio_tecnico::repositorio;

const VOLVER: &str = "Volver";

// Botones que acompañan a la tarjeta de un servicio.
const OPCIONES_DETALLE: [&str; 2] = ["Agendar una cita", "Ver otros servicios"];

// ---------- Paso 1: mostrar la lista de servicios ----------

// Entrada del flujo: la persona pidió ver los servicios. Los botones salen de la tabla Servicios.
pub async fn mostrar_lista(pool: &PgPool) -> Result<Paso, sqlx::Error> {
    let servicios = repositorio::listar_servicios(pool).await?;
    if servicios.is_empty() {
        return Ok((
            r("Por ahora no tengo servicios para mostrarte. Escríbenos y te ayudamos directamente.", &[]),
            LIBRE,
        ));
    }
    Ok((
        r(
            "Estos son nuestros servicios. ¿Cuál necesitas?",
            &botones_de_servicios(&servicios),
        ),
        ELEGIR_SERVICIO,
    ))
}

// ---------- Paso 2: la persona elige un servicio ----------

pub async fn paso_elegir_servicio(
    pool: &PgPool,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let servicios = repositorio::listar_servicios(pool).await?;
    let nombres: Vec<&str> = servicios.iter().map(|s| s.nombre.as_str()).collect();

    match elegir_opcion(&nombres, texto) {
        Some(posicion) => presentar_servicio(pool, datos, &servicios[posicion]).await,
        // No coincide con ningún servicio: ¿quiere salir? Si no, le mostramos la lista otra vez.
        None if quiere_salir(texto) => Ok(salir()),
        None => Ok((
            r(
                "No encontré ese servicio. Elige uno de la lista:",
                &botones_de_servicios(&servicios),
            ),
            ELEGIR_SERVICIO,
        )),
    }
}

// Con el servicio elegido: si sirve para varios equipos, pregunta cuál; si no, muestra el detalle directamente.
async fn presentar_servicio(
    pool: &PgPool,
    datos: &mut Datos,
    servicio: &Servicio,
) -> Result<Paso, sqlx::Error> {
    let equipos = repositorio::listar_equipos_de_servicio(pool, &servicio.codigo).await?;
    datos.servicio_elegido = Some(servicio.codigo.clone());
    datos.equipo_elegido = None;

    if equipos.len() > 1 {
        let mensaje = format!(
            "Elegiste «{}». ¿Para qué equipo lo necesitas?",
            servicio.nombre
        );
        return Ok((r(&mensaje, &botones_de_equipos(&equipos)), ELEGIR_EQUIPO));
    }

    // Un solo equipo (o ninguno asignado): no hace falta preguntar.
    let equipo = equipos.first();
    datos.equipo_elegido = equipo.map(|e| e.codigo.clone());
    Ok((detalle(servicio, equipo), LIBRE))
}

// ---------- Paso 3: la persona elige el equipo ----------

pub async fn paso_elegir_equipo(
    pool: &PgPool,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    // "Volver" regresa a la lista de servicios.
    if es_volver(texto) {
        return mostrar_lista(pool).await;
    }

    // Si por algún motivo se perdió el servicio (o ya no está activo), empezamos de nuevo por la lista.
    let Some(codigo) = datos.servicio_elegido.clone() else {
        return mostrar_lista(pool).await;
    };
    let Some(servicio) = repositorio::obtener_servicio(pool, &codigo).await? else {
        return mostrar_lista(pool).await;
    };

    let equipos = repositorio::listar_equipos_de_servicio(pool, &servicio.codigo).await?;
    let nombres: Vec<&str> = equipos.iter().map(|e| e.nombre.as_str()).collect();

    match elegir_opcion(&nombres, texto) {
        Some(posicion) => {
            datos.equipo_elegido = Some(equipos[posicion].codigo.clone());
            Ok((detalle(&servicio, Some(&equipos[posicion])), LIBRE))
        }
        None if quiere_salir(texto) => Ok(salir()),
        None => Ok((
            r(
                "Elige uno de los equipos de la lista:",
                &botones_de_equipos(&equipos),
            ),
            ELEGIR_EQUIPO,
        )),
    }
}

// ---------- Piezas de apoyo ----------

// La tarjeta con el detalle del servicio. El precio es solo una base: el costo final se confirma al revisar el equipo.
fn detalle(servicio: &Servicio, equipo: Option<&TipoEquipo>) -> Respuesta {
    let precio = formato::precio_base(servicio.precio_base.as_deref());
    let nota = if servicio.precio_base.is_some() {
        "El costo final se confirma después de revisar tu equipo, según su estado."
    } else {
        "Te damos el precio después de revisar tu equipo."
    };

    let mensaje = format!(
        "Aquí tienes los detalles de «{}». ¿Quieres agendar una cita?",
        servicio.nombre
    );
    r(&mensaje, &OPCIONES_DETALLE).con_componente(Componente::Servicio {
        nombre: servicio.nombre.clone(),
        descripcion: servicio.descripcion.clone(),
        precio,
        nota: nota.to_string(),
        tiempo: servicio.tiempo_estimado.clone(),
        equipo: equipo.map(|e| e.nombre.clone()),
    })
}

// Los nombres de los servicios más el botón "Volver".
fn botones_de_servicios(servicios: &[Servicio]) -> Vec<&str> {
    let mut botones: Vec<&str> = servicios.iter().map(|s| s.nombre.as_str()).collect();
    botones.push(VOLVER);
    botones
}

// Los nombres de los equipos más el botón "Volver".
fn botones_de_equipos(equipos: &[TipoEquipo]) -> Vec<&str> {
    let mut botones: Vec<&str> = equipos.iter().map(|e| e.nombre.as_str()).collect();
    botones.push(VOLVER);
    botones
}

fn es_volver(texto: &str) -> bool {
    normalizar(texto).trim() == VOLVER.to_lowercase()
}

// ¿Lo escrito significa "dejemos esto"? Sí si dice "volver", dice que no, o pide otra cosa conocida
// (cita, saludo, despedida...). Se revisa DESPUÉS de intentar elegir un servicio o equipo.
fn quiere_salir(texto: &str) -> bool {
    es_volver(texto)
        || es_negativo(texto)
        || !matches!(
            detectar_intencion(texto),
            Intencion::Desconocida | Intencion::Servicios
        )
}

fn salir() -> Paso {
    (r("Sin problema. ¿En qué te puedo ayudar?", &MENU), LIBRE)
}
