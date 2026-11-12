// flujo_solicitud.rs: las solicitudes de servicio técnico de la página /servicios. La página solo MUESTRA («Tu solicitud», el selector
// de día y hora, la confirmación); quien conversa, anota y agenda es Juno, con el flujo de citas de siempre (flujo_citas.rs).
//
// Hay dos caminos, y los dos terminan en una orden «Cita agendada»:
//   «Solicitar servicio: <nombre>» / «Solicitar un servicio técnico» -> la persona YA SABE qué necesita su equipo (una reparación, un
//        mantenimiento, cambio de pantalla o de teclado...). Elige el servicio, cuenta qué le pasa y elige día y hora.
//   «Agendar cita de diagnóstico» -> la persona NO SABE qué tiene su equipo: lo lleva al local para que lo revisen y recibe el
//        diagnóstico. Es una orden del servicio «Diagnóstico de equipo» (ver CODIGO_DIAGNOSTICO).
//
//   servicio -> equipo -> detalle (qué le pasa) -> día -> hora -> confirmar -> orden creada
//   (los pasos del flujo de siempre: elegir_servicio, elegir_equipo, cita_problema, cita_fecha, cita_hora, cita_confirmar)
//
// Cada respuesta de la ventana de Juno de /servicios lleva además el componente «solicitud» con lo que ya se anotó y en qué paso va
// (lo arma `estado`): así la página dibuja el cajón «Tu solicitud» sin decidir nada por su cuenta.
// Las órdenes de la página son textos exactos y las atiende `atender` antes que cualquier otra cosa (ver flujo::responder).
// Regla: este archivo NO escribe SQL; usa los repositorios.

use sqlx::PgPool;

use super::flujo_citas;
use super::flujo_servicios;
use super::modelo::{Componente, Conversacion, Datos, DiaCita, HoraCita, OrdenSolicitud};
use super::motor::{elegir_opcion, normalizar};
use super::pasos::{
    CITA_CONFIRMAR, CITA_FECHA, CITA_HORA, CITA_PROBLEMA, ELEGIR_EQUIPO, ELEGIR_ORDEN, ELEGIR_SERVICIO, LIBRE,
    CANCELAR_CITA, CATALOGO,
};
use super::respuesta::{r, Paso, MENU};
use crate::modulos::comun::{fechas, formato};
use crate::modulos::servicio_tecnico::modelo::Servicio;
use crate::modulos::servicio_tecnico::{agenda, repositorio, repositorio_ordenes};

// El servicio «Diagnóstico de equipo» tiene un código fijo (lo crea base_de_datos/12_servicios_y_diagnostico.sql): así la página y Juno
// saben cuál es sin depender de su nombre, que el personal puede cambiar desde el panel.
pub const CODIGO_DIAGNOSTICO: &str = "DIAGNOSTI";

pub const MODO_SERVICIO: &str = "servicio";
pub const MODO_DIAGNOSTICO: &str = "diagnostico";

const OPCIONES_ANONIMO: [&str; 3] = ["Ya soy cliente", "Registrarme", "Solo estoy de paso"];

// ---------- Entender la orden (función pura, con pruebas) ----------

#[derive(Debug, PartialEq)]
pub enum Orden {
    Elegir,           // «Solicitar un servicio técnico»: Juno muestra los servicios para elegir
    Servicio(String), // «Solicitar servicio: <nombre>»
    Diagnostico,      // «Agendar cita de diagnóstico»
    Duda(String),     // «Tengo una duda sobre el servicio: <nombre>» («Preguntarle a Juno» en la ficha)
    Cancelar,         // «Cancelar solicitud»
}

// Solo reconoce las órdenes exactas que envían los botones de la página: cualquier otra frase sigue su camino normal.
pub fn entender(texto: &str) -> Option<Orden> {
    let normal = normalizar(texto);
    let normal = normal.trim();

    if let Some(resto) = normal.strip_prefix("solicitar servicio:") {
        return Some(Orden::Servicio(resto.trim().to_string()));
    }
    if let Some(resto) = normal.strip_prefix("tengo una duda sobre el servicio:") {
        return Some(Orden::Duda(resto.trim().to_string()));
    }
    match normal {
        "solicitar un servicio tecnico" => Some(Orden::Elegir),
        "agendar cita de diagnostico" | "agendar una cita de diagnostico" => Some(Orden::Diagnostico),
        "cancelar solicitud" => Some(Orden::Cancelar),
        _ => None,
    }
}

// ¿En este paso se puede atender una orden de la página? Sí cuando la persona está libre o en medio de una cita (así puede
// cambiar de idea y pedir otro servicio); no en medio de un registro, de un apartado o de «Mi perfil».
pub fn puede_atender(paso: &str) -> bool {
    matches!(
        paso,
        LIBRE
            | "inicio"
            | CATALOGO
            | ELEGIR_SERVICIO
            | ELEGIR_EQUIPO
            | CITA_PROBLEMA
            | CITA_FECHA
            | CITA_HORA
            | CITA_CONFIRMAR
            | ELEGIR_ORDEN
            | CANCELAR_CITA
    )
}

// El mismo paso, como texto fijo (los pasos del flujo de citas se conservan; cualquier otro vuelve a la conversación libre).
fn mismo_paso(paso: &str) -> &'static str {
    match paso {
        ELEGIR_SERVICIO => ELEGIR_SERVICIO,
        ELEGIR_EQUIPO => ELEGIR_EQUIPO,
        CITA_PROBLEMA => CITA_PROBLEMA,
        CITA_FECHA => CITA_FECHA,
        CITA_HORA => CITA_HORA,
        CITA_CONFIRMAR => CITA_CONFIRMAR,
        _ => LIBRE,
    }
}

// ---------- Atender una orden ----------

// Devuelve None cuando lo escrito no es una orden de la página.
pub async fn atender(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Option<Paso>, sqlx::Error> {
    let Some(orden) = entender(texto) else {
        return Ok(None);
    };
    let paso = match orden {
        Orden::Elegir => elegir(pool, conversacion, datos).await?,
        Orden::Servicio(nombre) => servicio_nombrado(pool, conversacion, datos, &nombre).await?,
        Orden::Diagnostico => diagnostico(pool, conversacion, datos).await?,
        Orden::Duda(nombre) => duda(pool, conversacion, &nombre).await?,
        Orden::Cancelar => cancelar(datos),
    };
    Ok(Some(paso))
}

// Empieza una solicitud de cero: se olvida lo que quedó de otra.
fn empezar(datos: &mut Datos, modo: &str) {
    datos.solicitud = Some(modo.to_string());
    datos.servicio_elegido = None;
    datos.equipo_elegido = None;
    datos.cita_fecha = None;
    datos.cita_hora = None;
    datos.cita_problema = None;
    datos.orden_en_vista = None;
}

// Sin registro no hay a quién asociarle la orden ni cómo hacerle seguimiento. Lo elegido se queda guardado.
fn pedir_identidad() -> Paso {
    (
        r(
            "Para dejarte la cita primero necesito saber quién eres: así recibes un código para seguir tu equipo. \
             Si ya eres cliente, pulsa «Ya soy cliente» y te reconozco con tu documento. Lo que elegiste se queda guardado.",
            &OPCIONES_ANONIMO,
        ),
        LIBRE,
    )
}

// «Solicitar un servicio técnico»: Juno muestra los servicios para que la persona elija.
async fn elegir(pool: &PgPool, conversacion: &Conversacion, datos: &mut Datos) -> Result<Paso, sqlx::Error> {
    empezar(datos, MODO_SERVICIO);
    if conversacion.cliente.is_none() {
        return Ok(pedir_identidad());
    }
    flujo_servicios::mostrar_lista_con_texto(pool, "¿Qué servicio necesitas? Elígelo aquí o en la página:").await
}

// «Solicitar servicio: CAMBIO DE PANTALLA»: el servicio ya viene elegido.
async fn servicio_nombrado(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    nombre: &str,
) -> Result<Paso, sqlx::Error> {
    empezar(datos, MODO_SERVICIO);
    let servicios = repositorio::listar_servicios(pool).await?;
    let Some(servicio) = buscar(&servicios, nombre) else {
        if conversacion.cliente.is_none() {
            return Ok(pedir_identidad());
        }
        return flujo_servicios::mostrar_lista_con_texto(
            pool,
            "No encontré ese servicio, puede que ya no esté disponible. Elige uno de la lista:",
        )
        .await;
    };
    if conversacion.cliente.is_none() {
        datos.servicio_elegido = Some(servicio.codigo.clone());
        return Ok(pedir_identidad());
    }
    flujo_servicios::presentar_servicio(pool, datos, servicio).await
}

// «Agendar cita de diagnóstico»: la orden es del servicio «Diagnóstico de equipo».
async fn diagnostico(pool: &PgPool, conversacion: &Conversacion, datos: &mut Datos) -> Result<Paso, sqlx::Error> {
    let Some(servicio) = repositorio::obtener_servicio(pool, CODIGO_DIAGNOSTICO).await? else {
        datos.solicitud = None;
        return Ok((
            r(
                "Por ahora no puedo agendar citas de diagnóstico desde aquí. Escríbenos o acércate a la tienda y te ayudamos directamente.",
                &MENU,
            ),
            LIBRE,
        ));
    };
    empezar(datos, MODO_DIAGNOSTICO);
    if conversacion.cliente.is_none() {
        datos.servicio_elegido = Some(servicio.codigo.clone());
        return Ok(pedir_identidad());
    }
    flujo_servicios::presentar_servicio(pool, datos, &servicio).await
}

// «Preguntarle a Juno» en la ficha de un servicio: Juno dice lo que sabe (precio base y tiempo) y deja la conversación abierta,
// sin perder el lugar si la persona estaba en medio de una cita.
async fn duda(pool: &PgPool, conversacion: &Conversacion, nombre: &str) -> Result<Paso, sqlx::Error> {
    let servicios = repositorio::listar_servicios(pool).await?;
    let paso = mismo_paso(&conversacion.paso);
    let Some(servicio) = buscar(&servicios, nombre) else {
        return Ok((r("No encontré ese servicio. Dime su nombre o tócalo en la página.", &MENU), paso));
    };
    let equipos = repositorio::listar_equipos_de_servicio(pool, &servicio.codigo).await?;
    let mut respuesta = flujo_servicios::detalle(servicio, equipos.first());
    respuesta.texto = format!(
        "Claro. Esto es lo que sé de «{}». ¿Quieres solicitarlo? Si tienes una duda, escríbemela; si es algo que no tengo, lo mejor es confirmarlo en la tienda.",
        servicio.nombre
    );
    respuesta.opciones = vec![format!("Solicitar servicio: {}", servicio.nombre)];
    Ok((respuesta, paso))
}

// «Cancelar solicitud»: Juno deja la solicitud y lo anotado.
fn cancelar(datos: &mut Datos) -> Paso {
    let habia = datos.solicitud.is_some();
    datos.solicitud = None;
    datos.servicio_elegido = None;
    datos.equipo_elegido = None;
    datos.cita_fecha = None;
    datos.cita_hora = None;
    datos.cita_problema = None;
    let texto = if habia {
        "Listo, dejé tu solicitud. Cuando quieras, empezamos de nuevo."
    } else {
        "No tienes ninguna solicitud en curso. ¿En qué te puedo ayudar?"
    };
    (r(texto, &MENU), LIBRE)
}

// El servicio que dice la persona: por su nombre completo o, si no, por parte del nombre (si encaja en uno solo).
fn buscar<'a>(servicios: &'a [Servicio], dicho: &str) -> Option<&'a Servicio> {
    let nombres: Vec<&str> = servicios.iter().map(|s| s.nombre.as_str()).collect();
    elegir_opcion(&nombres, dicho).map(|i| &servicios[i])
}

// La persona se registró o se identificó en medio de una solicitud: se sigue con ella donde se quedó.
// «saludo» es lo que Juno dice primero («¡Perfecto, te reconozco!»); lo demás lo dice el flujo de la cita.
pub async fn reanudar_tras_identificar(
    pool: &PgPool,
    conversacion: &Conversacion,
    cliente: &str,
    datos: &mut Datos,
    saludo: &str,
) -> Result<Paso, sqlx::Error> {
    let con_cliente = Conversacion {
        codigo: conversacion.codigo.clone(),
        token: conversacion.token.clone(),
        paso: LIBRE.to_string(),
        cliente: Some(cliente.to_string()),
        datos: String::new(),
    };
    let (mut respuesta, paso) = flujo_citas::iniciar(pool, &con_cliente, datos).await?;
    respuesta.texto = format!("{} {}", saludo, respuesta.texto);
    Ok((respuesta, paso))
}

// ---------- El estado de «Tu solicitud» ----------

// Lo que la página dibuja en «Tu solicitud»: en qué paso va la persona y lo que Juno ya anotó. Se calcula al final de cada respuesta
// de la ventana de /servicios. «paso_anterior» y «paso» son los pasos de antes y de después de responder; «tenia_solicitud», si ya
// había una solicitud en curso antes de responder.
pub async fn estado(
    pool: &PgPool,
    datos: &mut Datos,
    paso_anterior: &str,
    paso: &str,
    componente: Option<&Componente>,
    tenia_solicitud: bool,
) -> Result<Option<Componente>, sqlx::Error> {
    if !datos.en_servicios {
        return Ok(None);
    }

    // La cita se acaba de confirmar: la solicitud terminó y la página muestra el código de la orden.
    if paso_anterior == CITA_CONFIRMAR {
        if let (
            Some(modo),
            Some(Componente::Orden { codigo: Some(codigo), estado, servicio, equipo, cita, problema, seguimiento }),
        ) = (datos.solicitud.clone(), componente)
        {
            datos.solicitud = None;
            return Ok(Some(Componente::Solicitud {
                modo,
                fase: "listo".to_string(),
                paso: 5,
                servicio: Some(servicio.clone()),
                servicio_linea: None,
                equipo: equipo.clone(),
                detalle: Some(problema.clone()),
                dia: None,
                cita: Some(cita.clone()),
                dias: Vec::new(),
                horas: Vec::new(),
                horario: String::new(),
                orden: Some(OrdenSolicitud {
                    codigo: codigo.clone(),
                    estado: estado.clone(),
                    seguimiento: seguimiento.clone(),
                }),
            }));
        }
    }

    // La persona dejó la solicitud sin terminarla: la página cierra el cajón.
    let Some(modo) = datos.solicitud.clone() else {
        return Ok(if tenia_solicitud { Some(fin()) } else { None });
    };

    let fase = match paso {
        ELEGIR_SERVICIO => "servicio",
        ELEGIR_EQUIPO => "equipo",
        CITA_PROBLEMA => "detalle",
        CITA_FECHA => "dia",
        CITA_HORA => "hora",
        CITA_CONFIRMAR => "confirmar",
        _ => "registro", // todavía no se sabe quién es: se está identificando o registrando
    };
    let numero = match fase {
        "servicio" | "registro" => 1,
        "equipo" => 2,
        "detalle" => 3,
        "dia" | "hora" => 4,
        _ => 5,
    };

    // El servicio y el equipo que ya se eligieron.
    let mut servicio = None;
    let mut servicio_linea = None;
    let mut equipo = None;
    if let Some(codigo) = datos.servicio_elegido.clone() {
        if let Some(elegido) = repositorio::obtener_servicio(pool, &codigo).await? {
            servicio_linea = linea_de_servicio(&elegido);
            if let Some(elegido_equipo) = datos.equipo_elegido.as_deref() {
                let equipos = repositorio::listar_equipos_de_servicio(pool, &elegido.codigo).await?;
                equipo = equipos.into_iter().find(|e| e.codigo == elegido_equipo).map(|e| e.nombre);
            }
            servicio = Some(elegido.nombre);
        }
    }

    // El día y la hora: los días para elegir (fases «dia» y «hora») y las horas del día elegido (fase «hora»).
    let ahora = fechas::ahora_lima();
    let fecha = datos.cita_fecha.as_deref().and_then(fechas::desde_iso);
    let mut dias = Vec::new();
    let mut horas = Vec::new();
    if fase == "dia" || fase == "hora" {
        dias = agenda::dias_disponibles(ahora)
            .into_iter()
            .map(|d| DiaCita {
                texto: fechas::corta(d),
                semana: fechas::semana_corta(d).to_uppercase(),
                numero: d.dia,
                mes: fechas::mes_corto(d),
                elegido: fase == "hora" && Some(d) == fecha,
            })
            .collect();
    }
    if fase == "hora" {
        if let Some(dia) = fecha {
            let ocupadas = repositorio_ordenes::contar_citas_por_hora(pool, &fechas::iso(dia)).await?;
            horas = agenda::horas_reservables(ahora, dia)
                .into_iter()
                .map(|hora| {
                    let citas = ocupadas.iter().find(|(h, _)| *h == hora as i32).map_or(0, |(_, cantidad)| *cantidad);
                    HoraCita {
                        texto: fechas::hora_texto(hora, 0),
                        cupos: (agenda::CITAS_POR_HORARIO - citas).max(0),
                    }
                })
                .collect();
        }
    }
    let cita = match (fecha, datos.cita_hora) {
        (Some(dia), Some(hora)) if fase == "confirmar" => Some(flujo_citas::texto_cita(dia, hora)),
        _ => None,
    };

    Ok(Some(Componente::Solicitud {
        modo,
        fase: fase.to_string(),
        paso: numero,
        servicio,
        servicio_linea,
        equipo,
        detalle: datos.cita_problema.clone(),
        dia: if fase == "hora" { fecha.map(fechas::corta) } else { None },
        cita,
        dias,
        horas,
        horario: agenda::horario_texto(),
        orden: None,
    }))
}

// El estado de una solicitud que se dejó: la página cierra el cajón y el selector.
fn fin() -> Componente {
    Componente::Solicitud {
        modo: String::new(),
        fase: "fin".to_string(),
        paso: 0,
        servicio: None,
        servicio_linea: None,
        equipo: None,
        detalle: None,
        dia: None,
        cita: None,
        dias: Vec::new(),
        horas: Vec::new(),
        horario: String::new(),
        orden: None,
    }
}

// «Desde S/ 150.00 · 1 a 2 días» (o «A cotizar · 1 día»). None si el servicio no tiene ni precio ni tiempo.
fn linea_de_servicio(servicio: &Servicio) -> Option<String> {
    let precio = formato::precio_base(servicio.precio_base.as_deref());
    match servicio.tiempo_estimado.as_deref().filter(|t| !t.trim().is_empty()) {
        Some(tiempo) => Some(format!("{} · {}", precio, tiempo)),
        None if servicio.precio_base.is_some() => Some(precio),
        None => None,
    }
}

// Si la persona recarga /servicios con una solicitud a medias, la página recupera su estado con esto.
pub async fn reanudar(pool: &PgPool, conversacion: &Conversacion) -> Result<Option<Componente>, sqlx::Error> {
    let mut datos: Datos = serde_json::from_str(&conversacion.datos).unwrap_or_default();
    if !datos.en_servicios || datos.solicitud.is_none() {
        return Ok(None);
    }
    estado(pool, &mut datos, "", &conversacion.paso, None, true).await
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn entiende_las_ordenes_de_la_pagina() {
        assert_eq!(entender("Solicitar servicio: CAMBIO DE PANTALLA"), Some(Orden::Servicio("cambio de pantalla".to_string())));
        assert_eq!(entender("Solicitar un servicio técnico"), Some(Orden::Elegir));
        assert_eq!(entender("Agendar cita de diagnóstico"), Some(Orden::Diagnostico));
        assert_eq!(entender("Cancelar solicitud"), Some(Orden::Cancelar));
        assert_eq!(
            entender("Tengo una duda sobre el servicio: Cambio de teclado"),
            Some(Orden::Duda("cambio de teclado".to_string()))
        );
    }

    #[test]
    fn otras_frases_siguen_su_camino() {
        assert_eq!(entender("Quiero agendar una cita"), None);
        assert_eq!(entender("Tengo una duda sobre: Mouse M170"), None); // esa es del carrito
        assert_eq!(entender("cancelar"), None);
        assert_eq!(entender("15:00"), None);
    }

    #[test]
    fn no_interrumpe_un_registro() {
        assert!(puede_atender(LIBRE));
        assert!(puede_atender(CITA_HORA));
        assert!(!puede_atender("consentimiento"));
        assert!(!puede_atender("numero_documento"));
        assert!(!puede_atender("dir_texto"));
    }

    #[test]
    fn conserva_el_paso_de_la_cita() {
        assert_eq!(mismo_paso(CITA_HORA), CITA_HORA);
        assert_eq!(mismo_paso("cualquier_otro"), LIBRE);
    }

    fn servicio(nombre: &str, precio: Option<&str>, tiempo: Option<&str>) -> Servicio {
        Servicio {
            codigo: "X".to_string(),
            nombre: nombre.to_string(),
            descripcion: String::new(),
            precio_base: precio.map(|p| p.to_string()),
            tiempo_estimado: tiempo.map(|t| t.to_string()),
            aviso_precio: true,
        }
    }

    #[test]
    fn linea_de_servicio_segun_lo_que_tenga() {
        assert_eq!(linea_de_servicio(&servicio("A", Some("150.00"), Some("1 a 2 días"))).as_deref(), Some("Desde S/ 150.00 · 1 a 2 días"));
        assert_eq!(linea_de_servicio(&servicio("A", None, Some("1 día"))).as_deref(), Some("A cotizar · 1 día"));
        assert_eq!(linea_de_servicio(&servicio("A", Some("60.00"), None)).as_deref(), Some("Desde S/ 60.00"));
        assert_eq!(linea_de_servicio(&servicio("A", None, None)), None);
    }

    #[test]
    fn busca_el_servicio_por_nombre_completo_o_parcial() {
        let lista = vec![
            servicio("CAMBIO DE PANTALLA", None, None),
            servicio("CAMBIO DE TECLADO", None, None),
            servicio("Mantenimiento preventivo", None, None),
        ];
        assert_eq!(buscar(&lista, "cambio de pantalla").map(|s| s.nombre.as_str()), Some("CAMBIO DE PANTALLA"));
        assert_eq!(buscar(&lista, "mantenimiento").map(|s| s.nombre.as_str()), Some("Mantenimiento preventivo"));
        assert!(buscar(&lista, "cambio").is_none()); // encaja en dos: no adivina
        assert!(buscar(&lista, "formateo").is_none());
    }
}
