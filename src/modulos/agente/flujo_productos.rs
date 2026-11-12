// flujo_productos.rs: la parte del agente que muestra los productos y deja apartarlos.
//
//   libre -> elegir_categoria -> elegir_producto -> libre (tarjeta del producto con «Apartar este producto»)
//   libre -> "apartar" -> (sin registrar) pide registrarse
//                      -> (hay varias unidades) apartar_cantidad -> apartar_confirmar -> libre (apartado creado)
//                      -> (queda una sola)                         apartar_confirmar -> libre
//   libre -> "mis apartados" -> (varios) elegir_apartado -> libre (tarjeta del apartado)
//   libre -> "cancelar mi apartado" -> cancelar_apartado -> libre
//
// Apartar NO es pagar: el stock se descuenta, el cliente recibe un código y tiene unos días para recogerlo y
// pagar en la tienda; el Vendedor confirma la entrega desde el panel. Si pasa el plazo, las unidades vuelven
// solas al catálogo (lo hace catalogo/apartados.rs). Regla: este archivo NO escribe SQL; usa el repositorio
// y las reglas del módulo catalogo.

use sqlx::PgPool;

use super::flujo_citas::texto_cita_guardada;
use super::modelo::{Campo, Componente, Conversacion, Datos};
use super::motor::{detectar_intencion, elegir_opcion, es_afirmativo, es_negativo, normalizar, Intencion};
use super::pasos::{
    APARTAR_CANTIDAD, APARTAR_CONFIRMAR, CANCELAR_APARTADO, ELEGIR_APARTADO, ELEGIR_CATEGORIA,
    ELEGIR_PRODUCTO, LIBRE,
};
use super::respuesta::{r, Paso, Respuesta, MENU};
use crate::modulos::catalogo::modelo::{Apartado, Categoria, Producto, ResultadoApartado};
use crate::modulos::catalogo::{apartados, repositorio};

const VOLVER: &str = "Volver";
const OPCIONES_CONFIRMAR: [&str; 3] = ["Confirmar apartado", "Cambiar cantidad", "Cancelar"];
const OPCIONES_ANONIMO: [&str; 3] = ["Ya soy cliente", "Registrarme", "Solo estoy de paso"];
const OPCIONES_CANCELAR: [&str; 2] = ["Sí, cancelar", "No, mantenerlo"];
// Cuántos apartados recientes se ofrecen cuando la persona tiene varios.
const APARTADOS_RECIENTES: i64 = 5;

// ---------- Piezas de apoyo ----------

// Una respuesta cuyos botones se calcularon en el momento (categorías, productos, cantidades...).
fn con_botones(texto: &str, botones: Vec<String>) -> Respuesta {
    let referencias: Vec<&str> = botones.iter().map(|b| b.as_str()).collect();
    r(texto, &referencias)
}

// Los nombres como botones, más "Volver".
fn botones_con_volver(nombres: impl Iterator<Item = String>) -> Vec<String> {
    let mut botones: Vec<String> = nombres.collect();
    botones.push(VOLVER.to_string());
    botones
}

fn es_volver(texto: &str) -> bool {
    normalizar(texto).trim() == "volver"
}

// ¿Lo escrito significa "dejemos esto"? Sí si dice "volver", dice que no, o pide otra cosa conocida
// (cita, saludo, despedida...). Se revisa DESPUÉS de intentar elegir una opción de la lista.
fn quiere_dejarlo(texto: &str) -> bool {
    es_volver(texto)
        || es_negativo(texto)
        || !matches!(
            detectar_intencion(texto),
            Intencion::Desconocida | Intencion::Productos
        )
}

// Solo con palabras claras: "cancelar", "volver", "salir" o un "no" (para los pasos de confirmación).
fn quiere_cancelar(texto: &str) -> bool {
    let normal = normalizar(texto);
    ["cancelar", "volver", "salir"].contains(&normal.trim()) || es_negativo(texto)
}

// Olvida lo recogido para apartar (el apartado en vista se conserva para «Cancelar mi apartado»).
fn limpiar_producto(datos: &mut Datos) {
    datos.categoria_elegida = None;
    datos.producto_elegido = None;
    datos.apartar_cantidad = None;
}

fn salir(datos: &mut Datos) -> Paso {
    limpiar_producto(datos);
    (r("Sin problema. ¿En qué te puedo ayudar?", &MENU), LIBRE)
}

// "1", "2 unidades", "quiero 3": el primer número que aparece en el texto.
fn numero_en(texto: &str) -> Option<i32> {
    let digitos: String = texto
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .take(3)
        .collect();
    digitos.parse().ok()
}

fn texto_unidades(cantidad: i32) -> String {
    if cantidad == 1 {
        "1 unidad".to_string()
    } else {
        format!("{} unidades", cantidad)
    }
}

fn texto_precio(precio: &str) -> String {
    format!("S/ {}", precio)
}

fn campo(etiqueta: &str, valor: String) -> Campo {
    Campo { etiqueta: etiqueta.to_string(), valor, verificacion: None }
}

// ---------- Paso 1: las categorías ----------

pub async fn mostrar_categorias(pool: &PgPool, datos: &mut Datos) -> Result<Paso, sqlx::Error> {
    mostrar_categorias_con_texto(pool, datos, "Estas son nuestras categorías. ¿Cuál te interesa?").await
}

async fn mostrar_categorias_con_texto(
    pool: &PgPool,
    datos: &mut Datos,
    mensaje: &str,
) -> Result<Paso, sqlx::Error> {
    limpiar_producto(datos);
    let categorias = repositorio::listar_categorias(pool).await?;
    if categorias.is_empty() {
        return Ok((
            r(
                "Por ahora no tengo productos para mostrarte. Escríbenos y te ayudamos directamente.",
                &MENU,
            ),
            LIBRE,
        ));
    }
    Ok((
        con_botones(mensaje, botones_con_volver(categorias.into_iter().map(|c| c.nombre))),
        ELEGIR_CATEGORIA,
    ))
}

pub async fn paso_elegir_categoria(
    pool: &PgPool,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    if es_volver(texto) {
        return Ok(salir(datos));
    }
    let categorias = repositorio::listar_categorias(pool).await?;
    let nombres: Vec<&str> = categorias.iter().map(|c| c.nombre.as_str()).collect();

    match elegir_opcion(&nombres, texto) {
        Some(posicion) => mostrar_productos(pool, datos, &categorias[posicion]).await,
        None if quiere_dejarlo(texto) => Ok(salir(datos)),
        None => Ok((
            con_botones(
                "No encontré esa categoría. Elige una de la lista:",
                botones_con_volver(categorias.into_iter().map(|c| c.nombre)),
            ),
            ELEGIR_CATEGORIA,
        )),
    }
}

// ---------- Paso 2: los productos de una categoría ----------

async fn mostrar_productos(
    pool: &PgPool,
    datos: &mut Datos,
    categoria: &Categoria,
) -> Result<Paso, sqlx::Error> {
    let productos = repositorio::listar_productos_de_categoria(pool, &categoria.codigo).await?;
    if productos.is_empty() {
        return mostrar_categorias_con_texto(pool, datos, "Esa categoría ya no tiene productos. Elige otra:").await;
    }
    datos.categoria_elegida = Some(categoria.codigo.clone());
    let mensaje = format!("Estos son los productos de «{}». ¿Cuál quieres ver?", categoria.nombre);
    Ok((
        con_botones(&mensaje, botones_con_volver(productos.into_iter().map(|p| p.nombre))),
        ELEGIR_PRODUCTO,
    ))
}

pub async fn paso_elegir_producto(
    pool: &PgPool,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    // "Volver" regresa a las categorías.
    if es_volver(texto) {
        return mostrar_categorias(pool, datos).await;
    }
    // Si se perdió la categoría, se empieza de nuevo por la lista.
    let Some(categoria) = datos.categoria_elegida.clone() else {
        return mostrar_categorias(pool, datos).await;
    };
    let productos = repositorio::listar_productos_de_categoria(pool, &categoria).await?;
    let nombres: Vec<&str> = productos.iter().map(|p| p.nombre.as_str()).collect();

    match elegir_opcion(&nombres, texto) {
        Some(posicion) => Ok(presentar_producto(datos, &productos[posicion])),
        None if quiere_dejarlo(texto) => Ok(salir(datos)),
        None => Ok((
            con_botones(
                "No encontré ese producto. Elige uno de la lista:",
                botones_con_volver(productos.into_iter().map(|p| p.nombre)),
            ),
            ELEGIR_PRODUCTO,
        )),
    }
}

// La tarjeta del producto. Si hay unidades se ofrece apartarlo; si no, solo seguir mirando.
fn presentar_producto(datos: &mut Datos, producto: &Producto) -> Paso {
    datos.producto_elegido = Some(producto.codigo.clone());
    datos.apartar_cantidad = None;

    let (mensaje, disponibilidad, opciones): (String, String, &[&str]) = if producto.stock > 0 {
        (
            format!("Aquí tienes «{}». ¿Quieres apartarlo?", producto.nombre),
            if producto.stock == 1 {
                "1 unidad disponible".to_string()
            } else {
                format!("{} unidades disponibles", producto.stock)
            },
            &["Apartar este producto", "Ver otros productos"],
        )
    } else {
        (
            format!("«{}» está agotado por ahora. ¿Quieres ver otros productos?", producto.nombre),
            "Agotado".to_string(),
            &["Ver otros productos", "Ver servicios"],
        )
    };
    (
        r(&mensaje, opciones).con_componente(Componente::Producto {
            nombre: producto.nombre.clone(),
            descripcion: producto.descripcion.clone(),
            categoria: producto.categoria_nombre.clone(),
            precio: texto_precio(&producto.precio),
            disponibilidad,
        }),
        LIBRE,
    )
}

// ---------- Paso 3: apartar ----------

// La persona pidió apartar el producto que está viendo.
pub async fn iniciar_apartado(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
) -> Result<Paso, sqlx::Error> {
    // Sin registro no hay a quién asociarle el apartado ni cómo avisarle.
    if conversacion.cliente.is_none() {
        return Ok((
            r(
                "Para apartar un producto primero necesito registrarte: así recibes un código para recogerlo. \
                 Si ya eres cliente, pulsa «Ya soy cliente» y te reconozco con tu documento.",
                &OPCIONES_ANONIMO,
            ),
            LIBRE,
        ));
    }
    let Some(codigo) = datos.producto_elegido.clone() else {
        return mostrar_categorias_con_texto(pool, datos, "Primero elige el producto que quieres apartar:").await;
    };
    let Some(producto) = repositorio::obtener_producto(pool, &codigo).await? else {
        return mostrar_categorias_con_texto(pool, datos, "Ese producto ya no está disponible. Elige otro:").await;
    };
    Ok(pedir_cantidad(datos, &producto))
}

// Pregunta cuántas unidades. Si no hay unidades se muestra agotado; si queda una sola, se pasa a confirmar.
fn pedir_cantidad(datos: &mut Datos, producto: &Producto) -> Paso {
    datos.producto_elegido = Some(producto.codigo.clone());
    if producto.stock <= 0 {
        return presentar_producto(datos, producto);
    }
    if producto.stock == 1 {
        datos.apartar_cantidad = Some(1);
        return pedir_confirmacion(producto, 1);
    }
    datos.apartar_cantidad = None;
    let tope = apartados::cantidades_ofrecidas(producto.stock);
    let mensaje = format!(
        "¿Cuántas unidades de «{}» quieres apartar? Hay {} disponibles (máximo {} por apartado).",
        producto.nombre,
        producto.stock,
        apartados::MAX_UNIDADES
    );
    let botones = botones_con_volver(tope.iter().map(|n| n.to_string()));
    (con_botones(&mensaje, botones), APARTAR_CANTIDAD)
}

pub async fn paso_cantidad(
    pool: &PgPool,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    if quiere_cancelar(texto) {
        return Ok(salir(datos));
    }
    let Some(codigo) = datos.producto_elegido.clone() else {
        return mostrar_categorias(pool, datos).await;
    };
    let Some(producto) = repositorio::obtener_producto(pool, &codigo).await? else {
        return mostrar_categorias_con_texto(pool, datos, "Ese producto ya no está disponible. Elige otro:").await;
    };

    match numero_en(texto) {
        Some(cantidad) if apartados::cantidad_valida(cantidad, producto.stock) => {
            datos.apartar_cantidad = Some(cantidad);
            Ok(pedir_confirmacion(&producto, cantidad))
        }
        _ => {
            // Esa cantidad no sirve: se repite la pregunta, avisando el rango (salvo que ya no haya nada que elegir).
            let (respuesta, paso) = pedir_cantidad(datos, &producto);
            if paso == APARTAR_CANTIDAD {
                let hasta = apartados::cantidades_ofrecidas(producto.stock).len();
                let aviso = format!("Elige una cantidad de 1 a {}.", hasta);
                Ok((Respuesta { texto: aviso, ..respuesta }, paso))
            } else {
                Ok((respuesta, paso))
            }
        }
    }
}

// La tarjeta de confirmación: todavía no hay código, nada está guardado.
fn pedir_confirmacion(producto: &Producto, cantidad: i32) -> Paso {
    let mensaje = format!(
        "Revisa tu apartado. Guardaremos {} por {} días y pagas al recogerlo en la tienda. ¿Lo confirmo?",
        if cantidad == 1 { "la unidad".to_string() } else { format!("las {} unidades", cantidad) },
        apartados::VALIDEZ_DIAS
    );
    let tarjeta = Componente::Apartado {
        codigo: None,
        estado: "Pendiente de confirmar".to_string(),
        producto: producto.nombre.clone(),
        campos: vec![
            campo("Cantidad", texto_unidades(cantidad)),
            campo("Precio unitario", texto_precio(&producto.precio)),
            campo("Total", texto_precio(&apartados::total_texto(&producto.precio, cantidad))),
            campo("Plazo para recogerlo", format!("{} días desde que apartas", apartados::VALIDEZ_DIAS)),
            campo("Pago", "En la tienda, al recoger".to_string()),
        ],
    };
    (r(&mensaje, &OPCIONES_CONFIRMAR).con_componente(tarjeta), APARTAR_CONFIRMAR)
}

pub async fn paso_confirmar(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let normal = normalizar(texto);
    let Some(cliente) = conversacion.cliente.clone() else {
        return Ok(salir(datos));
    };
    if quiere_cancelar(texto) && !normal.contains("cambiar") {
        limpiar_producto(datos);
        return Ok((r("Sin problema, no aparté nada. ¿En qué te puedo ayudar?", &MENU), LIBRE));
    }
    let (Some(codigo), Some(cantidad)) = (datos.producto_elegido.clone(), datos.apartar_cantidad) else {
        return mostrar_categorias(pool, datos).await;
    };
    let Some(producto) = repositorio::obtener_producto(pool, &codigo).await? else {
        return mostrar_categorias_con_texto(pool, datos, "Ese producto ya no está disponible. Elige otro:").await;
    };

    if normal.contains("cambiar") {
        return Ok(pedir_cantidad(datos, &producto));
    }
    if !(normal.contains("confirm") || es_afirmativo(texto)) {
        return Ok((
            r("¿Confirmo tu apartado? Elige una opción:", &OPCIONES_CONFIRMAR),
            APARTAR_CONFIRMAR,
        ));
    }
    crear_apartado(pool, datos, &cliente, &producto, cantidad).await
}

// Guarda el apartado. Se vuelve a revisar el stock en el servidor: pudo agotarse mientras la persona decidía.
async fn crear_apartado(
    pool: &PgPool,
    datos: &mut Datos,
    cliente: &str,
    producto: &Producto,
    cantidad: i32,
) -> Result<Paso, sqlx::Error> {
    if !apartados::cantidad_valida(cantidad, producto.stock) {
        return Ok(aviso_sin_stock(datos, producto));
    }
    match apartados::apartar(pool, cliente, &producto.codigo, cantidad).await? {
        ResultadoApartado::SinStock => {
            // Alguien se llevó las unidades justo ahora: se vuelve a mirar cuántas quedan.
            match repositorio::obtener_producto(pool, &producto.codigo).await? {
                Some(actual) => Ok(aviso_sin_stock(datos, &actual)),
                None => mostrar_categorias_con_texto(pool, datos, "Ese producto ya no está disponible. Elige otro:").await,
            }
        }
        ResultadoApartado::Creado(codigo) => {
            limpiar_producto(datos);
            match apartados::obtener_de_cliente(pool, &codigo, cliente).await? {
                Some(apartado) => {
                    let mensaje = format!(
                        "¡Listo! Tu apartado quedó guardado. Tu código es {}. Tienes hasta el {} para recogerlo en la tienda y pagar allí. \
                         Si ya no lo quieres, puedes cancelarlo aquí mismo.",
                        apartado.codigo,
                        texto_cita_guardada(&apartado.vence)
                    );
                    let tarjeta = tarjeta_de_apartado(&apartado);
                    let opciones = opciones_de_apartado(datos, &apartado);
                    Ok((r(&mensaje, &opciones).con_componente(tarjeta), LIBRE))
                }
                None => Ok((r("¡Listo! Tu apartado quedó guardado.", &MENU), LIBRE)),
            }
        }
    }
}

// Lo que pasó: ya no alcanzan las unidades. Si todavía queda algo se ofrece cambiar la cantidad.
fn aviso_sin_stock(datos: &mut Datos, producto: &Producto) -> Paso {
    if producto.stock <= 0 {
        let (respuesta, paso) = presentar_producto(datos, producto);
        let texto = format!("Lo siento, «{}» se acaba de agotar. ¿Quieres ver otros productos?", producto.nombre);
        return (Respuesta { texto, ..respuesta }, paso);
    }
    let (respuesta, paso) = pedir_cantidad(datos, producto);
    let texto = format!(
        "Ya no alcanzan las unidades que pediste: quedan {}. {}",
        producto.stock, respuesta.texto
    );
    (Respuesta { texto, ..respuesta }, paso)
}

// ---------- Mis apartados ----------

// La tarjeta de un apartado ya guardado. Solo el pendiente muestra el plazo y la forma de pago.
fn tarjeta_de_apartado(apartado: &Apartado) -> Componente {
    let mut campos = vec![
        campo("Cantidad", texto_unidades(apartado.cantidad)),
        campo("Total", texto_precio(&apartado.total)),
    ];
    if apartados::esta_pendiente(&apartado.estado_codigo) {
        campos.push(campo("Recoger antes del", texto_cita_guardada(&apartado.vence)));
        campos.push(campo("Pago", "En la tienda, al recoger".to_string()));
    }
    Componente::Apartado {
        codigo: Some(apartado.codigo.clone()),
        estado: apartado.estado_nombre.clone(),
        producto: apartado.producto.clone(),
        campos,
    }
}

// Los botones que acompañan a un apartado. Si sigue pendiente se puede cancelar (y se recuerda cuál).
fn opciones_de_apartado(datos: &mut Datos, apartado: &Apartado) -> Vec<&'static str> {
    if apartados::esta_pendiente(&apartado.estado_codigo) {
        datos.apartado_en_vista = Some(apartado.codigo.clone());
        vec!["Cancelar mi apartado", "Ver productos"]
    } else {
        datos.apartado_en_vista = None;
        vec!["Ver productos", "Ver servicios"]
    }
}

// "K7TQ3M9XA · Mouse inalámbrico M170"
fn etiqueta_de_apartado(apartado: &Apartado) -> String {
    format!("{} · {}", apartado.codigo, apartado.producto)
}

pub async fn ver_apartados(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.clone() else {
        return Ok((
            r(
                "Para consultar tus apartados primero necesito saber quién eres. \
                 Si ya eres cliente, pulsa «Ya soy cliente» y te reconozco con tu documento.",
                &OPCIONES_ANONIMO,
            ),
            LIBRE,
        ));
    };

    let lista = apartados::listar_de_cliente(pool, &cliente, APARTADOS_RECIENTES).await?;
    match lista.len() {
        0 => Ok((
            r(
                "Todavía no tienes productos apartados. ¿Quieres ver nuestros productos?",
                &["Ver productos", "Ver servicios"],
            ),
            LIBRE,
        )),
        1 => Ok(mostrar_apartado(datos, &lista[0])),
        _ => Ok((
            con_botones(
                "Estos son tus apartados más recientes. ¿Cuál quieres ver?",
                botones_con_volver(lista.iter().map(etiqueta_de_apartado)),
            ),
            ELEGIR_APARTADO,
        )),
    }
}

pub async fn paso_elegir_apartado(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let Some(cliente) = conversacion.cliente.clone() else {
        return Ok(salir(datos));
    };
    if quiere_cancelar(texto) {
        return Ok(salir(datos));
    }

    let lista = apartados::listar_de_cliente(pool, &cliente, APARTADOS_RECIENTES).await?;
    let etiquetas: Vec<String> = lista.iter().map(etiqueta_de_apartado).collect();
    let referencias: Vec<&str> = etiquetas.iter().map(|e| e.as_str()).collect();

    match elegir_opcion(&referencias, texto) {
        Some(i) => Ok(mostrar_apartado(datos, &lista[i])),
        None => Ok((
            con_botones(
                "No encontré ese apartado. Elige uno de la lista:",
                botones_con_volver(etiquetas.into_iter()),
            ),
            ELEGIR_APARTADO,
        )),
    }
}

fn mostrar_apartado(datos: &mut Datos, apartado: &Apartado) -> Paso {
    let mensaje = format!(
        "Aquí está tu apartado {}. Estado actual: {}.",
        apartado.codigo, apartado.estado_nombre
    );
    let tarjeta = tarjeta_de_apartado(apartado);
    let opciones = opciones_de_apartado(datos, apartado);
    (r(&mensaje, &opciones).con_componente(tarjeta), LIBRE)
}

// ---------- Cancelar un apartado ----------

// "Cancelar mi apartado": pide confirmación del apartado que la persona tiene a la vista. Si no hay ninguno, primero lo muestra.
pub async fn pedir_cancelacion(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
) -> Result<Paso, sqlx::Error> {
    let Some(codigo) = datos.apartado_en_vista.clone() else {
        return ver_apartados(pool, conversacion, datos).await;
    };
    let mensaje = format!("¿Seguro que quieres cancelar el apartado {}?", codigo);
    Ok((r(&mensaje, &OPCIONES_CANCELAR), CANCELAR_APARTADO))
}

pub async fn paso_cancelar(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Paso, sqlx::Error> {
    let (Some(cliente), Some(codigo)) = (conversacion.cliente.clone(), datos.apartado_en_vista.clone()) else {
        return Ok(salir(datos));
    };

    if es_negativo(texto) || es_volver(texto) {
        return match apartados::obtener_de_cliente(pool, &codigo, &cliente).await? {
            Some(apartado) => Ok(mostrar_apartado(datos, &apartado)),
            None => Ok(salir(datos)),
        };
    }
    if es_afirmativo(texto) || normalizar(texto).contains("cancelar") {
        let cancelado = apartados::cancelar_de_cliente(pool, &codigo, &cliente).await?;
        datos.apartado_en_vista = None;
        let mensaje = if cancelado {
            "Listo, cancelé tu apartado. Las unidades volvieron a estar disponibles. ¿Algo más?"
        } else {
            "No pude cancelarlo porque ya no está pendiente (se entregó, venció o ya estaba cancelado). Comunícate con H&Devs si necesitas ayuda."
        };
        return Ok((r(mensaje, &MENU), LIBRE));
    }
    Ok((r("¿Cancelo el apartado? Responde sí o no.", &OPCIONES_CANCELAR), CANCELAR_APARTADO))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn lee_la_cantidad_escrita() {
        assert_eq!(numero_en("2"), Some(2));
        assert_eq!(numero_en("quiero 3 unidades"), Some(3));
        assert_eq!(numero_en("dos"), None);
        assert_eq!(numero_en(""), None);
    }

    #[test]
    fn textos_de_unidades() {
        assert_eq!(texto_unidades(1), "1 unidad");
        assert_eq!(texto_unidades(4), "4 unidades");
    }
}
