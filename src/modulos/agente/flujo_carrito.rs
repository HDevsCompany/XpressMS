// flujo_carrito.rs: el carrito de la tienda (/catalogo). La página solo MUESTRA el carrito; quien lo guarda y lo aparta es Juno.
//
//   «Añadir al carrito: MOUSE M170»  -> suma una unidad (los botones de las tarjetas y de la ficha envían esto)
//   «Quitar del carrito: MOUSE M170» -> lo quita
//   «Cambiar cantidad: MOUSE M170 = 2» -> cambia las unidades
//   «Ver carrito» / «Vaciar carrito» / «Seguir viendo»
//   «Ir a pagar»                     -> pide registrarse si hace falta; si no, ofrece cómo completar el pedido
//   «Pagar ahora»                    -> el pago en línea todavía no está disponible: se ofrece apartar
//   «Apartar mi pedido»              -> aparta TODO el carrito en un solo pedido (un código de retiro) y se paga en la tienda
//
// El carrito vive en Datos.carrito (lo guarda la conversación de Juno, así sobrevive a recargar la página). Cada respuesta
// lleva el componente «carrito» con la lista ya calculada (nombres, cantidades, precios, total) y la página lo dibuja.
// Las órdenes son textos exactos y solo se atienden cuando la persona no está en medio de otra cosa (ver flujo::responder).
// Regla: este archivo NO escribe SQL; usa el repositorio y las reglas del módulo catalogo.

use sqlx::PgPool;

use super::flujo_catalogo::producto_nombrado;
use super::flujo_citas::texto_cita_guardada;
use super::modelo::{Componente, Conversacion, Datos, ItemCarrito, LineaCarrito, LineaPedido};
use super::motor::normalizar;
use super::pasos::{CATALOGO, LIBRE};
use super::respuesta::{r, Paso, Respuesta};
use crate::modulos::catalogo::modelo::{Producto, ResultadoPedido};
use crate::modulos::catalogo::{apartados, repositorio};

const OPCIONES_ANONIMO: [&str; 3] = ["Ya soy cliente", "Registrarme", "Solo estoy de paso"];
const OPCIONES_CIERRE: [&str; 2] = ["Pagar ahora", "Apartar y pagar en tienda"];
const OPCIONES_SEGUIR: [&str; 2] = ["Seguir viendo", "Ir a pagar"];
const VER_TODO: &str = "Ver todo";
const MENSAJE_REGISTRO: &str = "Para completar tu pedido primero necesito registrarte: así recibes un código para recogerlo. \
     Si ya eres cliente, pulsa «Ya soy cliente» y te reconozco con tu documento. Tu carrito se queda guardado.";

// ---------- Entender la orden (función pura, con pruebas) ----------

#[derive(Debug, PartialEq)]
pub enum Orden {
    Anadir(String),        // «Añadir al carrito: <nombre>»
    Quitar(String),        // «Quitar del carrito: <nombre>»
    Cantidad(String, i32), // «Cambiar cantidad: <nombre> = 2»
    Consulta(String),      // «Tengo una duda sobre: <nombre>» («Preguntarle a Juno» en la ficha)
    Ver,
    Vaciar,
    SeguirViendo,
    IrAPagar,
    PagarAhora,
    ApartarPedido,
}

// Solo reconoce las órdenes exactas del carrito (las que envían los botones de la página y de Juno): cualquier otra frase
// sigue su camino normal en el catálogo o en la conversación libre.
pub fn entender(texto: &str) -> Option<Orden> {
    let normal = normalizar(texto);
    let normal = normal.trim();

    for prefijo in ["añadir al carrito:", "anadir al carrito:", "agregar al carrito:"] {
        if let Some(resto) = normal.strip_prefix(prefijo) {
            return Some(Orden::Anadir(resto.trim().to_string()));
        }
    }
    if let Some(resto) = normal.strip_prefix("quitar del carrito:") {
        return Some(Orden::Quitar(resto.trim().to_string()));
    }
    if let Some(resto) = normal.strip_prefix("tengo una duda sobre:") {
        return Some(Orden::Consulta(resto.trim().to_string()));
    }
    if let Some(resto) = normal.strip_prefix("cambiar cantidad:") {
        let (nombre, cantidad) = resto.rsplit_once('=')?;
        let cantidad: i32 = cantidad.trim().parse().ok()?;
        return Some(Orden::Cantidad(nombre.trim().to_string(), cantidad));
    }
    match normal {
        "ver carrito" | "ver mi carrito" | "mi carrito" | "carrito" => Some(Orden::Ver),
        "vaciar carrito" | "vaciar mi carrito" => Some(Orden::Vaciar),
        "seguir viendo" => Some(Orden::SeguirViendo),
        "ir a pagar" | "continuar con mi pedido" | "finalizar pedido" => Some(Orden::IrAPagar),
        "pagar ahora" => Some(Orden::PagarAhora),
        "apartar mi pedido" | "apartar y pagar en tienda" | "apartar pedido" => Some(Orden::ApartarPedido),
        _ => None,
    }
}

// ---------- Dinero (en céntimos, sin decimales de computadora) ----------

// "91.00" -> 9100. Si no se entiende, 0.
fn a_centimos(texto: &str) -> i64 {
    let (soles, resto) = texto.split_once('.').unwrap_or((texto, "0"));
    let soles: i64 = soles.trim().parse().unwrap_or(0);
    let centimos: i64 = format!("{:0<2}", resto.trim()).chars().take(2).collect::<String>().parse().unwrap_or(0);
    soles * 100 + centimos
}

// 25700 -> "S/ 257.00"
fn en_soles(centimos: i64) -> String {
    format!("S/ {}.{:02}", centimos / 100, centimos % 100)
}

// ---------- Piezas de apoyo ----------

// Cuántas unidades de un producto se pueden llevar: lo que hay, con el tope por apartado.
fn maximo(producto: &Producto) -> i32 {
    producto.stock.max(0).min(apartados::MAX_UNIDADES)
}

fn cantidad_en(datos: &Datos, codigo: &str) -> i32 {
    datos.carrito.iter().find(|i| i.producto == codigo).map_or(0, |i| i.cantidad)
}

// El producto que dice la persona: por su nombre completo o, si no, por el nombre que aparezca en la frase.
fn buscar<'a>(productos: &'a [Producto], dicho: &str) -> Option<&'a Producto> {
    let dicho = normalizar(dicho);
    let dicho = dicho.trim();
    productos
        .iter()
        .find(|p| normalizar(&p.nombre) == dicho)
        .or_else(|| producto_nombrado(productos, dicho))
}

// En qué paso queda la conversación: en la vitrina si es la ventana de la tienda; si no, en la conversación libre.
fn paso_actual(datos: &Datos) -> &'static str {
    if datos.en_catalogo {
        CATALOGO
    } else {
        LIBRE
    }
}

// Ajusta el carrito a lo que hay hoy (quita lo que se agotó o ya no existe, baja las cantidades que ya no alcanzan) y arma
// lo que la página dibuja. Devuelve también un aviso, si tuvo que cambiar algo.
pub fn armar(todos: &[Producto], datos: &mut Datos, fase: &str) -> (Componente, Option<String>) {
    let mut avisos: Vec<String> = Vec::new();
    let mut lineas: Vec<LineaCarrito> = Vec::new();
    let mut vigentes: Vec<ItemCarrito> = Vec::new();
    let mut unidades = 0;
    let mut total: i64 = 0;

    for item in &datos.carrito {
        let Some(p) = todos.iter().find(|p| p.codigo == item.producto) else {
            continue; // el producto ya no existe
        };
        let tope = maximo(p);
        if tope == 0 {
            avisos.push(format!("«{}» se agotó y lo quité de tu carrito.", p.nombre));
            continue;
        }
        let cantidad = item.cantidad.clamp(1, tope);
        if cantidad < item.cantidad {
            avisos.push(format!("Ajusté «{}» a {} por las unidades que quedan.", p.nombre, cantidad));
        }
        let subtotal = a_centimos(&p.precio) * i64::from(cantidad);
        total += subtotal;
        unidades += cantidad;
        lineas.push(LineaCarrito {
            id: p.codigo.clone(),
            nombre: p.nombre.clone(),
            detalle: p.descripcion.clone(),
            categoria: p.categoria_nombre.clone(),
            cantidad,
            maximo: tope,
            precio_unitario: format!("S/ {}", p.precio),
            subtotal: en_soles(subtotal),
        });
        vigentes.push(ItemCarrito { producto: p.codigo.clone(), cantidad });
    }
    datos.carrito = vigentes;

    let aviso = if avisos.is_empty() { None } else { Some(avisos.join(" ")) };
    let componente =
        Componente::Carrito { fase: fase.to_string(), productos: lineas, unidades, total: en_soles(total) };
    (componente, aviso)
}

// Una respuesta de Juno que lleva el carrito al día. Si hubo que ajustar algo, el aviso va primero.
fn con_carrito(texto: &str, opciones: &[&str], todos: &[Producto], datos: &mut Datos, fase: &str) -> Respuesta {
    let (componente, aviso) = armar(todos, datos, fase);
    let texto = match aviso {
        Some(aviso) => format!("{} {}", aviso, texto),
        None => texto.to_string(),
    };
    r(&texto, opciones).con_componente(componente)
}

// Si la persona vuelve a abrir la tienda con un carrito guardado, la página lo recupera con esto.
pub async fn reanudar(pool: &PgPool, datos_guardados: &str) -> Result<Option<Componente>, sqlx::Error> {
    let mut datos: Datos = serde_json::from_str(datos_guardados).unwrap_or_default();
    if datos.carrito.is_empty() {
        return Ok(None);
    }
    let todos = repositorio::listar_productos(pool).await?;
    let (componente, _) = armar(&todos, &mut datos, "carrito");
    Ok(Some(componente))
}

// ---------- Atender una orden ----------

// Devuelve None cuando lo escrito no es una orden del carrito.
pub async fn atender(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Option<Paso>, sqlx::Error> {
    let Some(orden) = entender(texto) else {
        return Ok(None);
    };
    let todos = repositorio::listar_productos(pool).await?;
    let paso = match orden {
        Orden::Anadir(nombre) => anadir(&todos, datos, &nombre),
        Orden::Quitar(nombre) => quitar(&todos, datos, &nombre),
        Orden::Cantidad(nombre, cantidad) => cambiar_cantidad(&todos, datos, &nombre, cantidad),
        Orden::Consulta(nombre) => consulta(&todos, datos, &nombre),
        Orden::Ver => ver(&todos, datos),
        Orden::Vaciar => vaciar(&todos, datos),
        Orden::SeguirViendo => seguir_viendo(&todos, datos),
        Orden::IrAPagar => ir_a_pagar(&todos, conversacion, datos),
        Orden::PagarAhora => pagar_ahora(&todos, conversacion, datos),
        Orden::ApartarPedido => apartar_pedido(pool, &todos, conversacion, datos).await?,
    };
    Ok(Some(paso))
}

fn no_encontrado(datos: &Datos) -> Paso {
    (r("No encontré ese producto. Dime su nombre o tócalo en la vitrina.", &[VER_TODO]), paso_actual(datos))
}

fn anadir(todos: &[Producto], datos: &mut Datos, dicho: &str) -> Paso {
    let paso = paso_actual(datos);
    let Some(producto) = buscar(todos, dicho) else {
        return no_encontrado(datos);
    };
    let tope = maximo(producto);
    if tope == 0 {
        let texto = format!("Lo siento, «{}» está agotado por ahora. ¿Quieres ver otros productos?", producto.nombre);
        return (r(&texto, &[VER_TODO]), paso);
    }
    if cantidad_en(datos, &producto.codigo) >= tope {
        let texto = format!(
            "Ya tienes en tu carrito todas las unidades que puedo guardarte de «{}» ({}). ¿Algo más, o vamos a pagar?",
            producto.nombre, tope
        );
        return (con_carrito(&texto, &OPCIONES_SEGUIR, todos, datos, "carrito"), paso);
    }
    let posicion = datos.carrito.iter().position(|i| i.producto == producto.codigo);
    match posicion {
        Some(i) => datos.carrito[i].cantidad += 1,
        None => datos.carrito.push(ItemCarrito { producto: producto.codigo.clone(), cantidad: 1 }),
    }
    let texto = format!("Listo, añadí «{}» a tu carrito. ¿Algo más, o vamos a pagar?", producto.nombre);
    (con_carrito(&texto, &OPCIONES_SEGUIR, todos, datos, "carrito"), paso)
}

// «Preguntarle a Juno» en la ficha: Juno dice lo que sabe del producto (precio y existencias) y deja la conversación abierta.
// No inventa garantías ni plazos: lo que no está en el catálogo se consulta en la tienda.
fn consulta(todos: &[Producto], datos: &mut Datos, dicho: &str) -> Paso {
    let paso = paso_actual(datos);
    let Some(producto) = buscar(todos, dicho) else {
        return no_encontrado(datos);
    };
    let existencias = match producto.stock {
        n if n <= 0 => "por ahora está agotado".to_string(),
        1 => "queda 1 unidad".to_string(),
        n => format!("hay {} unidades disponibles", n),
    };
    let texto = format!(
        "Claro. «{}» cuesta S/ {} y {}. ¿Qué quieres saber? Escríbeme tu duda; si es algo que no tengo en el catálogo, lo mejor es confirmarlo en la tienda.",
        producto.nombre, producto.precio, existencias
    );
    (r(&texto, &[VER_TODO]), paso)
}

fn quitar(todos: &[Producto], datos: &mut Datos, dicho: &str) -> Paso {
    let paso = paso_actual(datos);
    let Some(producto) = buscar(todos, dicho) else {
        return no_encontrado(datos);
    };
    datos.carrito.retain(|i| i.producto != producto.codigo);
    let texto = format!("Quité «{}» de tu carrito.", producto.nombre);
    (tras_cambio(&texto, todos, datos), paso)
}

fn cambiar_cantidad(todos: &[Producto], datos: &mut Datos, dicho: &str, cantidad: i32) -> Paso {
    let paso = paso_actual(datos);
    let Some(producto) = buscar(todos, dicho) else {
        return no_encontrado(datos);
    };
    if cantidad <= 0 {
        return quitar(todos, datos, dicho);
    }
    let tope = maximo(producto);
    let nueva = cantidad.min(tope.max(1));
    let posicion = datos.carrito.iter().position(|i| i.producto == producto.codigo);
    match posicion {
        Some(i) => datos.carrito[i].cantidad = nueva,
        None => datos.carrito.push(ItemCarrito { producto: producto.codigo.clone(), cantidad: nueva }),
    }
    let texto = if cantidad > tope {
        format!("De «{}» solo puedo guardarte {}. Lo dejé en {}.", producto.nombre, tope, nueva)
    } else {
        format!("Listo, dejé «{}» en {}.", producto.nombre, nueva)
    };
    (tras_cambio(&texto, todos, datos), paso)
}

// La respuesta después de quitar o cambiar algo: si el carrito quedó vacío se dice; si no, se ofrece seguir o pagar.
fn tras_cambio(texto: &str, todos: &[Producto], datos: &mut Datos) -> Respuesta {
    if datos.carrito.is_empty() {
        let texto = format!("{} Tu carrito quedó vacío.", texto);
        return con_carrito(&texto, &[VER_TODO], todos, datos, "carrito");
    }
    con_carrito(texto, &OPCIONES_SEGUIR, todos, datos, "carrito")
}

fn ver(todos: &[Producto], datos: &mut Datos) -> Paso {
    let paso = paso_actual(datos);
    if datos.carrito.is_empty() {
        return (con_carrito("Tu carrito está vacío. Añade lo que te guste desde la vitrina.", &[VER_TODO], todos, datos, "carrito"), paso);
    }
    (con_carrito("Este es tu carrito. ¿Seguimos viendo productos o vamos a pagar?", &OPCIONES_SEGUIR, todos, datos, "carrito"), paso)
}

fn vaciar(todos: &[Producto], datos: &mut Datos) -> Paso {
    datos.carrito.clear();
    (con_carrito("Listo, vacié tu carrito.", &[VER_TODO], todos, datos, "carrito"), paso_actual(datos))
}

fn seguir_viendo(todos: &[Producto], datos: &mut Datos) -> Paso {
    let paso = paso_actual(datos);
    if datos.carrito.is_empty() {
        return (con_carrito("Perfecto, sigue mirando con calma.", &[VER_TODO], todos, datos, "carrito"), paso);
    }
    (
        con_carrito("Perfecto, sigue mirando. Tu carrito se queda guardado; cuando quieras, dime «Ir a pagar».", &["Ir a pagar"], todos, datos, "carrito"),
        paso,
    )
}

// ---------- Cerrar el pedido ----------

// Lo que falta antes de poder cerrar: que haya algo en el carrito y que la persona sea cliente. Devuelve la respuesta que
// corresponde si todavía no se puede, o None si ya se puede.
fn falta_algo(todos: &[Producto], conversacion: &Conversacion, datos: &mut Datos) -> Option<Paso> {
    let paso = paso_actual(datos);
    if datos.carrito.is_empty() {
        return Some((
            con_carrito("Tu carrito está vacío. Añade lo que te guste y seguimos.", &[VER_TODO], todos, datos, "carrito"),
            paso,
        ));
    }
    if conversacion.cliente.is_none() {
        return Some((con_carrito(MENSAJE_REGISTRO, &OPCIONES_ANONIMO, todos, datos, "carrito"), paso));
    }
    None
}

fn ir_a_pagar(todos: &[Producto], conversacion: &Conversacion, datos: &mut Datos) -> Paso {
    if let Some(paso) = falta_algo(todos, conversacion, datos) {
        return paso;
    }
    let paso = paso_actual(datos);
    (con_carrito("Tu pedido está listo. ¿Cómo quieres completarlo?", &OPCIONES_CIERRE, todos, datos, "cierre"), paso)
}

fn pagar_ahora(todos: &[Producto], conversacion: &Conversacion, datos: &mut Datos) -> Paso {
    if let Some(paso) = falta_algo(todos, conversacion, datos) {
        return paso;
    }
    let paso = paso_actual(datos);
    let texto = "El pago en línea todavía no está disponible. Mientras tanto puedes apartar tu pedido y pagarlo en la tienda al recogerlo.";
    (con_carrito(texto, &["Apartar y pagar en tienda"], todos, datos, "cierre"), paso)
}

// Aparta TODO el carrito en un solo pedido. Se revisa el stock otra vez en el servidor: pudo agotarse mientras la persona decidía.
async fn apartar_pedido(
    pool: &PgPool,
    todos: &[Producto],
    conversacion: &Conversacion,
    datos: &mut Datos,
) -> Result<Paso, sqlx::Error> {
    if let Some(paso) = falta_algo(todos, conversacion, datos) {
        return Ok(paso);
    }
    let paso = paso_actual(datos);
    let Some(cliente) = conversacion.cliente.clone() else {
        return Ok(ir_a_pagar(todos, conversacion, datos));
    };

    // Lo que se va a apartar es lo que hay hoy (si algo se agotó, el carrito se ajusta y se avisa antes de guardar nada).
    let (_, aviso) = armar(todos, datos, "carrito");
    if let Some(aviso) = aviso {
        let texto = format!("{} Revisa tu carrito y confírmame otra vez.", aviso);
        return Ok((con_carrito(&texto, &OPCIONES_SEGUIR, todos, datos, "carrito"), paso));
    }

    let items: Vec<(String, i32)> = datos.carrito.iter().map(|i| (i.producto.clone(), i.cantidad)).collect();
    match apartados::apartar_pedido(pool, &cliente, &items).await? {
        ResultadoPedido::SinStock(codigo) => {
            // Alguien se llevó las unidades justo ahora: se vuelve a mirar qué hay.
            let actuales = repositorio::listar_productos(pool).await?;
            let nombre = actuales
                .iter()
                .find(|p| p.codigo == codigo)
                .map_or("uno de tus productos".to_string(), |p| format!("«{}»", p.nombre));
            let texto = format!("Se acaba de agotar {} y no pude apartar tu pedido: no aparté nada. Revisa tu carrito.", nombre);
            Ok((con_carrito(&texto, &OPCIONES_SEGUIR, &actuales, datos, "carrito"), paso))
        }
        ResultadoPedido::Creado(pedido) => {
            datos.carrito.clear();
            let apartados = apartados::listar_pedido_de_cliente(pool, &pedido, &cliente).await?;
            let total: i64 = apartados.iter().map(|a| a_centimos(&a.total)).sum();
            let vence = apartados.first().map(|a| texto_cita_guardada(&a.vence)).unwrap_or_default();
            let productos: Vec<LineaPedido> = apartados
                .iter()
                .map(|a| LineaPedido {
                    nombre: a.producto.clone(),
                    cantidad: a.cantidad,
                    subtotal: format!("S/ {}", a.total),
                })
                .collect();
            let texto = format!(
                "¡Listo! Tu pedido quedó apartado. Tu código de retiro es {}. Tienes hasta el {} para recogerlo en la tienda y pagar allí.",
                pedido, vence
            );
            let tarjeta = Componente::Pedido { codigo: pedido, productos, total: en_soles(total), vence };
            Ok((r(&texto, &[VER_TODO]).con_componente(tarjeta), paso))
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn entiende_las_ordenes_exactas() {
        assert_eq!(entender("Añadir al carrito: MOUSE INALÁMBRICO LOGITECH M170"), Some(Orden::Anadir("mouse inalambrico logitech m170".to_string())));
        assert_eq!(entender("quitar del carrito: Mouse"), Some(Orden::Quitar("mouse".to_string())));
        assert_eq!(entender("Cambiar cantidad: Mouse M170 = 3"), Some(Orden::Cantidad("mouse m170".to_string(), 3)));
        assert_eq!(entender("Tengo una duda sobre: Mouse M170"), Some(Orden::Consulta("mouse m170".to_string())));
        assert_eq!(entender("Ver carrito"), Some(Orden::Ver));
        assert_eq!(entender("  Ir a pagar "), Some(Orden::IrAPagar));
        assert_eq!(entender("Apartar mi pedido"), Some(Orden::ApartarPedido));
        assert_eq!(entender("Apartar y pagar en tienda"), Some(Orden::ApartarPedido));
        assert_eq!(entender("Pagar ahora"), Some(Orden::PagarAhora));
        assert_eq!(entender("Seguir viendo"), Some(Orden::SeguirViendo));
    }

    #[test]
    fn lo_demas_no_es_una_orden_del_carrito() {
        assert_eq!(entender("laptops"), None);
        assert_eq!(entender("Apartar Mouse M170"), None);
        assert_eq!(entender("cambiar cantidad: mouse"), None); // falta el «= número»
        assert_eq!(entender("quiero pagar mi cita"), None);
    }

    #[test]
    fn los_montos_se_calculan_en_centimos() {
        assert_eq!(a_centimos("39.00"), 3900);
        assert_eq!(a_centimos("1899.00"), 189900);
        assert_eq!(a_centimos("0.5"), 50);
        assert_eq!(a_centimos("basura"), 0);
        assert_eq!(en_soles(25700), "S/ 257.00");
        assert_eq!(en_soles(5), "S/ 0.05");
    }

    fn producto(codigo: &str, nombre: &str, precio: &str, stock: i32) -> Producto {
        Producto {
            codigo: codigo.to_string(),
            categoria_codigo: "CAT".to_string(),
            categoria_nombre: "Accesorios".to_string(),
            nombre: nombre.to_string(),
            descripcion: "descripcion".to_string(),
            precio: precio.to_string(),
            stock,
        }
    }

    #[test]
    fn el_carrito_se_ajusta_al_stock_y_suma_el_total() {
        let todos = vec![
            producto("A", "MOUSE M170", "39.00", 25),
            producto("B", "SSD 500GB", "129.00", 2),
            producto("C", "PANTALLA", "259.00", 0),
        ];
        let mut datos = Datos::default();
        datos.carrito = vec![
            ItemCarrito { producto: "A".to_string(), cantidad: 2 },
            ItemCarrito { producto: "B".to_string(), cantidad: 4 }, // solo quedan 2
            ItemCarrito { producto: "C".to_string(), cantidad: 1 }, // agotado: se quita
            ItemCarrito { producto: "Z".to_string(), cantidad: 1 }, // ya no existe
        ];
        let (componente, aviso) = armar(&todos, &mut datos, "carrito");
        assert!(aviso.is_some());
        assert_eq!(datos.carrito.len(), 2);
        match componente {
            Componente::Carrito { unidades, total, productos, .. } => {
                assert_eq!(unidades, 4); // 2 mouses + 2 SSD
                assert_eq!(total, "S/ 336.00"); // 2*39 + 2*129
                assert_eq!(productos.len(), 2);
            }
            _ => panic!("tenía que ser un carrito"),
        }
    }

    #[test]
    fn busca_el_producto_por_nombre() {
        let todos = vec![producto("A", "MOUSE INALÁMBRICO LOGITECH M170", "39.00", 5), producto("B", "TECLADO USB", "55.00", 5)];
        assert_eq!(buscar(&todos, "mouse inalambrico logitech m170").map(|p| p.codigo.as_str()), Some("A"));
        assert_eq!(buscar(&todos, "teclado usb").map(|p| p.codigo.as_str()), Some("B"));
        assert!(buscar(&todos, "impresora").is_none());
    }
}
