// flujo_catalogo.rs: el catálogo público de productos. La pantalla solo MUESTRA la vitrina; quien busca y filtra es Juno.
//
//   libre -> "Ver catálogo / Ver productos" -> catalogo (vitrina completa)
//   catalogo -> «Solo Laptops» / «Menos de S/ 500» / «mouse inalámbrico» -> catalogo (vitrina filtrada)
//   catalogo -> «Ver todo» -> catalogo (sin filtros)
//   catalogo -> «Apartar Mouse M170» -> (sigue el flujo de apartados de flujo_productos.rs)
//   catalogo -> cualquier otra cosa conocida (citas, servicios, saludo...) -> la atiende la conversación libre
//
// El catálogo es PÚBLICO: se puede mirar sin registrarse. Apartar sí pide identificarse (eso lo hace flujo_productos.rs).
// Regla: este archivo NO escribe SQL; usa el repositorio del módulo catalogo. Los filtros se guardan en Datos
// (catalogo_categoria, catalogo_precio_max, catalogo_busqueda) y se aplican aquí, en Rust, sobre la lista completa.

use sqlx::PgPool;

use super::flujo_productos;
use super::modelo::{Chip, Componente, Conversacion, Datos, FilaFicha, GrupoFicha, ProductoCatalogo, Vista};
use super::motor::{detectar_intencion, normalizar, Intencion};
use super::pasos::{CATALOGO, LIBRE};
use super::respuesta::{r, Paso, Respuesta, MENU};
use crate::modulos::catalogo::modelo::{Caracteristica, Categoria, Imagen, Producto};
use crate::modulos::catalogo::repositorio;

const VER_TODO: &str = "Ver todo";
// Los topes de precio que Juno ofrece como botón (solo los que de verdad separan productos).
const TOPES: [u32; 6] = [100, 300, 500, 1000, 2000, 5000];
// Palabras que dicen «tengo un presupuesto»: «menos de 500», «hasta 1,000», «máximo 300»...
const CLAVES_PRECIO: [&str; 8] =
    ["menos", "menor", "hasta", "maximo", "debajo", "bajo", "no mas de", "presupuesto"];
// Palabras que no sirven para buscar un producto (relleno de la frase).
const RELLENO: [&str; 52] = [
    "ver", "los", "las", "del", "muestrame", "muestra", "mostrar", "mostrarme", "catalogo", "completo",
    "producto", "productos", "quiero", "busco", "buscar", "necesito", "tienen", "tienes", "tiene", "hay",
    "con", "para", "una", "uno", "unos", "unas", "que", "por", "favor", "solo", "todo", "todos", "algun",
    "alguna", "algo", "sus", "mas", "menos", "menor", "hasta", "maximo", "debajo", "bajo", "precio",
    "soles", "venden", "vendes", "dame", "filtra", "filtrar", "quisiera", "presupuesto",
];

// Lo que Juno tiene aplicado a la vitrina en este momento.
#[derive(Clone, Default)]
struct Filtros {
    categoria: Option<String>, // código de la categoría
    precio_max: Option<u32>,
    busqueda: Option<String>, // palabras útiles, ya normalizadas
}

impl Filtros {
    fn desde(datos: &Datos) -> Self {
        Filtros {
            categoria: datos.catalogo_categoria.clone(),
            precio_max: datos.catalogo_precio_max,
            busqueda: datos.catalogo_busqueda.clone(),
        }
    }

    fn guardar(&self, datos: &mut Datos) {
        datos.catalogo_categoria = self.categoria.clone();
        datos.catalogo_precio_max = self.precio_max;
        datos.catalogo_busqueda = self.busqueda.clone();
    }

    fn vacio(&self) -> bool {
        self.categoria.is_none() && self.precio_max.is_none() && self.busqueda.is_none()
    }
}

// ---------- Entender lo que escribió la persona (funciones puras, con pruebas) ----------

// El primer número del texto, entendiendo miles: "500" -> 500, "S/ 1,500" -> 1500, "1.500" -> 1500, "99.50" -> 99.
pub fn primer_numero(texto: &str) -> Option<u32> {
    let letras: Vec<char> = texto.chars().collect();
    let mut i = letras.iter().position(|c| c.is_ascii_digit())?;
    let mut digitos = String::new();
    while i < letras.len() {
        let c = letras[i];
        if c.is_ascii_digit() {
            digitos.push(c);
            i += 1;
        } else if (c == ',' || c == '.')
            && i + 3 < letras.len()
            && letras[i + 1..=i + 3].iter().all(|d| d.is_ascii_digit())
            && letras.get(i + 4).map_or(true, |d| !d.is_ascii_digit())
        {
            // Una coma o punto seguidos de EXACTAMENTE tres cifras es separador de miles.
            i += 1;
        } else {
            break;
        }
    }
    if digitos.len() > 7 {
        return None;
    }
    digitos.parse().ok()
}

// «Menos de S/ 500» -> Some(500). Solo cuenta si la frase habla de un tope («menos», «hasta», «máximo»...).
pub fn precio_dicho(normal: &str) -> Option<u32> {
    if CLAVES_PRECIO.iter().any(|clave| normal.contains(clave)) {
        primer_numero(normal).filter(|n| *n > 0)
    } else {
        None
    }
}

// La categoría que nombra la frase. Solo si es UNA: «laptops», «solo accesorios», «repuesto»...
pub fn categoria_nombrada<'a>(categorias: &'a [Categoria], normal: &str) -> Option<&'a Categoria> {
    let coincidencias: Vec<&Categoria> = categorias
        .iter()
        .filter(|c| {
            let nombre = normalizar(&c.nombre);
            let raiz = nombre.trim().trim_end_matches('s');
            raiz.chars().count() >= 4 && normal.contains(raiz)
        })
        .collect();
    if coincidencias.len() == 1 {
        Some(coincidencias[0])
    } else {
        None
    }
}

// Las palabras que sirven para buscar: sin relleno, sin números y de 3 letras o más.
pub fn palabras_utiles(normal: &str) -> Vec<String> {
    normal
        .split(|c: char| !c.is_alphanumeric())
        .filter(|p| p.chars().count() >= 3)
        .filter(|p| !p.chars().all(|c| c.is_ascii_digit()))
        .filter(|p| !RELLENO.contains(p))
        .map(|p| p.to_string())
        .collect()
}

// El producto cuyo nombre aparece en la frase (si hay varios, el de nombre más largo: «Mouse M170» gana a «Mouse»).
pub fn producto_nombrado<'a>(productos: &'a [Producto], normal: &str) -> Option<&'a Producto> {
    productos
        .iter()
        .filter(|p| normal.contains(&normalizar(&p.nombre)))
        .max_by_key(|p| p.nombre.chars().count())
}

fn quiere_ver_todo(normal: &str) -> bool {
    ["ver todo", "todo el catalogo", "catalogo completo", "sin filtro", "quitar filtro", "limpiar"]
        .iter()
        .any(|f| normal.contains(f))
        || normal == "todo"
}

fn precio_de(producto: &Producto) -> f64 {
    producto.precio.parse().unwrap_or(0.0)
}

// «mouses» encaja con «mouse»: se prueba también sin la «s» final.
fn encaja(texto: &str, palabra: &str) -> bool {
    texto.contains(palabra)
        || (palabra.ends_with('s') && palabra.chars().count() > 4 && texto.contains(&palabra[..palabra.len() - 1]))
}

// Los productos que cumplen los filtros (todos si no hay ninguno).
fn aplicar<'a>(productos: &'a [Producto], filtros: &Filtros) -> Vec<&'a Producto> {
    let palabras: Vec<String> = filtros
        .busqueda
        .as_deref()
        .map(|b| palabras_utiles(&normalizar(b)))
        .unwrap_or_default();
    productos
        .iter()
        .filter(|p| filtros.categoria.as_deref().map_or(true, |c| p.categoria_codigo == c))
        .filter(|p| filtros.precio_max.map_or(true, |max| precio_de(p) <= f64::from(max)))
        .filter(|p| {
            let texto = normalizar(&format!("{} {} {}", p.nombre, p.descripcion, p.categoria_nombre));
            palabras.iter().all(|palabra| encaja(&texto, palabra))
        })
        .collect()
}

// ---------- Armar la vitrina ----------

fn texto_cantidad(n: usize) -> String {
    if n == 1 {
        "1 producto".to_string()
    } else {
        format!("{} productos", n)
    }
}

// Los botones que Juno ofrece según lo que se está viendo: ver todo, otra categoría, un tope de precio.
fn botones(categorias: &[Categoria], visibles: &[&Producto], filtros: &Filtros) -> Vec<String> {
    let mut botones: Vec<String> = Vec::new();
    if !filtros.vacio() {
        botones.push(VER_TODO.to_string());
    }
    if filtros.categoria.is_none() {
        for categoria in categorias {
            if botones.len() >= 3 {
                break;
            }
            if visibles.iter().any(|p| p.categoria_codigo == categoria.codigo) {
                botones.push(format!("Solo {}", categoria.nombre));
            }
        }
    }
    if filtros.precio_max.is_none() {
        // Un tope sirve si deja fuera a alguien y suma productos que el tope anterior no tenía.
        let mut utiles: Vec<u32> = Vec::new();
        let mut anterior = 0;
        for tope in TOPES {
            let hasta = visibles.iter().filter(|p| precio_de(p) <= f64::from(tope)).count();
            if hasta > anterior && hasta < visibles.len() {
                utiles.push(tope);
            }
            anterior = hasta;
        }
        let desde = utiles.len().saturating_sub(2);
        for tope in &utiles[desde..] {
            botones.push(format!("Menos de S/ {}", tope));
        }
    }
    botones
}

// Los filtros puestos en palabras, para mostrarlos en la franja de información.
fn describir(categorias: &[Categoria], filtros: &Filtros) -> Vec<String> {
    let mut texto = Vec::new();
    if let Some(codigo) = &filtros.categoria {
        if let Some(c) = categorias.iter().find(|c| &c.codigo == codigo) {
            texto.push(c.nombre.clone());
        }
    }
    if let Some(max) = filtros.precio_max {
        texto.push(format!("Hasta S/ {}", max));
    }
    if let Some(busqueda) = &filtros.busqueda {
        texto.push(format!("“{}”", busqueda));
    }
    texto
}

// La respuesta con la vitrina: el texto de Juno, los botones y la lista de productos para dibujar.
fn vitrina(
    categorias: &[Categoria],
    todos: &[Producto],
    filtros: &Filtros,
    mensaje: String,
    extras: &Extras,
) -> Respuesta {
    let visibles = aplicar(todos, filtros);
    let opciones = botones(categorias, &visibles, filtros);
    let productos = visibles.iter().map(|p| a_producto_catalogo(p, extras)).collect();
    Respuesta {
        texto: mensaje,
        opciones,
        componente: Some(Componente::Catalogo {
            filtros: describir(categorias, filtros),
            total: todos.len(),
            productos,
        }),
        espera: None,
        solicitud: None,
    }
}

// ---------- Los pasos ----------

// Abre el catálogo completo, sin filtros.
pub async fn abrir(pool: &PgPool, datos: &mut Datos) -> Result<Paso, sqlx::Error> {
    Filtros::default().guardar(datos);
    datos.categoria_elegida = None;
    datos.producto_elegido = None;
    let todos = repositorio::listar_productos(pool).await?;
    if todos.is_empty() {
        return Ok((
            r("Por ahora no tengo productos para mostrarte. Escríbenos y te ayudamos directamente.", &MENU),
            LIBRE,
        ));
    }
    let categorias = repositorio::listar_categorias(pool).await?;
    let extras = cargar_extras(pool).await;
    let mensaje = "Este es nuestro catálogo. Dime qué buscas y lo filtro por ti, o toca una opción.".to_string();
    Ok((vitrina(&categorias, &todos, &Filtros::default(), mensaje, &extras), CATALOGO))
}

// La vitrina completa, sin filtros, para dibujar la página /catalogo al abrirla (no es una respuesta de Juno: no hay conversación).
pub async fn vitrina_completa(pool: &PgPool) -> Result<Componente, sqlx::Error> {
    let todos = repositorio::listar_productos(pool).await?;
    let categorias = repositorio::listar_categorias(pool).await?;
    let extras = cargar_extras(pool).await;
    let filtros = Filtros::default();
    let visibles = aplicar(&todos, &filtros);
    let productos = visibles.iter().map(|p| a_producto_catalogo(p, &extras)).collect();
    Ok(Componente::Catalogo { filtros: describir(&categorias, &filtros), total: todos.len(), productos })
}

// Lo que completa la ficha de cada producto: sus características y las vistas de su galería.
#[derive(Default)]
pub struct Extras {
    caracteristicas: Vec<Caracteristica>,
    imagenes: Vec<Imagen>,
}

// Carga las fichas. Si todavía no se ejecutó 11_pedidos_y_fichas.sql (o falla la consulta), la vitrina se ve igual,
// solo que sin cajas destacadas ni galería: la ficha nunca debe impedir ver el catálogo.
pub async fn cargar_extras(pool: &PgPool) -> Extras {
    let caracteristicas = repositorio::listar_caracteristicas(pool).await.unwrap_or_else(|error| {
        eprintln!("No pude leer las características de los productos: {}", error);
        Vec::new()
    });
    let imagenes = repositorio::listar_imagenes(pool).await.unwrap_or_else(|error| {
        eprintln!("No pude leer las imágenes de los productos: {}", error);
        Vec::new()
    });
    Extras { caracteristicas, imagenes }
}

// El grupo especial que NO va en la tabla de especificaciones: son las cajas destacadas del resumen.
const GRUPO_RESUMEN: &str = "RESUMEN";

// Las cajas destacadas de un producto (en su orden).
pub fn chips_de(producto: &str, caracteristicas: &[Caracteristica]) -> Vec<Chip> {
    caracteristicas
        .iter()
        .filter(|c| c.producto == producto && c.grupo == GRUPO_RESUMEN)
        .map(|c| Chip {
            icono: c.icono.clone().unwrap_or_else(|| "marca".to_string()),
            titulo: c.titulo.clone(),
            valor: c.valor.clone(),
        })
        .collect()
}

// La tabla de especificaciones de un producto: sus filas agrupadas, en el orden en que aparece cada grupo.
pub fn grupos_de(producto: &str, caracteristicas: &[Caracteristica]) -> Vec<GrupoFicha> {
    let mut grupos: Vec<GrupoFicha> = Vec::new();
    for c in caracteristicas.iter().filter(|c| c.producto == producto && c.grupo != GRUPO_RESUMEN) {
        let fila = FilaFicha { titulo: c.titulo.clone(), valor: c.valor.clone() };
        match grupos.iter_mut().find(|g| g.nombre == c.grupo) {
            Some(grupo) => grupo.filas.push(fila),
            None => grupos.push(GrupoFicha { nombre: c.grupo.clone(), filas: vec![fila] }),
        }
    }
    grupos
}

fn a_producto_catalogo(p: &Producto, extras: &Extras) -> ProductoCatalogo {
    ProductoCatalogo {
        id: p.codigo.clone(),
        nombre: p.nombre.clone(),
        descripcion: p.descripcion.clone(),
        categoria: p.categoria_nombre.clone(),
        precio: format!("S/ {}", p.precio),
        unidades: p.stock.max(0),
        chips: chips_de(&p.codigo, &extras.caracteristicas),
        grupos: grupos_de(&p.codigo, &extras.caracteristicas),
        vistas: extras
            .imagenes
            .iter()
            .filter(|i| i.producto == p.codigo)
            .map(|i| Vista { etiqueta: i.etiqueta.clone(), imagen: i.url.clone() })
            .collect(),
    }
}

// Si la frase nombra un producto, lo deja como el elegido (para que «Apartar Mouse M170» sepa de cuál se habla).
pub async fn fijar_producto_nombrado(
    pool: &PgPool,
    datos: &mut Datos,
    texto: &str,
) -> Result<(), sqlx::Error> {
    let normal = normalizar(texto);
    let todos = repositorio::listar_productos(pool).await?;
    if let Some(producto) = producto_nombrado(&todos, &normal) {
        datos.producto_elegido = Some(producto.codigo.clone());
    }
    Ok(())
}

// La persona escribe con la vitrina abierta. Devuelve None cuando NO es para el catálogo (una cita, un servicio,
// un saludo...): entonces la atiende la conversación libre.
pub async fn paso_catalogo(
    pool: &PgPool,
    conversacion: &Conversacion,
    datos: &mut Datos,
    texto: &str,
) -> Result<Option<Paso>, sqlx::Error> {
    let normal = normalizar(texto).trim().to_string();
    let intencion = detectar_intencion(texto);

    // «Apartar Mouse M170»: sigue el flujo de apartados, con el producto nombrado.
    if matches!(intencion, Intencion::Apartar) {
        let antes = datos.producto_elegido.take();
        fijar_producto_nombrado(pool, datos, texto).await?;
        if datos.producto_elegido.is_none() {
            datos.producto_elegido = antes;
        }
        if datos.producto_elegido.is_none() {
            return Ok(Some((
                r(
                    "¿Cuál producto quieres apartar? Dime su nombre o toca «Apartar» en su tarjeta.",
                    &[VER_TODO],
                ),
                CATALOGO,
            )));
        }
        return Ok(Some(flujo_productos::iniciar_apartado(pool, conversacion, datos).await?));
    }

    if quiere_ver_todo(&normal) {
        return Ok(Some(abrir(pool, datos).await?));
    }

    let precio = precio_dicho(&normal);
    let es_del_catalogo = matches!(intencion, Intencion::Desconocida | Intencion::Productos)
        || (matches!(intencion, Intencion::Servicios) && precio.is_some());
    if !es_del_catalogo {
        return Ok(None);
    }

    let categorias = repositorio::listar_categorias(pool).await?;
    let todos = repositorio::listar_productos(pool).await?;
    if todos.is_empty() {
        return Ok(Some(abrir(pool, datos).await?));
    }

    let mut filtros = Filtros::desde(datos);
    let mut palabras = palabras_utiles(&normal);
    let mut cambio = false;

    if let Some(max) = precio {
        filtros.precio_max = Some(max);
        cambio = true;
    }
    if let Some(categoria) = categoria_nombrada(&categorias, &normal) {
        filtros.categoria = Some(categoria.codigo.clone());
        // Las palabras de la categoría ya no son una búsqueda aparte.
        let nombre = normalizar(&categoria.nombre);
        let raiz = nombre.trim().trim_end_matches('s').to_string();
        palabras.retain(|p| !nombre.contains(p.as_str()) && !p.contains(raiz.as_str()));
        cambio = true;
    }
    if !palabras.is_empty() {
        filtros.busqueda = Some(palabras.join(" "));
        cambio = true;
    }

    if !cambio {
        // Nada que filtrar: «Ver productos» vuelve a abrir la vitrina; otra cosa se le pide con más claridad.
        if matches!(intencion, Intencion::Productos) {
            return Ok(Some(abrir(pool, datos).await?));
        }
        let visibles = aplicar(&todos, &Filtros::desde(datos));
        let opciones = botones(&categorias, &visibles, &Filtros::desde(datos));
        let respuesta = Respuesta {
            texto: "Cuéntame qué buscas: una categoría, un precio máximo o el nombre del producto.".to_string(),
            opciones,
            componente: None,
            espera: None,
            solicitud: None,
        };
        return Ok(Some((respuesta, CATALOGO)));
    }

    let visibles = aplicar(&todos, &filtros);
    if visibles.is_empty() {
        // No se cambia nada de lo que la persona ya está viendo.
        let actuales = Filtros::desde(datos);
        let respuesta = Respuesta {
            texto: "No encontré productos con eso. Prueba con otra palabra o toca «Ver todo».".to_string(),
            opciones: botones(&categorias, &aplicar(&todos, &actuales), &actuales),
            componente: None,
            espera: None,
            solicitud: None,
        };
        return Ok(Some((respuesta, CATALOGO)));
    }

    filtros.guardar(datos);
    let extras = cargar_extras(pool).await;
    let mensaje = format!("Encontré {} para ti. ¿Quieres afinar más o añadir alguno a tu carrito?", texto_cantidad(visibles.len()));
    Ok(Some((vitrina(&categorias, &todos, &filtros, mensaje, &extras), CATALOGO)))
}

// Si la persona recarga la página con la vitrina abierta, estos botones siguen ahí.
pub fn opciones_de_reanudacion() -> Vec<String> {
    vec![VER_TODO.to_string()]
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn producto(codigo: &str, categoria: &str, nombre: &str, descripcion: &str, precio: &str, stock: i32) -> Producto {
        Producto {
            codigo: codigo.to_string(),
            categoria_codigo: categoria.to_string(),
            categoria_nombre: if categoria == "C1" { "Laptops" } else { "Accesorios" }.to_string(),
            nombre: nombre.to_string(),
            descripcion: descripcion.to_string(),
            precio: precio.to_string(),
            stock,
        }
    }

    fn categorias() -> Vec<Categoria> {
        vec![
            Categoria { codigo: "C1".to_string(), nombre: "Laptops".to_string() },
            Categoria { codigo: "C2".to_string(), nombre: "Accesorios".to_string() },
        ]
    }

    fn catalogo() -> Vec<Producto> {
        vec![
            producto("P1", "C1", "Laptop Lenovo IdeaPad 3", "Procesador Ryzen 5, 8 GB de RAM", "1899.00", 3),
            producto("P2", "C1", "Laptop HP 15", "Intel Core i3, 4 GB de RAM", "1299.00", 0),
            producto("P3", "C2", "Mouse inalámbrico M170", "Receptor USB, pila incluida", "45.50", 12),
            producto("P4", "C2", "Teclado USB", "Teclado básico en español", "59.90", 5),
        ]
    }

    #[test]
    fn lee_el_primer_numero_con_miles() {
        assert_eq!(primer_numero("menos de 500"), Some(500));
        assert_eq!(primer_numero("menos de s/ 1,500"), Some(1500));
        assert_eq!(primer_numero("hasta 1.500 soles"), Some(1500));
        assert_eq!(primer_numero("hasta 99.50"), Some(99));
        assert_eq!(primer_numero("sin numeros"), None);
    }

    #[test]
    fn entiende_un_tope_de_precio() {
        assert_eq!(precio_dicho("menos de s/ 500"), Some(500));
        assert_eq!(precio_dicho("algo debajo de 300"), Some(300));
        // Sin palabra de tope, un número suelto no es un precio.
        assert_eq!(precio_dicho("laptop 15"), None);
        assert_eq!(precio_dicho("menos de nada"), None);
    }

    #[test]
    fn encuentra_la_categoria_nombrada() {
        let cats = categorias();
        assert_eq!(categoria_nombrada(&cats, "solo laptops").map(|c| c.codigo.as_str()), Some("C1"));
        assert_eq!(categoria_nombrada(&cats, "algun accesorio").map(|c| c.codigo.as_str()), Some("C2"));
        // Si nombra dos o ninguna, no adivina.
        assert!(categoria_nombrada(&cats, "laptops y accesorios").is_none());
        assert!(categoria_nombrada(&cats, "mouse").is_none());
    }

    #[test]
    fn deja_solo_las_palabras_que_buscan() {
        assert_eq!(palabras_utiles("muestrame el mouse inalambrico"), vec!["mouse", "inalambrico"]);
        assert!(palabras_utiles("ver productos").is_empty());
        assert!(palabras_utiles("menos de 500").is_empty());
    }

    #[test]
    fn reconoce_el_producto_nombrado() {
        let todos = catalogo();
        let normal = normalizar("Apartar Mouse inalámbrico M170");
        assert_eq!(producto_nombrado(&todos, &normal).map(|p| p.codigo.as_str()), Some("P3"));
        assert!(producto_nombrado(&todos, "apartar algo").is_none());
    }

    #[test]
    fn reconoce_ver_todo() {
        assert!(quiere_ver_todo("ver todo"));
        assert!(quiere_ver_todo("muestrame el catalogo completo"));
        assert!(!quiere_ver_todo("solo laptops"));
    }

    #[test]
    fn filtra_por_categoria_precio_y_palabras() {
        let todos = catalogo();
        let por_categoria = Filtros { categoria: Some("C2".to_string()), ..Filtros::default() };
        assert_eq!(aplicar(&todos, &por_categoria).len(), 2);

        let por_precio = Filtros { precio_max: Some(100), ..Filtros::default() };
        assert_eq!(aplicar(&todos, &por_precio).len(), 2);

        let por_palabra = Filtros { busqueda: Some("mouses".to_string()), ..Filtros::default() };
        let encontrados = aplicar(&todos, &por_palabra);
        assert_eq!(encontrados.len(), 1);
        assert_eq!(encontrados[0].codigo, "P3");

        // Los tres a la vez.
        let todos_a_la_vez = Filtros {
            categoria: Some("C1".to_string()),
            precio_max: Some(1500),
            busqueda: Some("ram".to_string()),
        };
        let uno = aplicar(&todos, &todos_a_la_vez);
        assert_eq!(uno.len(), 1);
        assert_eq!(uno[0].codigo, "P2");
        assert_eq!(aplicar(&todos, &Filtros::default()).len(), 4);
    }

    #[test]
    fn ofrece_botones_que_separan_productos() {
        let todos = catalogo();
        let cats = categorias();
        let sin_filtros = Filtros::default();
        let visibles = aplicar(&todos, &sin_filtros);
        let botones = botones(&cats, &visibles, &sin_filtros);
        assert!(!botones.contains(&"Ver todo".to_string()));
        assert!(botones.contains(&"Solo Laptops".to_string()));
        assert!(botones.contains(&"Menos de S/ 100".to_string()));
        assert!(botones.iter().any(|b| b.starts_with("Menos de S/ ")));
        // Con filtros siempre se puede volver a todo.
        let con_filtro = Filtros { categoria: Some("C1".to_string()), ..Filtros::default() };
        let visibles = aplicar(&todos, &con_filtro);
        assert_eq!(botones_primero(&cats, &visibles, &con_filtro), "Ver todo");
    }

    fn botones_primero(cats: &[Categoria], visibles: &[&Producto], filtros: &Filtros) -> String {
        botones(cats, visibles, filtros).remove(0)
    }

    #[test]
    fn describe_los_filtros() {
        let cats = categorias();
        let f = Filtros {
            categoria: Some("C1".to_string()),
            precio_max: Some(500),
            busqueda: Some("ram".to_string()),
        };
        assert_eq!(describir(&cats, &f), vec!["Laptops", "Hasta S/ 500", "“ram”"]);
    }
}
