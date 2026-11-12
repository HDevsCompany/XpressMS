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
//   libre -> tipo_documento -> numero_documento -> verificar_telefono -> libre
//            («Ya soy cliente»: solo se reconoce al cliente por su documento; si no existe, ofrece registrarlo)
//   libre -> agregar_correo -> libre (un cliente que omitió su correo lo agrega después)
//   libre -> elegir_servicio -> elegir_equipo -> libre (la consulta de servicios vive en flujo_servicios.rs)
//   libre -> cita_fecha -> cita_hora -> cita_problema -> cita_confirmar -> libre (citas y seguimiento: flujo_citas.rs)
//   libre -> elegir_categoria -> elegir_producto -> libre -> apartar_cantidad -> apartar_confirmar -> libre
//            (productos y apartados: flujo_productos.rs)
//
// Los nombres de los pasos están en pasos.rs y las piezas para armar respuestas en respuesta.rs.
//
// Regla: este archivo NO escribe SQL. Para leer o guardar datos llama a los repositorios de los
// módulos comun, acceso y agente.

use sqlx::PgPool;

use super::flujo_carrito;
use super::flujo_catalogo;
use super::flujo_citas;
use super::flujo_perfil;
use super::flujo_productos;
use super::flujo_servicios;
use super::flujo_solicitud;
use super::identidad;
use super::modelo::{Campo, Componente, Conversacion, Datos, Verificacion};
use super::motor::{detectar_intencion, es_afirmativo, es_negativo, normalizar, Intencion};
use super::pasos::{
    AGREGAR_CORREO, APARTAR_CANTIDAD, APARTAR_CONFIRMAR, CANCELAR_APARTADO, CANCELAR_CITA,
    CITA_CONFIRMAR, CITA_FECHA, CITA_HORA, CITA_PROBLEMA, CONFIRMAR_IDENTIDAD, CONSENTIMIENTO,
    CORREO, DIR_ACCION, DIR_CUAL, DIR_NOMBRE, DIR_PRINCIPAL, DIR_QUITAR_CONFIRMAR, DIR_REFERENCIA, DIR_TEXTO,
    DIR_TIPO, DIRECCION, CATALOGO, ELEGIR_APARTADO, ELEGIR_CATEGORIA, ELEGIR_EQUIPO, ELEGIR_ORDEN,
    ELEGIR_PRODUCTO, ELEGIR_SERVICIO, LIBRE, MANUAL_MATERNO, MANUAL_NOMBRES, MANUAL_PATERNO,
    MANUAL_RAZON_SOCIAL, NACIMIENTO, NUMERO_DOCUMENTO, PER_ACCION, PER_CODIGO, PER_CONFIRMAR, PER_DOCUMENTO,
    PER_NOMBRE, PER_PARENTESCO, PER_QUITAR_CONFIRMAR, PER_QUITAR_CUAL, SEXO, TELEFONO, TIPO_DOCUMENTO, VERIFICAR_CELULAR, VERIFICAR_TELEFONO,
};
use super::repositorio;
use super::respuesta::{r, Paso, Respuesta, MENU};
use crate::modulos::acceso::modelo::{ClienteCompleto, NuevoCliente};
use crate::modulos::acceso::codigo_otp;
use crate::modulos::acceso::verificacion_telefono::{self, Resultado as ResultadoCodigo, Solicitud};
use crate::modulos::acceso::{consulta_documento, repositorio as acceso_repo};
use crate::modulos::comun::{fechas, formato, repositorio as comun_repo, validaciones};

// Privacidad: ¿la tarjeta de confirmación muestra los datos de una persona completos?
// Con "false" (recomendado) la fecha de nacimiento y la dirección salen parcialmente ocultas
// ("••/••/1995", "•••••• - TRUJILLO"), porque cualquiera que conozca un DNI podría escribirlo en el chat
// y ver esos datos. Cambia a "true" solo si el negocio lo decide de forma consciente.
const MOSTRAR_DATOS_COMPLETOS: bool = false;

// ---------- Entrada al chat ----------

// Por dónde llegó la persona al chat. Lo decide la página principal (la tarjeta de ingreso), que ya le preguntó
// qué quería; así el saludo de Juno corresponde a lo que la persona acaba de elegir.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Entrada {
    Registro, // pulsó «Regístrate con Juno»
    Mirar,    // pulsó «Ver servicios y productos», o abrió el chat directamente
}

impl Entrada {
    // El navegador la envía como texto ("registro"). Cualquier otro valor (o ninguno) es «Mirar».
    pub fn desde_texto(texto: Option<&str>) -> Self {
        match texto {
            Some("registro") => Entrada::Registro,
            _ => Entrada::Mirar,
        }
    }
}

// El saludo de Juno según la entrada, y el paso en que queda la conversación.
// Quien se registra ve directamente la casilla de términos; quien solo mira ve los botones del menú.
pub fn saludo_de_entrada(entrada: Entrada) -> Paso {
    match entrada {
        Entrada::Registro => (
            r(
                &format!(
                    "¡Hola! Qué bueno que te animes a registrarte 👋 Soy {} y te acompaño. Antes de empezar, revisa los términos y acepta si estás de acuerdo.",
                    identidad::NOMBRE_PREDETERMINADO
                ),
                &[],
            )
            .con_componente(Componente::Consentimiento),
            CONSENTIMIENTO,
        ),
        Entrada::Mirar => (
            r(
                &format!(
                    "¡Hola! Soy {}, tu asistente 👋 Mira con calma; si algo te interesa, te ayudo a agendar una cita o apartar un producto. ¿Por dónde empezamos?",
                    identidad::NOMBRE_PREDETERMINADO
                ),
                &MENU,
            ),
            LIBRE,
        ),
    }
}

// Deja lista la conversación al abrir el chat: guarda el saludo que corresponde a la entrada.
// Quien viene a registrarse recibe su saludo (con la casilla de términos): una conversación NUEVA siempre lo recibe y una
// que ya existía solo si todavía no es cliente y no está a medias de otra cosa (así no se repite ni se interrumpe nada).
// Quien solo mira no recibe ninguno: el lienzo ya lo saluda.
pub async fn dar_bienvenida(
    pool: &PgPool,
    conversacion: &mut Conversacion,
    entrada: Entrada,
    es_nueva: bool,
) -> Result<(), sqlx::Error> {
    let sin_tarea = conversacion.paso == LIBRE || conversacion.paso == "inicio";
    let corresponde =
        es_nueva || (entrada == Entrada::Registro && conversacion.cliente.is_none() && sin_tarea);
    if !corresponde {
        return Ok(());
    }

    // Quien solo mira no recibe un mensaje escrito: el lienzo (Juno al centro, «Hola, soy Juno» y las píldoras) ya es
    // su saludo, y la conversación empieza vacía. Solo quien viene a registrarse recibe el saludo con los términos.
    if entrada == Entrada::Mirar {
        return Ok(());
    }

    let (saludo, paso) = saludo_de_entrada(entrada);
    repositorio::guardar_mensaje(pool, &conversacion.codigo, "A", &saludo.texto).await?;
    if paso != LIBRE {
        repositorio::guardar_estado(pool, &conversacion.codigo, paso, &Datos::default()).await?;
        conversacion.paso = paso.to_string();
    }
    Ok(())
}

// Deja lista la conversación de la ventana de Juno en la tienda (/catalogo): NO guarda ningún saludo (la ventana ya lo trae)
// y la marca para que lo que la persona escriba allí filtre la vitrina. Es una conversación aparte del chat principal.
pub async fn preparar_catalogo(pool: &PgPool, conversacion: &mut Conversacion) -> Result<(), sqlx::Error> {
    let mut datos: Datos = serde_json::from_str(&conversacion.datos).unwrap_or_default();
    // La página siempre se abre mostrando todo el catálogo: los filtros que Juno tuviera guardados de una visita anterior se
    // olvidan, para que lo que se ve en la vitrina y lo que Juno tiene aplicado sean siempre lo mismo.
    let hay_filtros = datos.catalogo_categoria.is_some() || datos.catalogo_precio_max.is_some() || datos.catalogo_busqueda.is_some();
    if datos.en_catalogo && !hay_filtros {
        return Ok(());
    }
    datos.en_catalogo = true;
    datos.catalogo_categoria = None;
    datos.catalogo_precio_max = None;
    datos.catalogo_busqueda = None;
    let paso = if conversacion.paso == "inicio" { LIBRE.to_string() } else { conversacion.paso.clone() };
    repositorio::guardar_estado(pool, &conversacion.codigo, &paso, &datos).await?;
    conversacion.paso = paso;
    conversacion.datos = serde_json::to_string(&datos).unwrap_or_else(|_| "{}".to_string());
    Ok(())
}

// Deja lista la conversación de la ventana de Juno en /servicios: tampoco guarda saludo y la marca para que cada respuesta lleve el
// estado de «Tu solicitud». Es una conversación aparte de la del chat principal y de la tienda. Si la persona recarga la página en
// medio de una solicitud, esta se conserva (la página la vuelve a dibujar con flujo_solicitud::reanudar).
pub async fn preparar_servicios(pool: &PgPool, conversacion: &mut Conversacion) -> Result<(), sqlx::Error> {
    let mut datos: Datos = serde_json::from_str(&conversacion.datos).unwrap_or_default();
    if datos.en_servicios && !datos.en_catalogo {
        return Ok(());
    }
    datos.en_servicios = true;
    datos.en_catalogo = false;
    let paso = if conversacion.paso == "inicio" { LIBRE.to_string() } else { conversacion.paso.clone() };
    repositorio::guardar_estado(pool, &conversacion.codigo, &paso, &datos).await?;
    conversacion.paso = paso;
    conversacion.datos = serde_json::to_string(&datos).unwrap_or_else(|_| "{}".to_string());
    Ok(())
}

// Si la persona recarga la página en medio del registro, hay que volver a dibujar la casilla de términos.
pub fn componente_de_reanudacion(paso: &str) -> Option<Componente> {
    if paso == CONSENTIMIENTO {
        Some(Componente::Consentimiento)
    } else {
        None
    }
}

// Los botones del menú se muestran solo cuando la conversación recién empieza (solo existe el saludo).
pub fn opciones_de_reanudacion(paso: &str, cantidad_mensajes: usize) -> Vec<String> {
    if cantidad_mensajes <= 1 && (paso == LIBRE || paso == "inicio") {
        MENU.iter().map(|o| o.to_string()).collect()
    } else if paso == CATALOGO {
        flujo_catalogo::opciones_de_reanudacion()
    } else if paso == SEXO {
        OPCIONES_SEXO.iter().map(|o| o.to_string()).collect()
    } else if paso == NACIMIENTO {
        vec!["Omitir".to_string()]
    } else if paso == VERIFICAR_CELULAR {
        // Si recarga la página esperando el SMS, los botones de ayuda tienen que seguir ahí.
        OPCIONES_CODIGO.iter().map(|o| o.to_string()).collect()
    } else {
        // Los pasos de «Mi perfil» (direcciones y personas autorizadas) tienen sus propios botones.
        flujo_perfil::opciones_de_reanudacion(paso)
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
    red: &str, // dirección de internet de quien escribe (límite de códigos SMS por red)
) -> Result<Respuesta, sqlx::Error> {
    // Recupera lo recogido hasta ahora (si el JSON estuviera dañado, empieza de cero).
    let mut datos: Datos = serde_json::from_str(&conversacion.datos).unwrap_or_default();

    // Las órdenes exactas de /servicios («Solicitar servicio: …», «Agendar cita de diagnóstico», «Cancelar solicitud»…): se atienden
    // cuando la persona está libre o en medio de una cita, no en medio de un registro.
    let tenia_solicitud = datos.solicitud.is_some();
    if flujo_solicitud::puede_atender(&conversacion.paso) {
        if let Some((mut respuesta, nuevo_paso)) = flujo_solicitud::atender(pool, conversacion, &mut datos, texto).await? {
            respuesta.solicitud = flujo_solicitud::estado(
                pool,
                &mut datos,
                &conversacion.paso,
                nuevo_paso,
                respuesta.componente.as_ref(),
                tenia_solicitud,
            )
            .await?;
            repositorio::guardar_estado(pool, &conversacion.codigo, nuevo_paso, &datos).await?;
            return Ok(respuesta);
        }
    }

    // El carrito de la tienda: sus órdenes exactas («Añadir al carrito: …», «Ir a pagar»…) solo se atienden cuando la persona no está
    // en medio de otra cosa (un registro, una cita...), para no interrumpirla. El carrito queda guardado en la conversación.
    if matches!(conversacion.paso.as_str(), LIBRE | "inicio" | CATALOGO) {
        if let Some((respuesta, nuevo_paso)) = flujo_carrito::atender(pool, conversacion, &mut datos, texto).await? {
            repositorio::guardar_estado(pool, &conversacion.codigo, nuevo_paso, &datos).await?;
            return Ok(respuesta);
        }
    }

    // Cada paso tiene su propia función. Los pasos "inicio" y "libre" usan paso_libre.
    let (mut respuesta, nuevo_paso) = match conversacion.paso.as_str() {
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
        SEXO => paso_sexo(&mut datos, texto),
        NACIMIENTO => paso_nacimiento(&mut datos, texto),
        TELEFONO => paso_telefono(pool, conversacion, &mut datos, texto, red).await?,
        VERIFICAR_CELULAR => paso_verificar_celular(pool, conversacion, &mut datos, texto, red).await?,
        CORREO => paso_correo(pool, conversacion, &mut datos, texto).await?,
        AGREGAR_CORREO => paso_agregar_correo(pool, conversacion, texto).await?,
        ELEGIR_SERVICIO => flujo_servicios::paso_elegir_servicio(pool, &mut datos, texto).await?,
        ELEGIR_EQUIPO => flujo_servicios::paso_elegir_equipo(pool, &mut datos, texto).await?,
        CITA_FECHA => flujo_citas::paso_fecha(pool, &mut datos, texto).await?,
        CITA_HORA => flujo_citas::paso_hora(pool, &mut datos, texto).await?,
        CITA_PROBLEMA => flujo_citas::paso_problema(pool, &mut datos, texto).await?,
        CITA_CONFIRMAR => flujo_citas::paso_confirmar(pool, conversacion, &mut datos, texto).await?,
        ELEGIR_ORDEN => flujo_citas::paso_elegir_orden(pool, conversacion, &mut datos, texto).await?,
        CANCELAR_CITA => flujo_citas::paso_cancelar(pool, conversacion, &mut datos, texto).await?,
        // La vitrina del catálogo: si lo escrito no es para ella (una cita, un servicio...), lo atiende la conversación libre.
        CATALOGO => match flujo_catalogo::paso_catalogo(pool, conversacion, &mut datos, texto).await? {
            Some(paso) => paso,
            None => paso_libre(pool, conversacion, &mut datos, texto).await?,
        },
        ELEGIR_CATEGORIA => flujo_productos::paso_elegir_categoria(pool, &mut datos, texto).await?,
        ELEGIR_PRODUCTO => flujo_productos::paso_elegir_producto(pool, &mut datos, texto).await?,
        APARTAR_CANTIDAD => flujo_productos::paso_cantidad(pool, &mut datos, texto).await?,
        APARTAR_CONFIRMAR => {
            flujo_productos::paso_confirmar(pool, conversacion, &mut datos, texto).await?
        }
        ELEGIR_APARTADO => {
            flujo_productos::paso_elegir_apartado(pool, conversacion, &mut datos, texto).await?
        }
        CANCELAR_APARTADO => {
            flujo_productos::paso_cancelar(pool, conversacion, &mut datos, texto).await?
        }
        DIR_ACCION => flujo_perfil::paso_dir_accion(pool, conversacion, &mut datos, texto).await?,
        DIR_CUAL => flujo_perfil::paso_dir_cual(pool, conversacion, &mut datos, texto).await?,
        DIR_QUITAR_CONFIRMAR => {
            flujo_perfil::paso_dir_quitar_confirmar(pool, conversacion, &mut datos, texto).await?
        }
        DIR_TIPO => flujo_perfil::paso_dir_tipo(&mut datos, texto),
        DIR_NOMBRE => flujo_perfil::paso_dir_nombre(&mut datos, texto),
        DIR_TEXTO => flujo_perfil::paso_dir_texto(&mut datos, texto),
        DIR_REFERENCIA => flujo_perfil::paso_dir_referencia(pool, conversacion, &mut datos, texto).await?,
        DIR_PRINCIPAL => flujo_perfil::paso_dir_principal(pool, conversacion, &mut datos, texto).await?,
        PER_ACCION => flujo_perfil::paso_per_accion(pool, conversacion, &mut datos, texto).await?,
        PER_NOMBRE => flujo_perfil::paso_per_nombre(&mut datos, texto),
        PER_DOCUMENTO => flujo_perfil::paso_per_documento(pool, conversacion, &mut datos, texto).await?,
        PER_PARENTESCO => flujo_perfil::paso_per_parentesco(pool, conversacion, &mut datos, texto).await?,
        PER_CONFIRMAR => flujo_perfil::paso_per_confirmar(pool, conversacion, &mut datos, texto, red).await?,
        PER_CODIGO => flujo_perfil::paso_per_codigo(pool, conversacion, &mut datos, texto, red).await?,
        PER_QUITAR_CUAL => flujo_perfil::paso_per_quitar_cual(pool, conversacion, &mut datos, texto).await?,
        PER_QUITAR_CONFIRMAR => {
            flujo_perfil::paso_per_quitar_confirmar(pool, conversacion, &mut datos, texto).await?
        }
        // La ventana de Juno en la tienda: lo que escribe la persona afina la vitrina; si no es para ella (una cita, un servicio...),
        // lo atiende la conversación libre.
        _ if datos.en_catalogo => {
            match flujo_catalogo::paso_catalogo(pool, conversacion, &mut datos, texto).await? {
                Some(paso) => paso,
                None => paso_libre(pool, conversacion, &mut datos, texto).await?,
            }
        }
        _ => paso_libre(pool, conversacion, &mut datos, texto).await?,
    };

    // En la ventana de /servicios, cada respuesta lleva el estado de «Tu solicitud» (en qué paso va y lo que ya se anotó).
    respuesta.solicitud = flujo_solicitud::estado(
        pool,
        &mut datos,
        &conversacion.paso,
        nuevo_paso,
        respuesta.componente.as_ref(),
        tenia_solicitud,
    )
    .await?;
    repositorio::guardar_estado(pool, &conversacion.codigo, nuevo_paso, &datos).await?;
    Ok(respuesta)
}

// ---------- Conversación libre ----------

// La persona pregunta lo que quiera. Si pide registrarse, empieza el registro.
async fn paso_libre(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
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
        Intencion::YaSoyCliente => {
            if conversacion.cliente.is_some() {
                (
                    r("¡Ya te tengo identificado! ¿En qué te puedo ayudar?", &MENU),
                    LIBRE,
                )
            } else {
                // No pide aceptar términos: solo se busca un registro que ya existe, no se guarda nada nuevo.
                datos.solo_identificar = true;
                return pedir_tipo_documento_de_cliente(pool).await;
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
        Intencion::AgendarCita => return flujo_citas::iniciar(pool, conversacion, datos).await,
        Intencion::MiOrden => return flujo_citas::ver_ordenes(pool, conversacion, datos).await,
        Intencion::CancelarCita => {
            return flujo_citas::pedir_cancelacion(pool, conversacion, datos).await
        }
        Intencion::Productos => return flujo_productos::mostrar_categorias(pool, datos).await,
        Intencion::Apartar => {
            // «Apartar Mouse M170»: si la frase nombra el producto, es ese el que se aparta.
            flujo_catalogo::fijar_producto_nombrado(pool, datos, texto).await?;
            return flujo_productos::iniciar_apartado(pool, conversacion, datos).await
        }
        Intencion::MiApartado => return flujo_productos::ver_apartados(pool, conversacion, datos).await,
        Intencion::CancelarApartado => {
            return flujo_productos::pedir_cancelacion(pool, conversacion, datos).await
        }
        Intencion::Direcciones => return flujo_perfil::iniciar_direcciones(pool, conversacion, datos).await,
        Intencion::Personas => return flujo_perfil::iniciar_personas(pool, conversacion, datos).await,
        Intencion::CambiarCelular => flujo_perfil::cambiar_celular(conversacion),
        Intencion::CorregirDatos => flujo_perfil::corregir_datos(conversacion),
        Intencion::PedirDatos => flujo_perfil::pedir_datos(conversacion),
        Intencion::EliminarCuenta => flujo_perfil::eliminar_cuenta(conversacion),
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
            r("Para guardar tu correo primero necesito registrarte. ¿Quieres que lo hagamos ahora?", &["Registrarme", "Solo estoy de paso"]),
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
                &["Registrarme"],
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

// «Ya soy cliente»: pregunta con qué documento se registró. Lleva «Cancelar» por si no lo tiene a la mano.
async fn pedir_tipo_documento_de_cliente(pool: &PgPool) -> Result<Paso, sqlx::Error> {
    let tipos = comun_repo::listar_tipos_documento(pool).await?;
    let mut opciones: Vec<&str> = tipos.iter().map(|t| t.nombre.as_str()).collect();
    opciones.push("Cancelar");
    Ok((
        r(
            "¡Perfecto! Para reconocerte, ¿con qué documento te registraste?",
            &opciones,
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

    // Salida para quien eligió «Ya soy cliente» y no tiene el documento a la mano.
    if entrada == "cancelar" {
        datos.reiniciar();
        return Ok((
            r("Sin problema. Puedes preguntarme lo que quieras.", &MENU),
            LIBRE,
        ));
    }

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
            let mut nombres: Vec<&str> = tipos.iter().map(|t| t.nombre.as_str()).collect();
            if datos.solo_identificar {
                nombres.push("Cancelar");
            }
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

    // 2b. «Ya soy cliente» pero ese documento no está registrado: no consultamos ningún servicio externo
    //     ni guardamos nada; le ofrecemos corregir el dato, registrarse o seguir de paso.
    if datos.solo_identificar {
        datos.reiniciar();
        return Ok((
            r(
                "No encontré ningún registro con ese documento. Puedes revisar el número e intentarlo otra vez, \
                 registrarte ahora o seguir de paso.",
                &["Ya soy cliente", "Registrarme", "Solo estoy de paso"],
            ),
            LIBRE,
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
            // Con el código por SMS el celular queda verificado (el reloj "pendiente" es solo por si algún día falta).
            verificacion: Some(if datos.telefono_verificado {
                Verificacion::Verificado
            } else {
                Verificacion::Pendiente
            }),
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
        datos.reiniciar();
        // Si venía pidiendo un servicio técnico en /servicios, se sigue con esa solicitud donde se quedó.
        if datos.solicitud.is_some() {
            return flujo_solicitud::reanudar_tras_identificar(
                pool,
                conversacion,
                &codigo_cliente,
                datos,
                "¡Perfecto, te reconozco! Bienvenido de vuelta.",
            )
            .await;
        }
        // Si venía armando un pedido en la tienda, lo que sigue es ese pedido (el carrito se quedó guardado).
        if !datos.carrito.is_empty() {
            return Ok((
                r("¡Perfecto, te reconozco! Bienvenido de vuelta. Tu carrito sigue guardado: ¿vamos a pagar?", &["Ir a pagar", "Seguir viendo"]),
                if datos.en_catalogo { CATALOGO } else { LIBRE },
            ));
        }
        Ok((
            r(
                "¡Perfecto, te reconozco! Bienvenido de vuelta. ¿En qué te puedo ayudar?",
                &["Ver servicios", "Agendar una cita", "Ver mi orden"],
            ),
            LIBRE,
        ))
    } else {
        datos.intentos += 1;
        if datos.intentos >= 3 {
            // Demasiados intentos: cerramos la verificación por seguridad.
            datos.reiniciar();
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
            carrito: std::mem::take(&mut datos.carrito),
            en_catalogo: datos.en_catalogo,
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
        siguiente_tras_direccion(datos)
    }
}

// Opciones del paso del sexo.
const OPCIONES_SEXO: [&str; 3] = ["Masculino", "Femenino", "Prefiero no decirlo"];

// Una empresa (RUC) no tiene sexo ni fecha de nacimiento; todo lo demás es una persona.
fn es_persona(datos: &Datos) -> bool {
    datos.tipo_documento.as_deref() != Some(validaciones::TIPO_RUC)
}

// Después de la dirección: si es una persona y la consulta del documento NO trajo su sexo o su fecha de nacimiento,
// se los pedimos (son opcionales). Si ya los trajo, o es una empresa, se sigue con el celular.
fn siguiente_tras_direccion(datos: &Datos) -> Paso {
    if es_persona(datos) && datos.sexo.is_none() {
        (
            r("¿Cuál es tu sexo? Es opcional: si prefieres no decirlo, pulsa «Prefiero no decirlo».", &OPCIONES_SEXO),
            SEXO,
        )
    } else {
        siguiente_tras_sexo(datos)
    }
}

fn siguiente_tras_sexo(datos: &Datos) -> Paso {
    if es_persona(datos) && datos.fecha_nacimiento.is_none() {
        (
            r("¿Cuál es tu fecha de nacimiento? Es opcional. Escríbela así: 15/03/1995.", &["Omitir"]),
            NACIMIENTO,
        )
    } else {
        pedir_telefono()
    }
}

fn paso_sexo(datos: &mut Datos, texto: &str) -> Paso {
    let t = normalizar(texto);
    let t = t.trim();
    match t {
        "m" | "masculino" | "hombre" | "varon" => datos.sexo = Some("M".to_string()),
        "f" | "femenino" | "mujer" => datos.sexo = Some("F".to_string()),
        _ if t.contains("prefiero") || t == "omitir" || es_negativo(t) => datos.sexo = None, // no quiso decirlo
        _ => {
            return (
                r("Elige una opción con los botones: Masculino, Femenino o «Prefiero no decirlo».", &OPCIONES_SEXO),
                SEXO,
            )
        }
    }
    siguiente_tras_sexo(datos)
}

fn paso_nacimiento(datos: &mut Datos, texto: &str) -> Paso {
    let t = normalizar(texto);
    let omitir = t.trim() == "omitir" || t.contains("prefiero") || es_negativo(&t);
    if omitir {
        datos.fecha_nacimiento = None;
    } else {
        let hoy = fechas::iso(fechas::ahora_lima().fecha);
        match validaciones::fecha_nacimiento_valida(texto, &hoy) {
            Some(fecha) => datos.fecha_nacimiento = Some(fecha),
            None => {
                return (
                    r(
                        "No pude entender esa fecha. Escríbela con día, mes y año, por ejemplo 15/03/1995, o pulsa «Omitir».",
                        &["Omitir"],
                    ),
                    NACIMIENTO,
                )
            }
        }
    }
    pedir_telefono()
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
    siguiente_tras_direccion(datos)
}

fn pedir_telefono() -> Paso {
    (
        r(
            "¿Cuál es tu número de celular? Te enviaremos un código por SMS para confirmar que es tuyo; también lo usaremos para coordinar tus citas.",
            &[],
        )
        .con_espera("Enviando el código…"),
        TELEFONO,
    )
}

// Botones que acompañan al paso del código.
const OPCIONES_CODIGO: [&str; 2] = ["Reenviar código", "Cambiar número"];

// La persona escribe su celular: si es válido, se le envía el código por SMS y se espera que lo escriba.
async fn paso_telefono(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
    red: &str,
) -> Result<Paso, sqlx::Error> {
    match validaciones::normalizar_telefono(texto) {
        Some(telefono) => {
            datos.telefono = Some(telefono);
            datos.telefono_verificado = false; // un número nuevo siempre hay que verificarlo
            enviar_codigo(pool, conversacion, datos, red).await
        }
        None => Ok((r("El celular debe tener 9 números y empezar con 9 (por ejemplo, 987654321). Inténtalo de nuevo.", &[]), TELEFONO)),
    }
}

// Pide el código para el celular que ya está en los datos y le cuenta a la persona qué pasó.
async fn enviar_codigo(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &Datos,
    red: &str,
) -> Result<Paso, sqlx::Error> {
    let telefono = datos.telefono.clone().unwrap_or_default();

    let texto = match verificacion_telefono::solicitar_codigo(pool, &conversacion.codigo, &telefono, red).await? {
        Solicitud::Enviado => format!(
            "Te enviamos un código de 6 dígitos por SMS al {}. Escríbelo aquí; vale {} minutos. Si no te llega, pulsa «Reenviar código».",
            formato::telefono_enmascarado(&telefono),
            codigo_otp::VIGENCIA_MINUTOS
        ),
        Solicitud::LimiteExcedido => format!(
            "Ya se pidieron {} códigos en los últimos {} minutos. Por seguridad, espera un poco y pulsa «Reenviar código», o cambia de número.",
            verificacion_telefono::MAX_SOLICITUDES,
            verificacion_telefono::VENTANA_MINUTOS
        ),
        Solicitud::Demorado => {
            "El SMS está tardando más de lo normal. Espera un momento: si te llega, escribe el código aquí; si no, pulsa «Reenviar código».".to_string()
        }
        Solicitud::NoSePudoEnviar => {
            "No pudimos enviar el SMS en este momento. Espera un minuto y pulsa «Reenviar código».".to_string()
        }
        Solicitud::NumeroInvalido => return Ok(pedir_telefono()),
    };

    Ok((r(&texto, &OPCIONES_CODIGO), VERIFICAR_CELULAR))
}

// La persona escribe el código que le llegó (o pide otro, o cambia de número).
async fn paso_verificar_celular(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
    red: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(telefono) = datos.telefono.clone().filter(|t| !t.is_empty()) else {
        return Ok(pedir_telefono()); // por si los datos quedaron incompletos
    };

    let t = normalizar(texto);
    let t = t.trim();
    if t.contains("cambiar") {
        datos.telefono = None;
        datos.telefono_verificado = false;
        return Ok(pedir_telefono());
    }
    if t.contains("reenviar") || t.contains("otro codigo") || t.contains("nuevo codigo") {
        return enviar_codigo(pool, conversacion, datos, red).await;
    }

    let Some(codigo) = codigo_otp::codigo_escrito(texto) else {
        return Ok((
            r("Escribe los 6 números del código que te llegó por SMS. Si no te llegó, pulsa «Reenviar código».", &OPCIONES_CODIGO),
            VERIFICAR_CELULAR,
        ));
    };

    let mensaje = match verificacion_telefono::verificar_codigo(pool, &conversacion.codigo, &telefono, &codigo).await? {
        ResultadoCodigo::Verificado => {
            datos.telefono_verificado = true;
            return Ok((
                r("¡Listo, tu celular quedó verificado! ¿Cuál es tu correo electrónico? Es opcional: si no quieres darlo, pulsa «Omitir».", &["Omitir"]),
                CORREO,
            ));
        }
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
    Ok((r(&mensaje, &OPCIONES_CODIGO), VERIFICAR_CELULAR))
}

async fn paso_correo(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    // Sin el celular verificado no se registra a nadie (por si alguien intenta saltarse el código).
    if !datos.telefono_verificado {
        return Ok(pedir_telefono());
    }

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
        telefono_verificado: datos.telefono_verificado,
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
    datos.reiniciar();

    let mensaje = format!(
        "¡Listo, {}! Ya quedaste registrado. Tu código de cliente es {}. ¿En qué te puedo ayudar con el servicio técnico?",
        nombre, codigo
    );
    // Si venía pidiendo un servicio técnico en /servicios, se sigue con esa solicitud donde se quedó.
    if datos.solicitud.is_some() {
        let saludo = format!("¡Listo, {}! Ya quedaste registrado. Tu código de cliente es {}.", nombre, codigo);
        let (mut respuesta, paso) =
            flujo_solicitud::reanudar_tras_identificar(pool, conversacion, &codigo, datos, &saludo).await?;
        respuesta.componente = respuesta.componente.or(Some(tarjeta));
        return Ok((respuesta, paso));
    }
    // Si venía armando un pedido en la tienda, lo que sigue es ese pedido (el carrito se quedó guardado).
    if !datos.carrito.is_empty() {
        let mensaje = format!("¡Listo, {}! Ya quedaste registrado. Tu código de cliente es {}. Tu carrito sigue guardado: ¿vamos a pagar?", nombre, codigo);
        let paso = if datos.en_catalogo { CATALOGO } else { LIBRE };
        return Ok((r(&mensaje, &["Ir a pagar", "Seguir viendo"]).con_componente(tarjeta), paso));
    }
    let opciones: &[&str] = if sin_correo {
        &["Ver servicios", "Agendar una cita", "Agregar mi correo"]
    } else {
        &MENU
    };
    Ok((r(&mensaje, opciones).con_componente(tarjeta), LIBRE))
}
