// flujo_citas.rs: la parte del agente que agenda citas y muestra el seguimiento de las órdenes.
//
//   libre -> (sin registrar) pide registrarse
//         -> (sin servicio elegido) lista de servicios, y de ahí vuelve a "Agendar una cita"
//         -> cita_fecha -> cita_hora -> cita_problema -> cita_confirmar -> libre (orden creada)
//   libre -> "seguimiento" -> (varias órdenes) elegir_orden -> libre (tarjeta con el seguimiento)
//   libre -> "cancelar mi cita" -> cancelar_cita -> libre
//
// La persona agenda UNA orden que nace como "Cita agendada" y luego avanza por estados en el taller
// (los cambia el personal desde el panel). Regla: este archivo NO escribe SQL; usa los repositorios.
// Los horarios y cupos están en servicio_tecnico/agenda.rs (puro, con pruebas).

use sqlx::PgPool;

use super::flujo_servicios;
use super::modelo::{Componente, Conversacion, Datos};
use super::motor::{elegir_opcion, es_afirmativo, es_negativo, normalizar};
use super::pasos::{CANCELAR_CITA, CITA_CONFIRMAR, CITA_FECHA, CITA_HORA, CITA_PROBLEMA, ELEGIR_ORDEN, LIBRE};
use super::respuesta::{r, Paso, Respuesta, MENU};
use crate::modulos::comun::fechas::{self, Fecha};
use crate::modulos::servicio_tecnico::modelo::{DetalleOrden, NuevaOrden, ResultadoCita, ResumenOrden};
use crate::modulos::servicio_tecnico::{agenda, estados, repositorio, repositorio_ordenes};

const CANCELAR: &str = "Cancelar";
const VOLVER: &str = "Volver";
const CAMBIAR_DIA: &str = "Cambiar día";
const OPCIONES_CONFIRMAR: [&str; 3] = ["Confirmar cita", "Cambiar fecha", "Cancelar"];
const OPCIONES_ANONIMO: [&str; 3] = ["Ya soy cliente", "Registrarme", "Solo estoy de paso"];
const MENSAJE_DIA: &str = "¿Qué día te acomoda? Elige uno:";
const MIN_PROBLEMA: usize = 5;
const MAX_PROBLEMA: usize = 500;
// Cuántas órdenes recientes se ofrecen cuando la persona tiene varias.
const ORDENES_RECIENTES: i64 = 5;

// ---------- Piezas de apoyo ----------

// Una respuesta cuyos botones se calcularon en el momento (días, horas, órdenes).
fn con_botones(texto: &str, botones: Vec<String>) -> Respuesta {
    let referencias: Vec<&str> = botones.iter().map(|b| b.as_str()).collect();
    r(texto, &referencias)
}

// ¿El texto es exactamente una de esas palabras (sin importar mayúsculas ni tildes)?
fn es_palabra(texto: &str, palabras: &[&str]) -> bool {
    let normal = normalizar(texto);
    palabras.contains(&normal.trim())
}

// ¿La persona quiere dejar el trámite? Solo con palabras claras: "cancelar", "volver", "salir" o un "no".
fn quiere_salir(texto: &str) -> bool {
    es_palabra(texto, &["cancelar", "volver", "salir"]) || es_negativo(texto)
}

// Olvida lo recogido para la cita (el servicio y el equipo elegidos, y la solicitud en curso, se conservan).
fn limpiar_cita(datos: &mut Datos) {
    datos.cita_fecha = None;
    datos.cita_hora = None;
    datos.cita_problema = None;
}

// La persona deja el trámite: se olvida la cita y también la solicitud de /servicios que estuviera en curso.
fn salir(datos: &mut Datos) -> Paso {
    limpiar_cita(datos);
    datos.solicitud = None;
    (r("Sin problema. ¿En qué te puedo ayudar?", &MENU), LIBRE)
}

// "lunes 9 de noviembre · 15:00"
pub(super) fn texto_cita(fecha: Fecha, hora: u32) -> String {
    format!("{} · {}", fechas::larga(fecha), fechas::hora_texto(hora, 0))
}

// Lo mismo, pero a partir del texto que guarda la base ("2026-11-09 15:00").
pub(super) fn texto_cita_guardada(guardada: &str) -> String {
    if let Some((dia, hora)) = guardada.split_once(' ') {
        if let Some(fecha) = fechas::desde_iso(dia) {
            return format!("{} · {}", fechas::larga(fecha), hora);
        }
    }
    guardada.to_string()
}

// ---------- Entrada: "Agendar una cita" ----------

pub async fn iniciar(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
) -> Result<Paso, sqlx::Error> {
    // Sin registro no hay a quién asociarle la orden ni cómo hacerle seguimiento.
    if conversacion.cliente.is_none() {
        return Ok((
            r(
                "Para agendar una cita primero necesito registrarte: así recibes un código para seguir tu equipo. \
                 Si ya eres cliente, pulsa «Ya soy cliente» y te reconozco con tu documento.",
                &OPCIONES_ANONIMO,
            ),
            LIBRE,
        ));
    }
    limpiar_cita(datos);

    // Falta el servicio: se elige primero (así la orden dice qué se va a atender).
    let Some(codigo) = datos.servicio_elegido.clone() else {
        return flujo_servicios::mostrar_lista_con_texto(
            pool,
            "Para agendar tu cita, primero elige el servicio que necesitas:",
        )
        .await;
    };
    let Some(servicio) = repositorio::obtener_servicio(pool, &codigo).await? else {
        datos.servicio_elegido = None;
        datos.equipo_elegido = None;
        return flujo_servicios::mostrar_lista_con_texto(
            pool,
            "Ese servicio ya no está disponible. Elige otro para tu cita:",
        )
        .await;
    };

    // Si el servicio sirve para varios equipos y aún no eligió, se le pregunta.
    if datos.equipo_elegido.is_none() {
        let equipos = repositorio::listar_equipos_de_servicio(pool, &servicio.codigo).await?;
        if equipos.len() > 1 {
            return flujo_servicios::presentar_servicio(pool, datos, &servicio).await;
        }
    }
    Ok(pedir_problema(datos, &servicio.nombre))
}

// ---------- Paso 1: qué le pasa al equipo ----------

// La primera pregunta de la cita (ya con el servicio y el equipo elegidos). En el diagnóstico la persona no sabe qué tiene el equipo:
// se le pide que cuente lo que ve.
pub(super) fn pedir_problema(datos: &Datos, servicio: &str) -> Paso {
    let mensaje = if datos.solicitud.as_deref() == Some("diagnostico") {
        "Cuéntame qué le pasa a tu equipo: qué hace o qué dejó de hacer. Si sabes su marca y modelo, dímelo también: \
         así el técnico llega preparado a revisarlo."
            .to_string()
    } else if servicio.is_empty() {
        "Cuéntame qué le pasa a tu equipo y, si lo sabes, su marca y modelo.".to_string()
    } else {
        format!(
            "Perfecto, «{}». Cuéntame qué le pasa a tu equipo y, si lo sabes, su marca y modelo.",
            servicio
        )
    };
    (r(&mensaje, &[CANCELAR]), CITA_PROBLEMA)
}

// ---------- Paso 2: el día ----------

fn pedir_dia(mensaje: &str) -> Paso {
    let dias = agenda::dias_disponibles(fechas::ahora_lima());
    if dias.is_empty() {
        return (
            r(
                "Por ahora no tengo días disponibles para agendar. Escríbenos y te ayudamos directamente.",
                &MENU,
            ),
            LIBRE,
        );
    }
    let mut botones: Vec<String> = dias.iter().map(|d| fechas::corta(*d)).collect();
    botones.push(CANCELAR.to_string());
    (con_botones(mensaje, botones), CITA_FECHA)
}

pub async fn paso_fecha(
    pool: &PgPool,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    if quiere_salir(texto) {
        return Ok(salir(datos));
    }
    let dias = agenda::dias_disponibles(fechas::ahora_lima());
    let nombres: Vec<String> = dias.iter().map(|d| fechas::corta(*d)).collect();
    let referencias: Vec<&str> = nombres.iter().map(|n| n.as_str()).collect();

    // Acepta el botón ("lun 09/11") o un día escrito ("9/11").
    let posicion = elegir_opcion(&referencias, texto).or_else(|| agenda::dia_desde_texto(texto, &dias));
    match posicion {
        Some(i) => {
            datos.cita_fecha = Some(fechas::iso(dias[i]));
            mostrar_horas(pool, datos, dias[i], None).await
        }
        None => Ok(pedir_dia("Elige uno de los días disponibles:")),
    }
}

// ---------- Paso 3: la hora ----------

// Muestra las horas con cupo de ese día. "aviso" va antes del mensaje (por ejemplo, "Esa hora se acaba de llenar.").
async fn mostrar_horas(
    pool: &PgPool,
    datos: &mut Datos,
    fecha: Fecha,
    aviso: Option<&str>,
) -> Result<Paso, sqlx::Error> {
    let ocupadas = repositorio_ordenes::contar_citas_por_hora(pool, &fechas::iso(fecha)).await?;
    let libres = agenda::horas_libres(fechas::ahora_lima(), fecha, &ocupadas);

    if libres.is_empty() {
        datos.cita_fecha = None;
        let mensaje = format!("El {} ya no tiene horas libres. Elige otro día:", fechas::larga(fecha));
        return Ok(pedir_dia(&mensaje));
    }

    let mut botones = agenda::botones_de_horas(&libres);
    botones.push(CAMBIAR_DIA.to_string());
    let base = format!(
        "Para el {} tengo estas horas libres. ¿Cuál prefieres?",
        fechas::larga(fecha)
    );
    let mensaje = match aviso {
        Some(aviso) => format!("{} {}", aviso, base),
        None => base,
    };
    Ok((con_botones(&mensaje, botones), CITA_HORA))
}

pub async fn paso_hora(
    pool: &PgPool,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    if quiere_salir(texto) {
        return Ok(salir(datos));
    }
    if es_palabra(texto, &["cambiar dia", "cambiar fecha"]) {
        return Ok(pedir_dia(MENSAJE_DIA));
    }
    let Some(fecha) = datos.cita_fecha.as_deref().and_then(fechas::desde_iso) else {
        return Ok(pedir_dia(MENSAJE_DIA));
    };

    // Elegir otro día mientras se miran las horas («mar 10/11»): se muestran las horas de ese día.
    let dias = agenda::dias_disponibles(fechas::ahora_lima());
    if let Some(i) = agenda::dia_desde_texto(texto, &dias) {
        datos.cita_fecha = Some(fechas::iso(dias[i]));
        return mostrar_horas(pool, datos, dias[i], None).await;
    }

    let ocupadas = repositorio_ordenes::contar_citas_por_hora(pool, &fechas::iso(fecha)).await?;
    let libres = agenda::horas_libres(fechas::ahora_lima(), fecha, &ocupadas);
    let nombres = agenda::botones_de_horas(&libres);
    let referencias: Vec<&str> = nombres.iter().map(|n| n.as_str()).collect();

    // Acepta el botón ("15:00") o una hora escrita ("15", "3:00"), siempre que tenga cupo.
    let hora = elegir_opcion(&referencias, texto)
        .map(|i| libres[i])
        .or_else(|| agenda::hora_desde_texto(texto).filter(|h| libres.contains(h)));

    match hora {
        Some(hora) => {
            datos.cita_hora = Some(hora);
            pedir_confirmacion(pool, datos).await
        }
        None => mostrar_horas(pool, datos, fecha, Some("Esa hora no está disponible.")).await,
    }
}

// ---------- Paso 1 (respuesta): el problema ----------

pub async fn paso_problema(
    pool: &PgPool,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    if es_palabra(texto, &["cancelar", "volver", "salir"]) {
        return Ok(salir(datos));
    }
    // Sin espacios repetidos ni saltos de línea.
    let problema = texto.split_whitespace().collect::<Vec<_>>().join(" ");
    let largo = problema.chars().count();
    if largo < MIN_PROBLEMA {
        return Ok((
            r("Cuéntame un poco más para que el técnico sepa qué revisar.", &[CANCELAR]),
            CITA_PROBLEMA,
        ));
    }
    if largo > MAX_PROBLEMA {
        return Ok((
            r("Es un poco largo. ¿Puedes resumirlo en menos de 500 caracteres?", &[CANCELAR]),
            CITA_PROBLEMA,
        ));
    }
    datos.cita_problema = Some(problema);
    // Con el problema anotado se elige el día y la hora (si por algún motivo ya estaban elegidos, se pasa a confirmar).
    if datos.cita_fecha.is_some() && datos.cita_hora.is_some() {
        return pedir_confirmacion(pool, datos).await;
    }
    Ok(pedir_dia(MENSAJE_DIA))
}

// ---------- Paso 4: confirmar ----------

// La tarjeta con el resumen de la cita, antes de guardarla.
async fn pedir_confirmacion(pool: &PgPool, datos: &mut Datos) -> Result<Paso, sqlx::Error> {
    let (Some(codigo), Some(dia), Some(hora)) =
        (datos.servicio_elegido.clone(), datos.cita_fecha.clone(), datos.cita_hora)
    else {
        return Ok(pedir_dia(MENSAJE_DIA));
    };
    let Some(problema) = datos.cita_problema.clone() else {
        return Ok(pedir_problema(datos, ""));
    };
    let Some(fecha) = fechas::desde_iso(&dia) else {
        return Ok(pedir_dia(MENSAJE_DIA));
    };
    let Some(servicio) = repositorio::obtener_servicio(pool, &codigo).await? else {
        datos.servicio_elegido = None;
        datos.equipo_elegido = None;
        limpiar_cita(datos);
        return flujo_servicios::mostrar_lista_con_texto(
            pool,
            "Ese servicio ya no está disponible. Elige otro para tu cita:",
        )
        .await;
    };

    // El nombre del equipo elegido (si el servicio distingue equipos).
    let equipos = repositorio::listar_equipos_de_servicio(pool, &servicio.codigo).await?;
    let equipo = datos
        .equipo_elegido
        .as_deref()
        .and_then(|elegido| equipos.iter().find(|e| e.codigo == elegido))
        .map(|e| e.nombre.clone());

    let tarjeta = Componente::Orden {
        codigo: None,
        estado: "Pendiente de confirmar".to_string(),
        servicio: servicio.nombre,
        equipo,
        cita: texto_cita(fecha, hora),
        problema,
        seguimiento: Vec::new(),
    };
    Ok((
        r("Revisa los datos de tu cita. ¿La confirmo?", &OPCIONES_CONFIRMAR).con_componente(tarjeta),
        CITA_CONFIRMAR,
    ))
}

pub async fn paso_confirmar(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let normal = normalizar(texto);
    if normal.contains("cambiar") {
        return Ok(pedir_dia(MENSAJE_DIA));
    }
    if quiere_salir(texto) {
        return Ok(salir(datos));
    }
    if !(normal.contains("confirm") || es_afirmativo(texto)) {
        return Ok((
            r("¿Confirmo tu cita? Elige una opción:", &OPCIONES_CONFIRMAR),
            CITA_CONFIRMAR,
        ));
    }
    crear_cita(pool, conversacion, datos).await
}

// Guarda la orden. Se vuelve a revisar todo en el servidor: la hora pudo pasar o llenarse mientras la persona decidía.
async fn crear_cita(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.clone() else {
        return Ok(salir(datos));
    };
    let (Some(servicio), Some(dia), Some(hora), Some(problema)) = (
        datos.servicio_elegido.clone(),
        datos.cita_fecha.clone(),
        datos.cita_hora,
        datos.cita_problema.clone(),
    ) else {
        return Ok(pedir_dia(MENSAJE_DIA));
    };
    let Some(fecha) = fechas::desde_iso(&dia) else {
        return Ok(pedir_dia(MENSAJE_DIA));
    };
    if !agenda::horas_reservables(fechas::ahora_lima(), fecha).contains(&hora) {
        return mostrar_horas(pool, datos, fecha, Some("Esa hora ya no se puede reservar.")).await;
    }

    let nueva = NuevaOrden {
        cliente: cliente.clone(),
        servicio,
        tipo_equipo: datos.equipo_elegido.clone(),
        problema,
        fecha_hora: agenda::fecha_hora_texto(fecha, hora),
    };
    match repositorio_ordenes::crear_orden(pool, &nueva, agenda::CITAS_POR_HORARIO).await? {
        ResultadoCita::SinCupo => {
            mostrar_horas(pool, datos, fecha, Some("Justo esa hora se acaba de llenar.")).await
        }
        ResultadoCita::Creada(codigo) => {
            // La orden ya existe: se olvida lo de la cita y el servicio elegido.
            limpiar_cita(datos);
            datos.servicio_elegido = None;
            datos.equipo_elegido = None;

            let mensaje = format!(
                "¡Listo! Tu cita quedó agendada. Tu código de orden es {}. Guárdalo: con él, o escribiendo «seguimiento», \
                 puedes ver cómo va tu equipo. Si no puedes asistir, puedes cancelarla aquí mismo.",
                codigo
            );
            match repositorio_ordenes::obtener_orden_de_cliente(pool, &codigo, &cliente).await? {
                Some(detalle) => {
                    let tarjeta = tarjeta_de_orden(pool, &detalle).await?;
                    let opciones = opciones_de_orden(datos, &detalle);
                    Ok((r(&mensaje, &opciones).con_componente(tarjeta), LIBRE))
                }
                None => Ok((r(&mensaje, &MENU), LIBRE)),
            }
        }
    }
}

// ---------- Seguimiento: "Ver mi orden" ----------

// Arma la tarjeta de una orden con su seguimiento (pasos hechos, actual y pendientes).
async fn tarjeta_de_orden(pool: &PgPool, detalle: &DetalleOrden) -> Result<Componente, sqlx::Error> {
    let catalogo = repositorio_ordenes::listar_estados(pool).await?;
    let historial = repositorio_ordenes::historial_de_orden(pool, &detalle.codigo).await?;
    Ok(Componente::Orden {
        codigo: Some(detalle.codigo.clone()),
        estado: detalle.estado_nombre.clone(),
        servicio: detalle.servicio.clone(),
        equipo: detalle.equipo.clone(),
        cita: texto_cita_guardada(&detalle.fecha_cita),
        problema: detalle.problema.clone(),
        seguimiento: estados::seguimiento(&catalogo, &detalle.estado_codigo, &historial),
    })
}

// Los botones que acompañan a una orden. Si todavía es una cita, se puede cancelar (y se recuerda cuál,
// para que «Cancelar mi cita» sepa de cuál hablamos).
fn opciones_de_orden(datos: &mut Datos, detalle: &DetalleOrden) -> Vec<&'static str> {
    if estados::cliente_puede_cancelar(&detalle.estado_codigo) {
        datos.orden_en_vista = Some(detalle.codigo.clone());
        vec!["Cancelar mi cita", "Ver servicios"]
    } else {
        datos.orden_en_vista = None;
        vec!["Ver servicios", "Agendar una cita"]
    }
}

// La persona pidió ver su orden. Con una sola se muestra directo; con varias, elige.
pub async fn ver_ordenes(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.clone() else {
        return Ok((
            r(
                "Para consultar tus órdenes primero necesito saber quién eres. \
                 Si ya eres cliente, pulsa «Ya soy cliente» y te reconozco con tu documento.",
                &OPCIONES_ANONIMO,
            ),
            LIBRE,
        ));
    };

    let ordenes = repositorio_ordenes::listar_ordenes_de_cliente(pool, &cliente, ORDENES_RECIENTES).await?;
    match ordenes.len() {
        0 => Ok((
            r(
                "Todavía no tienes citas ni órdenes. ¿Quieres agendar una?",
                &["Agendar una cita", "Ver servicios"],
            ),
            LIBRE,
        )),
        1 => mostrar_orden(pool, datos, &cliente, &ordenes[0].codigo).await,
        _ => {
            let mut botones: Vec<String> = ordenes.iter().map(etiqueta_de_orden).collect();
            botones.push(VOLVER.to_string());
            Ok((
                con_botones("Estas son tus órdenes más recientes. ¿Cuál quieres ver?", botones),
                ELEGIR_ORDEN,
            ))
        }
    }
}

// "K7TQ3M9XA · Formateo e instalación de Windows"
fn etiqueta_de_orden(orden: &ResumenOrden) -> String {
    format!("{} · {}", orden.codigo, orden.servicio)
}

pub async fn paso_elegir_orden(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.clone() else {
        return Ok(salir(datos));
    };
    if quiere_salir(texto) {
        return Ok(salir(datos));
    }

    let ordenes = repositorio_ordenes::listar_ordenes_de_cliente(pool, &cliente, ORDENES_RECIENTES).await?;
    let etiquetas: Vec<String> = ordenes.iter().map(etiqueta_de_orden).collect();
    let referencias: Vec<&str> = etiquetas.iter().map(|e| e.as_str()).collect();

    match elegir_opcion(&referencias, texto) {
        Some(i) => mostrar_orden(pool, datos, &cliente, &ordenes[i].codigo).await,
        None => {
            let mut botones = etiquetas;
            botones.push(VOLVER.to_string());
            Ok((
                con_botones("No encontré esa orden. Elige una de la lista:", botones),
                ELEGIR_ORDEN,
            ))
        }
    }
}

// La tarjeta de una orden del cliente (nunca de otro: la consulta exige que sea suya).
async fn mostrar_orden(
    pool: &PgPool,
    datos: &mut Datos,
    cliente: &str,
    codigo: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(detalle) = repositorio_ordenes::obtener_orden_de_cliente(pool, codigo, cliente).await? else {
        return Ok((r("No encontré esa orden. ¿En qué más te puedo ayudar?", &MENU), LIBRE));
    };
    let tarjeta = tarjeta_de_orden(pool, &detalle).await?;
    let opciones = opciones_de_orden(datos, &detalle);
    let mensaje = format!(
        "Aquí está tu orden {}. Estado actual: {}.",
        detalle.codigo, detalle.estado_nombre
    );
    Ok((r(&mensaje, &opciones).con_componente(tarjeta), LIBRE))
}

// ---------- Cancelar una cita ----------

// "Cancelar mi cita": pide confirmación de la orden que la persona tiene a la vista. Si no hay ninguna, primero la muestra.
pub async fn pedir_cancelacion(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
) -> Result<Paso, sqlx::Error> {
    let Some(codigo) = datos.orden_en_vista.clone() else {
        return ver_ordenes(pool, conversacion, datos).await;
    };
    let mensaje = format!("¿Seguro que quieres cancelar la cita de la orden {}?", codigo);
    Ok((r(&mensaje, &["Sí, cancelar", "No, mantenerla"]), CANCELAR_CITA))
}

pub async fn paso_cancelar(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let (Some(cliente), Some(codigo)) = (conversacion.cliente.clone(), datos.orden_en_vista.clone()) else {
        return Ok(salir(datos));
    };

    if es_negativo(texto) || es_palabra(texto, &["volver"]) {
        return mostrar_orden(pool, datos, &cliente, &codigo).await;
    }
    if es_afirmativo(texto) || normalizar(texto).contains("cancelar") {
        let cancelada = repositorio_ordenes::cancelar_cita_de_cliente(pool, &codigo, &cliente).await?;
        datos.orden_en_vista = None;
        let mensaje = if cancelada {
            "Listo, cancelé tu cita. Cuando quieras, puedes agendar otra."
        } else {
            "No pude cancelarla porque tu equipo ya está en proceso. Comunícate con H&Devs y te ayudan."
        };
        return Ok((r(mensaje, &MENU), LIBRE));
    }
    Ok((
        r("¿Cancelo la cita? Responde sí o no.", &["Sí, cancelar", "No, mantenerla"]),
        CANCELAR_CITA,
    ))
}
