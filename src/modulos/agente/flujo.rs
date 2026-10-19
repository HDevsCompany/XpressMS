// flujo.rs: lo que el agente DICE en cada momento y en qué paso va la conversación.
// Es una "máquina de estados": la conversación siempre está en un PASO (guardado en la base de datos),
// y según el paso en que esté, el mensaje de la persona significa una cosa u otra.
//
//   inicio / libre -> consentimiento -> tipo_documento -> numero_documento
//        numero_documento -> (documento ya registrado)  verificar_telefono -> libre
//        numero_documento -> (datos encontrados)        confirmar_identidad -> [direccion] -> telefono
//        numero_documento -> (no encontrado)            manual_nombres -> manual_paterno -> manual_materno -> direccion -> telefono
//                                                       (RUC: manual_razon_social -> direccion -> telefono)
//        telefono -> correo -> libre (cliente creado)
//   libre -> agregar_correo -> libre (un cliente que omitió su correo lo agrega después)
//   libre -> elegir_servicio -> elegir_equipo -> libre (la consulta de servicios vive en flujo_servicios.rs)
//
// Los nombres de los pasos están en pasos.rs y las piezas para armar respuestas en respuesta.rs.
//
// Regla: este archivo NO escribe SQL. Para leer o guardar datos llama a los repositorios de los
// módulos comun, acceso y agente.

use sqlx::PgPool;

use super::flujo_servicios;
use super::modelo::{Campo, Componente, Conversacion, Datos, Verificacion};
use super::motor::{detectar_intencion, es_afirmativo, es_negativo, normalizar, Intencion};
use super::pasos::{
    AGREGAR_CORREO, CONFIRMAR_IDENTIDAD, CONSENTIMIENTO, CORREO, DIRECCION, ELEGIR_EQUIPO,
    ELEGIR_SERVICIO, LIBRE, MANUAL_MATERNO, MANUAL_NOMBRES, MANUAL_PATERNO, MANUAL_RAZON_SOCIAL,
    NUMERO_DOCUMENTO, TELEFONO, TIPO_DOCUMENTO, VERIFICAR_TELEFONO,
};
use super::repositorio;
use super::respuesta::{r, Paso, Respuesta, MENU};
use crate::modulos::acceso::modelo::{ClienteCompleto, NuevoCliente};
use crate::modulos::acceso::{consulta_documento, repositorio as acceso_repo};
use crate::modulos::comun::{formato, repositorio as comun_repo, validaciones};

// Privacidad: ¿la tarjeta de confirmación muestra los datos de una persona completos?
// Con "false" (recomendado) la fecha de nacimiento y la dirección salen parcialmente ocultas
// ("••/••/1995", "•••••• - TRUJILLO"), porque cualquiera que conozca un DNI podría escribirlo en el chat
// y ver esos datos. Cambia a "true" solo si el negocio lo decide de forma consciente.
const MOSTRAR_DATOS_COMPLETOS: bool = false;

// ---------- Punto de entrada ----------

// El primer mensaje de cualquier conversación nueva.
pub fn saludo_inicial() -> Respuesta {
    r(
        "¡Hola! 👋 Soy el asistente de H&Devs. Puedo ayudarte con nuestro servicio técnico. \
         ¿Quieres que te registre para atenderte mejor, o solo estás de paso? \
         Si es así, no hay problema: puedes preguntarme lo que desees.",
        &["Quiero registrarme", "Solo estoy de paso"],
    )
}

// Si la persona recarga la página en medio de un paso que tiene elemento visual, hay que volver a dibujarlo.
// Hoy solo el consentimiento lo necesita.
pub fn componente_de_reanudacion(paso: &str) -> Option<Componente> {
    if paso == CONSENTIMIENTO {
        Some(Componente::Consentimiento)
    } else {
        None
    }
}

// Decide qué responder a lo que escribió la persona, según el paso en que va la conversación.
// "accion" viene de los elementos visuales (por ejemplo "aceptar_terminos" al pulsar Continuar);
// cuando la persona solo escribe, es None.
// Después guarda el nuevo paso y los datos recogidos para continuar en el próximo mensaje.
pub async fn responder(
    pool: &PgPool,
    conversacion: &Conversacion,
    texto: &str,
    accion: Option<&str>,
) -> Result<Respuesta, sqlx::Error> {
    // Recupera lo recogido hasta ahora (si el JSON estuviera dañado, empieza de cero).
    let mut datos: Datos = serde_json::from_str(&conversacion.datos).unwrap_or_default();

    // Cada paso tiene su propia función. Los pasos "inicio" y "libre" usan paso_libre.
    let (respuesta, nuevo_paso) = match conversacion.paso.as_str() {
        CONSENTIMIENTO => paso_consentimiento(pool, &mut datos, texto, accion).await?,
        TIPO_DOCUMENTO => paso_tipo_documento(pool, &mut datos, texto).await?,
        NUMERO_DOCUMENTO => paso_numero_documento(pool, &mut datos, texto).await?,
        VERIFICAR_TELEFONO => {
            paso_verificar_telefono(pool, conversacion, &mut datos, texto).await?
        }
        CONFIRMAR_IDENTIDAD => paso_confirmar_identidad(pool, &mut datos, texto).await?,
        MANUAL_NOMBRES => paso_manual_nombres(&mut datos, texto),
        MANUAL_PATERNO => paso_manual_paterno(&mut datos, texto),
        MANUAL_MATERNO => paso_manual_materno(&mut datos, texto),
        MANUAL_RAZON_SOCIAL => paso_manual_razon_social(&mut datos, texto),
        DIRECCION => paso_direccion(&mut datos, texto),
        TELEFONO => paso_telefono(&mut datos, texto),
        CORREO => paso_correo(pool, conversacion, &mut datos, texto).await?,
        AGREGAR_CORREO => paso_agregar_correo(pool, conversacion, texto).await?,
        ELEGIR_SERVICIO => flujo_servicios::paso_elegir_servicio(pool, &mut datos, texto).await?,
        ELEGIR_EQUIPO => flujo_servicios::paso_elegir_equipo(pool, &mut datos, texto).await?,
        _ => paso_libre(pool, conversacion, texto).await?,
    };

    repositorio::guardar_estado(pool, &conversacion.codigo, nuevo_paso, &datos).await?;
    Ok(respuesta)
}

// ---------- Conversación libre ----------

// La persona pregunta lo que quiera. Si pide registrarse, empieza el registro.
async fn paso_libre(
    pool: &PgPool,
    conversacion: &Conversacion,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let paso = match detectar_intencion(texto) {
        Intencion::Registrarse => {
            if conversacion.cliente.is_some() {
                (
                    r(
                        "¡Ya estás registrado, gracias! ¿En qué te puedo ayudar?",
                        &MENU,
                    ),
                    LIBRE,
                )
            } else {
                (pedir_consentimiento(), CONSENTIMIENTO)
            }
        }
        Intencion::AgregarCorreo => return pedir_correo_adicional(pool, conversacion).await,
        Intencion::Saludo => (
            r(
                "¡Hola! ¿En qué te puedo ayudar con el servicio técnico?",
                &MENU,
            ),
            LIBRE,
        ),
        Intencion::SoloDePaso => (
            r(
                "¡Sin problema! Pregúntame lo que quieras sobre nuestros servicios.",
                &MENU,
            ),
            LIBRE,
        ),
        Intencion::Servicios => return flujo_servicios::mostrar_lista(pool).await,
        Intencion::AgendarCita => (
            r(
                "Claro, vamos a agendar tu cita. (El agendamiento lo construimos más adelante.)",
                &[],
            ),
            LIBRE,
        ),
        Intencion::Despedida => (
            r("¡Gracias a ti! Aquí estaré si necesitas algo más. 😊", &[]),
            LIBRE,
        ),
        Intencion::Desconocida => (
            r(
                "Mmm, no estoy seguro de haber entendido. ¿Puedes decirlo de otra forma?",
                &MENU,
            ),
            LIBRE,
        ),
    };
    Ok(paso)
}

// ---------- Agregar el correo después del registro ----------

// La persona pidió agregar su correo (por ejemplo, porque pulsó «Omitir» al registrarse).
// Solo se permite si ya es cliente y todavía NO tiene un correo guardado.
async fn pedir_correo_adicional(
    pool: &PgPool,
    conversacion: &Conversacion,
) -> Result<Paso, sqlx::Error> {
    let Some(codigo) = conversacion.cliente.as_deref() else {
        return Ok((
            r("Para guardar tu correo primero necesito registrarte. ¿Quieres que lo hagamos ahora?", &["Quiero registrarme", "Solo estoy de paso"]),
            LIBRE,
        ));
    };

    let correo_actual = acceso_repo::obtener_cliente(pool, codigo)
        .await?
        .and_then(|c| c.correo)
        .filter(|c| !c.is_empty());
    match correo_actual {
        // Ya tiene uno: no lo cambiamos desde el chat (se mostraría solo enmascarado).
        Some(correo) => {
            let mensaje = format!(
                "Ya tienes un correo registrado ({}). Si necesitas cambiarlo, contáctanos y te ayudamos.",
                formato::correo_enmascarado(&correo)
            );
            Ok((r(&mensaje, &MENU), LIBRE))
        }
        None => Ok((
            r("Claro. ¿Cuál es tu correo electrónico?", &["Cancelar"]),
            AGREGAR_CORREO,
        )),
    }
}

async fn paso_agregar_correo(
    pool: &PgPool,
    conversacion: &Conversacion,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(codigo) = conversacion.cliente.as_deref() else {
        return Ok((
            r(
                "No encontré tu registro. ¿Quieres registrarte?",
                &["Quiero registrarme"],
            ),
            LIBRE,
        ));
    };

    if validaciones::correo_valido(texto) {
        let correo = texto.trim().to_lowercase();
        if !acceso_repo::agregar_correo(pool, codigo, &correo).await? {
            return Ok((r("Ya tenías un correo registrado, así que no lo cambié. ¿En qué más te puedo ayudar?", &MENU), LIBRE));
        }
        // Guardado: volvemos a mostrar la credencial, ya con el correo incluido.
        let respuesta = match acceso_repo::obtener_cliente(pool, codigo).await? {
            Some(cliente) => {
                let datos = datos_de_cliente(&cliente);
                r("¡Listo! Guardé tu correo. Así quedó tu credencial:", &MENU).con_componente(
                    credencial(
                        &cliente.tipo_documento,
                        &cliente.numero_documento,
                        &datos,
                        Some(codigo),
                    ),
                )
            }
            None => r("¡Listo! Guardé tu correo.", &MENU),
        };
        Ok((respuesta, LIBRE))
    } else if normalizar(texto).trim() == "cancelar" || es_negativo(texto) {
        Ok((r("Sin problema, lo dejamos así. Cuando quieras puedes agregarlo. ¿En qué más te puedo ayudar?", &MENU), LIBRE))
    } else {
        Ok((
            r(
                "Ese correo no parece válido. Escríbelo de nuevo o pulsa «Cancelar».",
                &["Cancelar"],
            ),
            AGREGAR_CORREO,
        ))
    }
}

// Convierte un cliente guardado en "Datos", que es lo que usa credencial() para dibujar la tarjeta.
fn datos_de_cliente(cliente: &ClienteCompleto) -> Datos {
    Datos {
        tipo_documento: Some(cliente.tipo_documento.clone()),
        numero_documento: Some(cliente.numero_documento.clone()),
        paterno: cliente.paterno.clone(),
        materno: cliente.materno.clone(),
        nombres: cliente.nombres.clone(),
        razon_social: cliente.razon_social.clone(),
        direccion: Some(cliente.direccion.clone()),
        sexo: cliente.sexo.clone(),
        fecha_nacimiento: cliente.fecha_nacimiento.clone(),
        telefono: Some(cliente.telefono.clone()),
        correo: cliente.correo.clone(),
        ..Default::default()
    }
}

// ---------- Registro: consentimiento ----------

// Ley N.° 29733: el consentimiento debe ser previo, informado, expreso e inequívoco.
// Por eso NO basta con que la persona escriba "sí": debe marcar la casilla de la tarjeta de términos
// (donde puede abrir y leer el texto legal completo) y pulsar "Continuar".
fn pedir_consentimiento() -> Respuesta {
    r(
        "Para registrarte necesito guardar algunos datos tuyos. Antes de continuar, revisa los términos y acepta si estás de acuerdo.",
        &[],
    )
    .con_componente(Componente::Consentimiento)
}

async fn paso_consentimiento(
    pool: &PgPool,
    datos: &mut Datos,
    texto: &str,
    accion: Option<&str>,
) -> Result<Paso, sqlx::Error> {
    match accion {
        // Marcó la casilla y pulsó "Continuar": dejamos constancia de la fecha y hora exactas.
        Some("aceptar_terminos") => {
            datos.consentimiento = Some(comun_repo::ahora_texto(pool).await?);
            pedir_tipo_documento(pool).await
        }
        // Pulsó "No, gracias": no guardamos nada y puede seguir consultando.
        Some("rechazar_terminos") => Ok((
            r(
                "Entendido, no guardaré ningún dato. Puedes seguir preguntándome lo que quieras.",
                &MENU,
            ),
            LIBRE,
        )),
        // Escribió algo en lugar de usar la tarjeta.
        _ => {
            if es_negativo(texto) {
                Ok((r("Entendido, no guardaré ningún dato. Puedes seguir preguntándome lo que quieras.", &MENU), LIBRE))
            } else {
                // Escribir "sí" no cuenta como aceptación: volvemos a mostrar la tarjeta.
                Ok((
                    r("Para continuar necesito que marques la casilla de Términos y Condiciones. Puedes leerlos antes de aceptar.", &[])
                        .con_componente(Componente::Consentimiento),
                    CONSENTIMIENTO,
                ))
            }
        }
    }
}

// ---------- Registro: tipo y número de documento ----------

// Pregunta con qué documento quiere registrarse. Las opciones salen de la tabla Comun.TiposDocumento.
async fn pedir_tipo_documento(pool: &PgPool) -> Result<Paso, sqlx::Error> {
    let tipos = comun_repo::listar_tipos_documento(pool).await?;
    let nombres: Vec<&str> = tipos.iter().map(|t| t.nombre.as_str()).collect();
    Ok((
        r(
            "¡Gracias! ¿Con qué documento quieres registrarte?",
            &nombres,
        ),
        TIPO_DOCUMENTO,
    ))
}

async fn paso_tipo_documento(
    pool: &PgPool,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let tipos = comun_repo::listar_tipos_documento(pool).await?;
    let entrada = normalizar(texto);

    // Acepta el código ("1"), el nombre exacto ("dni") o parte del nombre ("carnet").
    let elegido = tipos.iter().find(|tipo| {
        let nombre = normalizar(&tipo.nombre);
        entrada == tipo.codigo
            || entrada == nombre
            || (entrada.len() >= 3 && nombre.contains(&entrada))
    });

    match elegido {
        Some(tipo) => {
            datos.tipo_documento = Some(tipo.codigo.clone());
            let mensaje = format!("Perfecto. Escribe tu número de {}.", tipo.nombre);
            let mut respuesta = r(&mensaje, &[]);
            // Con DNI y RUC la consulta tarda: avisamos para que el navegador muestre "Consultando…".
            if tipo.codigo == validaciones::TIPO_DNI || tipo.codigo == validaciones::TIPO_RUC {
                respuesta = respuesta.con_espera("Consultando tus datos…");
            }
            Ok((respuesta, NUMERO_DOCUMENTO))
        }
        None => {
            let nombres: Vec<&str> = tipos.iter().map(|t| t.nombre.as_str()).collect();
            Ok((
                r(
                    "No reconocí ese documento. Elige una de las opciones:",
                    &nombres,
                ),
                TIPO_DOCUMENTO,
            ))
        }
    }
}

async fn paso_numero_documento(
    pool: &PgPool,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let tipo = datos.tipo_documento.clone().unwrap_or_default();

    // 1. Revisamos que el número tenga el formato correcto para ese tipo de documento.
    let numero = match validaciones::validar_documento(&tipo, texto) {
        Ok(numero) => numero,
        Err(mensaje) => return Ok((r(&mensaje, &[]), NUMERO_DOCUMENTO)),
    };
    datos.numero_documento = Some(numero.clone());

    // 2. ¿Ya está registrado? Por seguridad NO mostramos sus datos: pedimos una verificación sencilla.
    if let Some(existente) = acceso_repo::buscar_cliente_por_documento(pool, &tipo, &numero).await?
    {
        datos.cliente_existente = Some(existente.codigo);
        datos.intentos = 0;
        return Ok((
            r(
                "Ya tengo un registro con ese documento. Por seguridad no muestro los datos guardados. \
                 Para confirmar que eres tú, escribe los últimos 3 dígitos del celular con el que te registraste.",
                &[],
            ),
            VERIFICAR_TELEFONO,
        ));
    }

    // 3. Intentamos obtener los datos automáticamente (solo DNI y RUC).
    if let Some(persona) = consulta_documento::consultar(&tipo, &numero).await {
        let tiene_nombres = persona.nombres.as_deref().map_or(false, |n| !n.is_empty());
        let tiene_razon_social = persona
            .razon_social
            .as_deref()
            .map_or(false, |n| !n.is_empty());

        // Solo seguimos por aquí si la consulta trajo un nombre o una razón social.
        if tiene_nombres || tiene_razon_social {
            // Guardamos todo "detrás de cámaras" y le mostramos la tarjeta para que confirme que son sus datos.
            datos.nombres = persona.nombres;
            datos.paterno = persona.paterno;
            datos.materno = persona.materno;
            datos.razon_social = persona.razon_social;
            datos.direccion = persona.direccion;
            datos.sexo = persona.sexo;
            datos.fecha_nacimiento = persona.fecha_nacimiento;

            let respuesta = tarjeta_de_confirmacion(&tipo, &numero, datos);
            return Ok((respuesta, CONFIRMAR_IDENTIDAD));
        }
    }

    // 4. No se pudo obtener automáticamente: la persona escribe sus datos.
    if tipo == validaciones::TIPO_RUC {
        Ok((r("No pude obtener los datos de ese RUC automáticamente. ¿Cuál es la razón social de tu empresa?", &[]), MANUAL_RAZON_SOCIAL))
    } else {
        Ok((r("No pude obtener tus datos automáticamente, así que los anotamos juntos. ¿Cuáles son tus nombres?", &[]), MANUAL_NOMBRES))
    }
}

// Arma el mensaje "¿Estos datos son correctos?" con la credencial (carnet) de datos (aún sin código de cliente).
fn tarjeta_de_confirmacion(tipo: &str, numero: &str, datos: &Datos) -> Respuesta {
    let opciones: &[&str] = if tipo == validaciones::TIPO_RUC {
        &["Sí, son correctos", "No, no es mi empresa"]
    } else {
        &["Sí, son correctos", "No, no soy yo"]
    };
    r("¿Estos datos son correctos?", opciones).con_componente(credencial(tipo, numero, datos, None))
}

// Construye la credencial (carnet) con los datos de la persona o empresa.
// - Sin "codigo": es la tarjeta de confirmación (estado "Pendiente de confirmar", sin código de barras).
// - Con "codigo": es la credencial final del cliente ya registrado. El código de barras lleva SOLO ese código
//   (nunca datos personales): quien lo escanee solo obtiene un código, y los datos se consultan en el sistema.
fn credencial(tipo: &str, numero: &str, datos: &Datos, codigo: Option<&str>) -> Componente {
    let campo = |etiqueta: &str, valor: String| Campo {
        etiqueta: etiqueta.to_string(),
        valor,
        verificacion: None,
    };
    let texto_de = |valor: &Option<String>| valor.clone().unwrap_or_default();
    let direccion = datos.direccion.as_deref().filter(|d| !d.is_empty());
    let estado = if codigo.is_some() {
        "Activo"
    } else {
        "Pendiente de confirmar"
    };

    if tipo == validaciones::TIPO_RUC {
        // Una empresa: sus datos de RUC son públicos, se muestran completos.
        let mut campos = vec![campo("RUC", numero.to_string())];
        if let Some(direccion) = direccion {
            campos.push(campo("Dirección", formato::a_titulo(direccion)));
        }
        agregar_contacto(&mut campos, datos, codigo.is_some());
        Componente::Identidad {
            variante: "empresa".to_string(),
            nombre: texto_de(&datos.razon_social), // la razón social se muestra tal cual (S.A.C., E.I.R.L....)
            subtitulo: "Empresa".to_string(),
            campos,
            estado: estado.to_string(),
            codigo: codigo.map(str::to_string),
        }
    } else {
        // Una persona: nombre y sexo completos; fecha de nacimiento y dirección según MOSTRAR_DATOS_COMPLETOS.
        let nombre_completo = format!(
            "{} {} {}",
            texto_de(&datos.nombres),
            texto_de(&datos.paterno),
            texto_de(&datos.materno)
        );
        let mut campos = vec![campo("Documento", format!("DNI {}", numero))];
        if let Some(sexo) = datos.sexo.as_deref() {
            campos.push(campo("Sexo", formato::nombre_sexo(sexo).to_string()));
        }
        if let Some(fecha) = datos.fecha_nacimiento.as_deref() {
            let valor = if MOSTRAR_DATOS_COMPLETOS {
                formato::formatear_fecha(fecha)
            } else {
                formato::fecha_enmascarada(fecha)
            };
            campos.push(campo("Nacimiento", valor));
        }
        if let Some(direccion) = direccion {
            let valor = if MOSTRAR_DATOS_COMPLETOS {
                direccion.to_string()
            } else {
                formato::direccion_enmascarada(direccion)
            };
            campos.push(campo("Dirección", formato::a_titulo(&valor)));
        }
        agregar_contacto(&mut campos, datos, codigo.is_some());
        Componente::Identidad {
            variante: "persona".to_string(),
            nombre: formato::a_titulo(&nombre_completo),
            subtitulo: "Persona natural".to_string(),
            campos,
            estado: estado.to_string(),
            codigo: codigo.map(str::to_string),
        }
    }
}

// Agrega a la credencial final el celular (con su estado de verificación) y el correo (solo si lo dio).
// En la tarjeta de confirmación todavía no existen, así que no se agregan.
fn agregar_contacto(campos: &mut Vec<Campo>, datos: &Datos, es_credencial_final: bool) {
    if !es_credencial_final {
        return;
    }
    if let Some(telefono) = datos.telefono.as_deref().filter(|t| !t.is_empty()) {
        campos.push(Campo {
            etiqueta: "Celular".to_string(),
            valor: formato::formatear_telefono(telefono),
            // Todavía no enviamos SMS de confirmación: el celular queda "pendiente" hasta que exista ese paso.
            verificacion: Some(Verificacion::Pendiente),
        });
    }
    if let Some(correo) = datos.correo.as_deref().filter(|c| !c.is_empty()) {
        campos.push(Campo {
            etiqueta: "Correo".to_string(),
            valor: correo.to_string(),
            verificacion: None,
        });
    }
}

// ---------- Registro: verificación de un cliente que ya existía ----------

async fn paso_verificar_telefono(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let digitos: String = texto.chars().filter(|c| c.is_ascii_digit()).collect();
    let codigo_cliente = datos.cliente_existente.clone().unwrap_or_default();
    let telefono = acceso_repo::obtener_telefono(pool, &codigo_cliente)
        .await?
        .unwrap_or_default();

    if digitos.len() == 3 && telefono.ends_with(&digitos) {
        // Coinciden: es él. Vinculamos esta conversación a su cliente (sin mostrar ningún dato).
        repositorio::vincular_cliente(pool, &conversacion.codigo, &codigo_cliente, None).await?;
        *datos = Datos::default();
        Ok((
            r(
                "¡Perfecto, te reconozco! Bienvenido de vuelta. ¿En qué te puedo ayudar?",
                &MENU,
            ),
            LIBRE,
        ))
    } else {
        datos.intentos += 1;
        if datos.intentos >= 3 {
            // Demasiados intentos: cerramos la verificación por seguridad.
            *datos = Datos::default();
            Ok((
                r(
                    "No pude verificar tu identidad, así que por seguridad no puedo continuar con ese documento. \
                     Comunícate directamente con H&Devs para que te ayuden. Mientras tanto, puedes preguntarme por nuestros servicios.",
                    &MENU,
                ),
                LIBRE,
            ))
        } else {
            Ok((r("Esos dígitos no coinciden. Inténtalo otra vez: escribe los últimos 3 dígitos de tu celular registrado.", &[]), VERIFICAR_TELEFONO))
        }
    }
}

// ---------- Registro: confirmación de identidad ----------

async fn paso_confirmar_identidad(
    pool: &PgPool,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    if es_negativo(texto) {
        // No es él: borramos lo obtenido (conservamos solo el consentimiento) y volvemos a preguntar el documento.
        *datos = Datos {
            consentimiento: datos.consentimiento.take(),
            ..Default::default()
        };
        pedir_tipo_documento(pool).await
    } else if es_afirmativo(texto) {
        Ok(siguiente_tras_identidad(datos))
    } else {
        Ok((
            r(
                "¿Me confirmas si esos datos son correctos? Responde sí o no.",
                &["Sí, son correctos", "No, no soy yo"],
            ),
            CONFIRMAR_IDENTIDAD,
        ))
    }
}

// Después de confirmar la identidad: pedimos la dirección solo si la consulta no la trajo.
fn siguiente_tras_identidad(datos: &Datos) -> Paso {
    let falta_direccion = datos
        .direccion
        .as_deref()
        .map_or(true, |d| d.trim().is_empty());
    if falta_direccion {
        (r("¿Cuál es tu dirección?", &[]), DIRECCION)
    } else {
        pedir_telefono()
    }
}

// ---------- Registro: datos escritos a mano ----------

fn paso_manual_nombres(datos: &mut Datos, texto: &str) -> Paso {
    match validaciones::limpiar_nombre(texto) {
        Some(nombres) => {
            datos.nombres = Some(nombres);
            (r("¿Y tu apellido paterno?", &[]), MANUAL_PATERNO)
        }
        None => (
            r("Escribe tus nombres usando solo letras, por favor.", &[]),
            MANUAL_NOMBRES,
        ),
    }
}

fn paso_manual_paterno(datos: &mut Datos, texto: &str) -> Paso {
    match validaciones::limpiar_nombre(texto) {
        Some(paterno) => {
            datos.paterno = Some(paterno);
            (
                r(
                    "¿Y tu apellido materno? Si no tienes, escribe «no tengo».",
                    &["No tengo"],
                ),
                MANUAL_MATERNO,
            )
        }
        None => (
            r(
                "Escribe tu apellido paterno usando solo letras, por favor.",
                &[],
            ),
            MANUAL_PATERNO,
        ),
    }
}

fn paso_manual_materno(datos: &mut Datos, texto: &str) -> Paso {
    if es_negativo(texto) || texto.trim() == "-" {
        datos.materno = None; // no tiene apellido materno
    } else if let Some(materno) = validaciones::limpiar_nombre(texto) {
        datos.materno = Some(materno);
    } else {
        return (
            r(
                "Escribe tu apellido materno usando solo letras, o pulsa «No tengo».",
                &["No tengo"],
            ),
            MANUAL_MATERNO,
        );
    }
    (r("¿Cuál es tu dirección?", &[]), DIRECCION)
}

fn paso_manual_razon_social(datos: &mut Datos, texto: &str) -> Paso {
    let razon = texto.split_whitespace().collect::<Vec<_>>().join(" "); // quita espacios repetidos
    if razon.chars().count() < 3 || razon.chars().count() > 200 {
        return (
            r(
                "Escribe la razón social completa (entre 3 y 200 caracteres).",
                &[],
            ),
            MANUAL_RAZON_SOCIAL,
        );
    }
    datos.razon_social = Some(razon.to_uppercase());
    (r("¿Cuál es la dirección de la empresa?", &[]), DIRECCION)
}

// ---------- Registro: dirección, teléfono y correo ----------

fn paso_direccion(datos: &mut Datos, texto: &str) -> Paso {
    let direccion = texto.split_whitespace().collect::<Vec<_>>().join(" ");
    let largo = direccion.chars().count();
    if !(5..=120).contains(&largo) {
        return (
            r(
                "Escribe una dirección de entre 5 y 120 caracteres, por favor.",
                &[],
            ),
            DIRECCION,
        );
    }
    datos.direccion = Some(direccion.to_uppercase());
    pedir_telefono()
}

fn pedir_telefono() -> Paso {
    (
        r(
            "¿Cuál es tu número de celular? Lo usaremos para coordinar tus citas.",
            &[],
        ),
        TELEFONO,
    )
}

fn paso_telefono(datos: &mut Datos, texto: &str) -> Paso {
    match validaciones::normalizar_telefono(texto) {
        Some(telefono) => {
            datos.telefono = Some(telefono);
            (
                r("Gracias. ¿Cuál es tu correo electrónico? Es opcional: si no quieres darlo, pulsa «Omitir».", &["Omitir"]),
                CORREO,
            )
        }
        None => (r("El celular debe tener 9 números y empezar con 9 (por ejemplo, 987654321). Inténtalo de nuevo.", &[]), TELEFONO),
    }
}

async fn paso_correo(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let omitir = es_negativo(texto) || normalizar(texto).trim() == "omitir";

    if omitir {
        datos.correo = None;
    } else if validaciones::correo_valido(texto) {
        datos.correo = Some(texto.trim().to_lowercase());
    } else {
        return Ok((
            r(
                "Ese correo no parece válido. Escríbelo de nuevo o pulsa «Omitir».",
                &["Omitir"],
            ),
            CORREO,
        ));
    }

    finalizar_registro(pool, conversacion, datos).await
}

// ---------- Registro: guardar al cliente ----------

// Con todos los datos recogidos, crea el cliente en Acceso.Clientes y vincula la conversación.
async fn finalizar_registro(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
) -> Result<Paso, sqlx::Error> {
    let nuevo = NuevoCliente {
        tipo_documento: datos.tipo_documento.clone().unwrap_or_default(),
        numero_documento: datos.numero_documento.clone().unwrap_or_default(),
        paterno: datos.paterno.clone(),
        materno: datos.materno.clone(),
        nombres: datos.nombres.clone(),
        razon_social: datos.razon_social.clone(),
        direccion: datos.direccion.clone().unwrap_or_default(),
        sexo: datos.sexo.clone(),
        fecha_nacimiento: datos.fecha_nacimiento.clone(),
        telefono: datos.telefono.clone().unwrap_or_default(),
        correo: datos.correo.clone(),
        consentimiento: datos.consentimiento.clone().unwrap_or_default(),
    };

    let codigo = acceso_repo::crear_cliente(pool, &nuevo).await?;
    repositorio::vincular_cliente(pool, &conversacion.codigo, &codigo, Some("registro")).await?;

    // Nombre para saludar: el primer nombre, o la razón social si es una empresa.
    let nombre = match datos
        .nombres
        .as_deref()
        .and_then(|n| n.split_whitespace().next())
    {
        Some(primer_nombre) => formato::a_titulo(primer_nombre),
        None => datos.razon_social.clone().unwrap_or_default(),
    };

    // Si omitió el correo, le ofrecemos agregarlo después con un botón.
    let sin_correo = datos.correo.is_none();

    // La credencial final se arma ahora, antes de borrar los datos temporales (ya con el código de cliente).
    let tipo = datos.tipo_documento.clone().unwrap_or_default();
    let numero = datos.numero_documento.clone().unwrap_or_default();
    let tarjeta = credencial(&tipo, &numero, datos, Some(codigo.as_str()));

    // Los datos temporales ya cumplieron su función: se borran (los definitivos están en Acceso.Clientes).
    *datos = Datos::default();

    let mensaje = format!(
        "¡Listo, {}! Ya quedaste registrado. Tu código de cliente es {}. ¿En qué te puedo ayudar con el servicio técnico?",
        nombre, codigo
    );
    let opciones: &[&str] = if sin_correo {
        &["Ver servicios", "Agendar una cita", "Agregar mi correo"]
    } else {
        &MENU
    };
    Ok((r(&mensaje, opciones).con_componente(tarjeta), LIBRE))
}
