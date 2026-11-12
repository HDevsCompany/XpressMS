// panel_servicios.rs: el cerebro de Juno en el panel de servicios (solo Administrador). Aquí se decide qué responde y qué ficha arma
// cuando la persona dice «agrega limpieza de ventiladores», «súbele a 100 el cambio de teclado» o «quita el servicio X».
//
// Regla de este archivo (igual que motor.rs): NO toca la base de datos ni la red. Recibe lo que escribió la persona, el estado de la
// conversación (que guarda el navegador) y el catálogo actual, y devuelve la respuesta, la ficha en construcción (el «borrador») y,
// si hay que guardar algo, la ACCIÓN a ejecutar. Quien ejecuta (crear, editar, eliminar) es rutas_panel.rs: «la interfaz muestra,
// Juno ejecuta». Por eso todo se prueba solo con «cargo test».
//
// Juno NO guarda nada sin que la persona lo confirme con un botón (Crear servicio, Guardar cambio, Sí, eliminar) o con un «sí».
//
// La descripción la redacta una plantilla (no una inteligencia artificial): sirve de punto de partida y la persona puede dictar la suya.

use serde::{Deserialize, Serialize};

use super::motor::normalizar;
use super::panel_interprete::{self as lector, Busqueda, Equipo, Lectura, Palabra};

// ---------- Lo que se guarda en el navegador entre un mensaje y otro ----------

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Modo {
    #[default]
    #[serde(rename = "")]
    Ninguno,
    #[serde(rename = "crear")]
    Crear,
    #[serde(rename = "editar")]
    Editar,
    #[serde(rename = "eliminar")]
    Eliminar,
}

// Un dato que puede estar sin decir (dicho = false), dicho como «a cotizar» o «sin tiempo fijo» (dicho = true, valor = None) o con valor.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Campo {
    #[serde(default)]
    pub dicho: bool,
    #[serde(default)]
    pub valor: Option<String>,
}

impl Campo {
    fn con(valor: Option<String>) -> Campo {
        Campo { dicho: true, valor }
    }
}

// La ficha que se está armando. «imagen»: "" = sin imagen, "actual" = la que ya tenía, "nueva" = el navegador la subirá al confirmar,
// "quitar" = se le quitará la que tenía.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Ficha {
    #[serde(default)]
    pub nombre: String,
    #[serde(default)]
    pub descripcion: String,
    #[serde(default)]
    pub descripcion_propia: bool, // true = la dictó la persona (si cambia el nombre, no se vuelve a redactar)
    #[serde(default)]
    pub precio: Campo,
    #[serde(default)]
    pub tiempo: Campo,
    #[serde(default)]
    pub equipos: Option<Vec<String>>, // códigos de los tipos de equipo; None = todavía no se dijo
    #[serde(default)]
    pub aviso_precio: bool,
    #[serde(default)]
    pub imagen: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Estado {
    #[serde(default)]
    pub modo: Modo,
    #[serde(default)]
    pub codigo: String, // el servicio que se edita o elimina
    #[serde(default)]
    pub pregunta: String, // lo que Juno acaba de preguntar: «nombre», «precio», «tiempo», «equipos», «descripcion», «servicio_editar»…
    #[serde(default)]
    pub ficha: Ficha,
    #[serde(default)]
    pub original: Option<Ficha>, // al editar: cómo estaba el servicio (para mostrar el «antes»)
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Entrada {
    #[serde(default)]
    pub texto: String,
    #[serde(default)]
    pub accion: Option<String>,
    #[serde(default)]
    pub estado: Estado,
}

// ---------- Lo que Juno conoce del catálogo ----------

#[derive(Clone, Debug)]
pub struct ServicioCat {
    pub codigo: String,
    pub nombre: String,
    pub descripcion: String,
    pub precio: Option<String>,
    pub tiempo: Option<String>,
    pub equipos: Vec<String>, // códigos
    pub aviso_precio: bool,
    pub imagen: Option<String>, // la dirección de su imagen, si tiene
    pub protegido: bool,        // true = no se puede eliminar (el «Diagnóstico de equipo» que usa la página /servicios)
}

impl ServicioCat {
    fn ficha(&self) -> Ficha {
        Ficha {
            nombre: self.nombre.clone(),
            descripcion: self.descripcion.clone(),
            descripcion_propia: true,
            precio: Campo::con(self.precio.clone()),
            tiempo: Campo::con(self.tiempo.clone()),
            equipos: if self.equipos.is_empty() { None } else { Some(self.equipos.clone()) },
            aviso_precio: self.aviso_precio,
            imagen: if self.imagen.is_some() { "actual".to_string() } else { String::new() },
        }
    }
}

#[derive(Clone, Debug)]
pub struct EquipoCat {
    pub codigo: String,
    pub nombre: String,
}

#[derive(Clone, Debug, Default)]
pub struct Catalogo {
    pub servicios: Vec<ServicioCat>,
    pub equipos: Vec<EquipoCat>,
}

// ---------- Lo que Juno devuelve ----------

// Un botón bajo el mensaje. «tono»: "" normal, "v" confirmar, "a" editar (ámbar), "r" retroceder (rojo suave), "rd" eliminar (rojo).
// «accion»: lo que se envía al pulsarlo; si está vacío se envía el texto del botón como si lo hubiera escrito la persona.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Opcion {
    pub texto: String,
    pub tono: String,
    pub accion: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Dato {
    pub clave: String,  // nombre, descripcion, precio, tiempo, equipos
    pub titulo: String, // «Precio base»
    pub estado: String, // ok, jn (lo escribió Juno), falta, cambio, quitado
    pub valor: String,
    pub sub: String,
    pub antes: String, // al editar: el valor anterior
}

// La ficha en construcción tal como la dibuja la pantalla (anillo de avance, pasos, tarjetas y vista previa).
#[derive(Clone, Debug, Serialize)]
pub struct Borrador {
    pub modo: Modo,
    pub codigo: String,
    pub nombre: String,
    pub descripcion: String,
    pub precio: Option<String>,
    pub tiempo: Option<String>,
    pub equipos: Vec<String>, // nombres
    pub aviso_precio: bool,
    pub imagen: String,
    pub imagen_actual: Option<String>,
    pub datos: Vec<Dato>,
    pub completos: usize,
    pub total: usize,
    pub listo: bool,
    pub hay_cambios: bool,
    pub aviso_cambio: bool, // al editar: el aviso de precio no es el que tenía
}

#[derive(Clone, Debug, Serialize)]
pub struct Hecho {
    pub tipo: String, // creado, editado, eliminado
    pub codigo: String,
    pub nombre: String,
    pub imagen: String, // "nueva" = el navegador debe subir la imagen que eligió
}

#[derive(Clone, Debug, Serialize)]
pub struct Salida {
    pub respuesta: String, // los párrafos separados por una línea en blanco son burbujas distintas; **así** se resalta
    pub opciones: Vec<Opcion>,
    pub estado: Estado,
    pub borrador: Option<Borrador>,
    pub hecho: Option<Hecho>,
    pub foco: Vec<String>, // los datos que cambiaron en este turno
}

// Lo que hay que ejecutar antes de contestar de verdad.
#[derive(Clone, Debug, PartialEq)]
pub enum Accion {
    Crear,
    Editar,
    Eliminar,
}

pub struct Turno {
    pub salida: Salida,
    pub accion: Option<Accion>,
}

// Cómo salió lo que se ejecutó.
pub enum Resultado {
    Hecho(String), // el código del servicio creado (o el mismo, al editar o eliminar)
    Rechazo(String),
}

// ---------- Utilidades ----------

fn op(texto: &str, tono: &str, accion: &str) -> Opcion {
    Opcion { texto: texto.to_string(), tono: tono.to_string(), accion: accion.to_string() }
}

// Resalta un valor en el mensaje. Se quitan los asteriscos del valor para no romper el resaltado.
fn rs(valor: &str) -> String {
    format!("**{}**", valor.replace('*', ""))
}

fn unir(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        n => format!("{} y {}", items[..n - 1].join(", "), items[n - 1]),
    }
}

fn dinero(precio: &Option<String>) -> String {
    match precio {
        Some(valor) => format!("S/ {}", valor),
        None => "A cotizar".to_string(),
    }
}

fn tiempo_texto(tiempo: &Option<String>) -> String {
    tiempo.clone().unwrap_or_else(|| "Sin tiempo fijo".to_string())
}

fn ordenar(cat: &Catalogo, codigos: &[String]) -> Vec<String> {
    cat.equipos.iter().map(|q| q.codigo.clone()).filter(|c| codigos.contains(c)).collect()
}

fn nombres_equipos(cat: &Catalogo, codigos: &[String]) -> Vec<String> {
    cat.equipos.iter().filter(|q| codigos.contains(&q.codigo)).map(|q| q.nombre.clone()).collect()
}

fn servicio<'a>(cat: &'a Catalogo, codigo: &str) -> Option<&'a ServicioCat> {
    cat.servicios.iter().find(|s| s.codigo == codigo)
}

// ¿Ya hay otro servicio con ese nombre? («excepto» es el que se está editando.)
fn repetido<'a>(cat: &'a Catalogo, nombre: &str, excepto: &str) -> Option<&'a ServicioCat> {
    let buscado = normalizar(nombre);
    cat.servicios.iter().find(|s| s.codigo != excepto && normalizar(&s.nombre) == buscado)
}

fn equipos_del_lector(cat: &Catalogo) -> Vec<Equipo> {
    cat.equipos.iter().map(|q| Equipo { codigo: q.codigo.clone(), nombre: q.nombre.clone() }).collect()
}

fn menu() -> Vec<Opcion> {
    vec![
        op("Agregar un servicio", "", ""),
        op("Cambiar un precio", "", ""),
        op("Servicios sin precio", "", ""),
        op("Quitar un servicio", "", ""),
    ]
}

// Los botones para elegir qué dato cambiar.
fn campos(con_descartar: bool) -> Vec<Opcion> {
    let mut lista = vec![
        op("Nombre", "", "pedir:nombre"),
        op("Descripción", "", "pedir:descripcion"),
        op("Precio", "", "pedir:precio"),
        op("Tiempo", "", "pedir:tiempo"),
        op("Equipos", "", "pedir:equipos"),
        op("Imagen", "", "pedir:imagen"),
    ];
    if con_descartar {
        lista.push(op("Descartar", "r", "descartar"));
    }
    lista
}

// ---------- La descripción que redacta Juno ----------

// Una descripción sencilla a partir del nombre. No es una inteligencia artificial: son plantillas por tipo de trabajo.
pub fn redactar_descripcion(nombre: &str) -> String {
    let limpio = nombre.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut palabras = limpio.split(' ');
    let primera = palabras.next().unwrap_or("");
    let resto = palabras.collect::<Vec<_>>().join(" ").to_lowercase();
    let con_resto = |prefijo: &str, sin_resto: &str| -> String {
        if resto.is_empty() {
            sin_resto.to_string()
        } else {
            format!("{} {}", prefijo, resto)
        }
    };
    match normalizar(primera).as_str() {
        "cambio" => format!(
            "{} en tu equipo y comprobamos que todo funcione bien antes de entregártelo.",
            con_resto("Hacemos el cambio", "Hacemos el cambio que tu equipo necesita")
        ),
        "limpieza" => format!(
            "{} de tu equipo para que trabaje mejor y se mantenga fresco.",
            con_resto("Hacemos la limpieza", "Hacemos la limpieza")
        ),
        "reparacion" => format!(
            "{} de tu equipo y comprobamos que quede funcionando bien.",
            con_resto("Hacemos la reparación", "Hacemos la reparación")
        ),
        "instalacion" => format!(
            "{} y lo dejamos configurado y listo para usar.",
            con_resto("Hacemos la instalación", "Hacemos la instalación")
        ),
        "mantenimiento" => format!(
            "{}: revisión, limpieza y ajustes para que tu equipo siga funcionando bien.",
            con_resto("Hacemos el mantenimiento", "Hacemos el mantenimiento de tu equipo")
        ),
        "formateo" => format!(
            "{} y dejamos tu equipo limpio, ordenado y listo para usar.",
            con_resto("Hacemos el formateo", "Hacemos el formateo")
        ),
        "diagnostico" | "revision" => {
            "Revisamos tu equipo para encontrar la falla y te explicamos qué necesita.".to_string()
        }
        _ => format!(
            "{}: revisamos tu equipo, hacemos el trabajo con cuidado y comprobamos que todo funcione bien antes de entregártelo.",
            limpio
        ),
    }
}

// ---------- El borrador que se dibuja ----------

fn construir_borrador(cat: &Catalogo, e: &Estado) -> Borrador {
    let f = &e.ficha;
    let original = e.original.as_ref();
    let edita = e.modo == Modo::Editar;
    let mut datos: Vec<Dato> = Vec::new();

    let dato = |clave: &str, titulo: &str, estado: &str, valor: String, sub: &str, antes: String| Dato {
        clave: clave.to_string(),
        titulo: titulo.to_string(),
        estado: estado.to_string(),
        valor,
        sub: sub.to_string(),
        antes,
    };

    // Nombre
    if f.nombre.is_empty() {
        datos.push(dato("nombre", "Nombre", "falta", String::new(), "", String::new()));
    } else if let Some(o) = original.filter(|o| edita && o.nombre != f.nombre) {
        datos.push(dato("nombre", "Nombre", "cambio", f.nombre.clone(), "", o.nombre.clone()));
    } else {
        datos.push(dato("nombre", "Nombre", "ok", f.nombre.clone(), "", String::new()));
    }

    // Descripción
    if f.descripcion.is_empty() {
        datos.push(dato("descripcion", "Descripción", "falta", String::new(), "", String::new()));
    } else if let Some(o) = original.filter(|o| edita && o.descripcion != f.descripcion) {
        datos.push(dato("descripcion", "Descripción", "cambio", f.descripcion.clone(), "", o.descripcion.clone()));
    } else if !f.descripcion_propia {
        datos.push(dato("descripcion", "Descripción", "jn", f.descripcion.clone(), "Redactada por Juno", String::new()));
    } else {
        datos.push(dato("descripcion", "Descripción", "ok", f.descripcion.clone(), "", String::new()));
    }

    // Precio base
    if !f.precio.dicho {
        datos.push(dato("precio", "Precio base", "falta", String::new(), "", String::new()));
    } else {
        let sub = if f.precio.valor.is_none() { "Se cotiza al revisar el equipo" } else { "" };
        match original.filter(|o| edita && o.precio != f.precio) {
            Some(o) => datos.push(dato("precio", "Precio base", "cambio", dinero(&f.precio.valor), sub, dinero(&o.precio.valor))),
            None => datos.push(dato("precio", "Precio base", "ok", dinero(&f.precio.valor), sub, String::new())),
        }
    }

    // Tiempo estimado
    if !f.tiempo.dicho {
        datos.push(dato("tiempo", "Tiempo estimado", "falta", String::new(), "", String::new()));
    } else {
        match original.filter(|o| edita && o.tiempo != f.tiempo) {
            Some(o) => {
                let estado = if f.tiempo.valor.is_none() { "quitado" } else { "cambio" };
                datos.push(dato("tiempo", "Tiempo estimado", estado, tiempo_texto(&f.tiempo.valor), "", tiempo_texto(&o.tiempo.valor)));
            }
            None => datos.push(dato("tiempo", "Tiempo estimado", "ok", tiempo_texto(&f.tiempo.valor), "", String::new())),
        }
    }

    // Equipos
    let codigos = f.equipos.clone().unwrap_or_default();
    let nombres = nombres_equipos(cat, &codigos);
    if nombres.is_empty() {
        datos.push(dato("equipos", "Sirve para", "falta", String::new(), "", String::new()));
    } else {
        let antes = original
            .filter(|o| edita && o.equipos != f.equipos)
            .map(|o| nombres_equipos(cat, &o.equipos.clone().unwrap_or_default()).join(", "));
        match antes {
            Some(antes) => datos.push(dato("equipos", "Sirve para", "cambio", nombres.join(", "), "", antes)),
            None => datos.push(dato("equipos", "Sirve para", "ok", nombres.join(", "), "", String::new())),
        }
    }

    let completos = datos.iter().filter(|d| d.estado != "falta").count();
    let total = datos.len();
    let hay_cambios = edita
        && (datos.iter().any(|d| d.estado == "cambio" || d.estado == "quitado")
            || original.is_some_and(|o| o.aviso_precio != f.aviso_precio || o.imagen != f.imagen));
    let imagen_actual = servicio(cat, &e.codigo).and_then(|s| s.imagen.clone());

    Borrador {
        modo: e.modo,
        codigo: e.codigo.clone(),
        nombre: f.nombre.clone(),
        descripcion: f.descripcion.clone(),
        precio: f.precio.valor.clone(),
        tiempo: f.tiempo.valor.clone(),
        equipos: nombres,
        aviso_precio: f.aviso_precio,
        imagen: f.imagen.clone(),
        imagen_actual,
        datos,
        completos,
        total,
        listo: completos == total,
        hay_cambios,
        aviso_cambio: edita && original.is_some_and(|o| o.aviso_precio != f.aviso_precio),
    }
}

// Lo que todavía falta, en orden: (clave, cómo se dice en el mensaje).
fn faltan(b: &Borrador) -> Vec<(String, &'static str)> {
    b.datos
        .iter()
        .filter(|d| d.estado == "falta")
        .map(|d| {
            let frase = match d.clave.as_str() {
                "nombre" => "el nombre",
                "descripcion" => "la descripción",
                "precio" => "el precio",
                "tiempo" => "el tiempo estimado",
                _ => "para qué equipos sirve",
            };
            (d.clave.clone(), frase)
        })
        .collect()
}

fn salida(cat: &Catalogo, estado: Estado, respuesta: String, opciones: Vec<Opcion>, foco: Vec<String>) -> Salida {
    let borrador = if estado.modo == Modo::Ninguno { None } else { Some(construir_borrador(cat, &estado)) };
    Salida { respuesta, opciones, estado, borrador, hecho: None, foco }
}

fn turno(salida: Salida) -> Turno {
    Turno { salida, accion: None }
}

fn con_accion(salida: Salida, accion: Accion) -> Turno {
    Turno { salida, accion: Some(accion) }
}

// Los botones que corresponden después de un cambio en la ficha.
fn opciones_de_ficha(cat: &Catalogo, e: &Estado, b: &Borrador) -> Vec<Opcion> {
    if e.modo == Modo::Editar {
        if b.hay_cambios {
            return vec![op("Guardar cambio", "v", "guardar"), op("Cambiar algo más", "a", "cambiar"), op("Descartar", "r", "descartar")];
        }
        return campos(true);
    }
    if b.listo {
        return vec![op("Crear servicio", "v", "crear"), op("Cambiar algo", "a", "cambiar"), op("Descartar", "r", "descartar")];
    }
    let mut lista: Vec<Opcion> = Vec::new();
    match e.pregunta.as_str() {
        "precio" => lista.push(op("A cotizar", "", "")),
        "tiempo" => lista.push(op("Sin tiempo fijo", "", "")),
        "equipos" => {
            for q in cat.equipos.iter().take(5) {
                lista.push(op(&q.nombre, "", ""));
            }
            if cat.equipos.len() > 1 {
                lista.push(op("Todos los equipos", "", ""));
            }
        }
        _ => {}
    }
    lista.push(op("Descartar", "r", "descartar"));
    lista
}

// Lo próximo que Juno pregunta, según lo que falte.
fn proxima_pregunta(b: &Borrador) -> String {
    faltan(b).first().map(|(clave, _)| if clave == "descripcion" { String::new() } else { clave.clone() }).unwrap_or_default()
}

// ---------- Lo que cambió en este turno ----------

// (clave, ya_tenia): «ya_tenia» es true cuando ese dato ya estaba dicho antes (entonces se cambió en vez de anotarse).
type Cambios = Vec<(&'static str, bool)>;

#[derive(Default)]
struct Aplicado {
    cambios: Cambios,
    avisos: Vec<String>, // cosas que Juno debe contar tal cual (nombre repetido, cómo poner la imagen…)
}

fn foco_de(cambios: &Cambios) -> Vec<String> {
    let mut lista: Vec<String> = Vec::new();
    for (clave, _) in cambios {
        let dato = match *clave {
            "nombre" => "nombre",
            "descripcion" | "descripcion_auto" => "descripcion",
            "precio" | "q_precio" => "precio",
            "tiempo" | "q_tiempo" => "tiempo",
            "equipos" => "equipos",
            _ => continue,
        };
        if !lista.iter().any(|x| x == dato) {
            lista.push(dato.to_string());
        }
    }
    lista
}

fn frase_de(cat: &Catalogo, e: &Estado, clave: &str, ya_tenia: bool) -> String {
    let f = &e.ficha;
    let cambia = ya_tenia || e.modo == Modo::Editar;
    match clave {
        "nombre" if cambia => format!("cambié el nombre a {}", rs(&f.nombre)),
        "nombre" => "ya tengo el nombre".to_string(),
        "descripcion_auto" if cambia => "ajusté la descripción".to_string(),
        "descripcion_auto" => "agregué una descripción".to_string(),
        "descripcion" => "guardé la descripción que me diste".to_string(),
        "precio" => match (&f.precio.valor, cambia) {
            (Some(v), true) => format!("cambié el precio a {}", rs(&format!("S/ {}", v))),
            (Some(v), false) => format!("anoté el precio en {}", rs(&format!("S/ {}", v))),
            (None, _) => "dejé el precio a cotizar".to_string(),
        },
        "tiempo" => match (&f.tiempo.valor, cambia) {
            (Some(v), true) => format!("cambié el tiempo a {}", rs(v)),
            (Some(v), false) => format!("anoté el tiempo en {}", rs(v)),
            (None, _) => "dejé el tiempo sin fijar".to_string(),
        },
        "equipos" => {
            let nombres = nombres_equipos(cat, &f.equipos.clone().unwrap_or_default());
            if cambia {
                format!("ahora sirve para {}", rs(&nombres.join(", ")))
            } else {
                format!("anoté los equipos: {}", rs(&nombres.join(", ")))
            }
        }
        "q_precio" => {
            if e.modo == Modo::Editar {
                "dejé el precio a cotizar".to_string()
            } else {
                "quité el precio".to_string()
            }
        }
        "q_tiempo" => {
            if e.modo == Modo::Editar {
                "quité el tiempo estimado".to_string()
            } else {
                "quité el tiempo".to_string()
            }
        }
        "aviso_si" => "activé el aviso de que el precio puede cambiar".to_string(),
        "aviso_no" => "quité el aviso de que el precio puede cambiar".to_string(),
        "imagen_nueva" => "dejé lista la imagen (se sube cuando confirmes)".to_string(),
        "imagen_quitar" => "quité la imagen".to_string(),
        _ => String::new(),
    }
}

// Aplica a la ficha lo que se entendió del mensaje.
fn aplicar(cat: &Catalogo, e: &mut Estado, l: Lectura) -> Aplicado {
    let mut a = Aplicado::default();

    if let Some(nombre) = l.nombre {
        if let Some(otro) = repetido(cat, &nombre, &e.codigo) {
            a.avisos.push(format!("Ya existe un servicio llamado {}, elige otro nombre.", rs(&otro.nombre)));
        } else if nombre != e.ficha.nombre {
            let tenia = !e.ficha.nombre.is_empty();
            e.ficha.nombre = nombre;
            a.cambios.push(("nombre", tenia));
            if !e.ficha.descripcion_propia {
                let nueva = redactar_descripcion(&e.ficha.nombre);
                if nueva != e.ficha.descripcion {
                    let tenia_descripcion = !e.ficha.descripcion.is_empty();
                    e.ficha.descripcion = nueva;
                    a.cambios.push(("descripcion_auto", tenia_descripcion));
                }
            }
        }
    }
    if let Some(texto) = l.descripcion {
        if texto != e.ficha.descripcion {
            e.ficha.descripcion = texto;
            e.ficha.descripcion_propia = true;
            a.cambios.push(("descripcion", true));
        }
    }
    if let Some(precio) = l.precio {
        let nuevo = Campo::con(precio);
        if nuevo != e.ficha.precio {
            let tenia = e.ficha.precio.dicho;
            e.ficha.precio = nuevo;
            a.cambios.push(("precio", tenia));
        }
    }
    if let Some(tiempo) = l.tiempo {
        let nuevo = Campo::con(tiempo);
        if nuevo != e.ficha.tiempo {
            let tenia = e.ficha.tiempo.dicho;
            e.ficha.tiempo = nuevo;
            a.cambios.push(("tiempo", tenia));
        }
    }
    if l.equipos.is_some() || !l.equipos_quitar.is_empty() {
        let actual = e.ficha.equipos.clone().unwrap_or_default();
        let mut lista: Vec<String> = if l.equipos_suman { actual.clone() } else if l.equipos.is_some() { Vec::new() } else { actual.clone() };
        for codigo in l.equipos.clone().unwrap_or_default() {
            if !lista.contains(&codigo) {
                lista.push(codigo);
            }
        }
        lista.retain(|c| !l.equipos_quitar.contains(c));
        let lista = ordenar(cat, &lista);
        let nuevo = if lista.is_empty() { None } else { Some(lista) };
        if nuevo != e.ficha.equipos {
            let tenia = e.ficha.equipos.is_some();
            e.ficha.equipos = nuevo;
            a.cambios.push(("equipos", tenia));
        }
    }
    for campo in &l.quitar {
        match *campo {
            "tiempo" => {
                let nuevo = if e.modo == Modo::Editar { Campo::con(None) } else { Campo::default() };
                if nuevo != e.ficha.tiempo {
                    e.ficha.tiempo = nuevo;
                    a.cambios.push(("q_tiempo", true));
                }
            }
            "precio" => {
                let nuevo = if e.modo == Modo::Editar { Campo::con(None) } else { Campo::default() };
                if nuevo != e.ficha.precio {
                    e.ficha.precio = nuevo;
                    a.cambios.push(("q_precio", true));
                }
            }
            _ => {}
        }
    }
    if let Some(aviso) = l.aviso {
        if aviso != e.ficha.aviso_precio {
            e.ficha.aviso_precio = aviso;
            a.cambios.push((if aviso { "aviso_si" } else { "aviso_no" }, true));
        }
    }
    if l.quitar_imagen && quitar_imagen(e) {
        a.cambios.push(("imagen_quitar", true));
    }
    if l.pedir_imagen && !l.quitar_imagen {
        a.avisos.push("Para la imagen usa el botón **Agregar imagen** de la ficha: yo no puedo recibir archivos por el chat.".to_string());
    }
    a
}

// Quita la imagen de la ficha. Devuelve false si no había nada que quitar.
fn quitar_imagen(e: &mut Estado) -> bool {
    match e.ficha.imagen.as_str() {
        "nueva" => {
            // Se descarta la que se había elegido; si el servicio tenía una, vuelve a quedar la de antes.
            e.ficha.imagen = e.original.as_ref().map(|o| o.imagen.clone()).unwrap_or_default();
            true
        }
        "actual" => {
            e.ficha.imagen = "quitar".to_string();
            true
        }
        _ => false,
    }
}

// El mensaje de Juno después de cambiar la ficha.
fn mensaje_de_ficha(cat: &Catalogo, e: &Estado, a: &Aplicado, b: &Borrador, primera: bool) -> String {
    let frases: Vec<String> = a.cambios.iter().map(|(clave, ya)| frase_de(cat, e, clave, *ya)).filter(|f| !f.is_empty()).collect();
    let mut partes: Vec<String> = Vec::new();

    if e.modo == Modo::Crear && primera && b.listo && a.avisos.is_empty() {
        partes.push(format!("Listo, armé la ficha de {} con su descripción. ¿La creo?", rs(&e.ficha.nombre)));
        return partes.join(" ");
    }
    if !frases.is_empty() {
        partes.push(format!("Listo, {}.", unir(&frases)));
    }
    partes.extend(a.avisos.iter().cloned());

    if e.modo == Modo::Editar {
        if b.hay_cambios {
            partes.push("¿Guardo el cambio?".to_string());
        } else if frases.is_empty() && a.avisos.is_empty() {
            partes.push("Eso ya estaba así. ¿Qué otra cosa quieres cambiar?".to_string());
        } else {
            partes.push("¿Qué otra cosa quieres cambiar?".to_string());
        }
    } else {
        let pendientes = faltan(b);
        if pendientes.is_empty() {
            partes.push("Con eso la ficha está completa. ¿La creo?".to_string());
        } else {
            let lista: Vec<String> = pendientes.iter().map(|(_, f)| f.to_string()).collect();
            partes.push(format!("Para terminar faltaría: {}.", unir(&lista)));
        }
    }
    partes.join(" ")
}

// ---------- Entrada principal ----------

pub fn responder(cat: &Catalogo, usuario: &str, entrada: &Entrada) -> Turno {
    let estado = entrada.estado.clone();
    let accion = entrada.accion.as_deref().unwrap_or("").trim().to_string();
    let texto: String = entrada.texto.trim().chars().take(600).collect();

    if !accion.is_empty() {
        if let Some(turno_) = por_accion(cat, usuario, &estado, &accion) {
            return turno_;
        }
    }
    por_texto(cat, usuario, estado, &texto)
}

fn bienvenida(cat: &Catalogo, usuario: &str) -> Turno {
    let primer_nombre = usuario.split_whitespace().next().unwrap_or("").to_string();
    let saludo = if primer_nombre.is_empty() { "Hola".to_string() } else { format!("Hola, {}", primer_nombre) };
    let texto = format!(
        "{}. Cuéntame qué servicio quieres agregar y yo armo la ficha\n\nTambién puedo editar sus datos y eliminar algún servicio",
        saludo
    );
    turno(salida(cat, Estado::default(), texto, menu(), vec![]))
}

fn descartar(cat: &Catalogo, e: &Estado) -> Turno {
    let texto = match e.modo {
        Modo::Crear => "Listo, descarté la ficha. ¿Qué hacemos ahora?",
        Modo::Editar => "Listo, descarté los cambios. ¿Qué hacemos ahora?",
        Modo::Eliminar => "Listo, no eliminé nada. ¿Qué hacemos ahora?",
        Modo::Ninguno => "No hay nada que descartar. ¿Qué hacemos ahora?",
    };
    turno(salida(cat, Estado::default(), texto.to_string(), menu(), vec![]))
}

fn por_accion(cat: &Catalogo, usuario: &str, e: &Estado, accion: &str) -> Option<Turno> {
    match accion {
        "inicio" => Some(bienvenida(cat, usuario)),
        "descartar" => Some(descartar(cat, e)),
        "crear" => Some(confirmar_crear(cat, e)),
        "guardar" => Some(confirmar_guardar(cat, e)),
        "eliminar_si" => Some(confirmar_eliminar(cat, e)),
        "cambiar" => Some(que_cambiar(cat, e)),
        "aviso:1" | "aviso:0" => Some(poner_aviso(cat, e, accion == "aviso:1")),
        "imagen_nueva" => Some(imagen_nueva(cat, e)),
        "imagen_quitar" => Some(imagen_quitada(cat, e)),
        "imagen_restaurar" => Some(imagen_restaurada(cat, e)),
        _ => {
            if let Some(que) = accion.strip_prefix("pedir:") {
                return Some(pedir(cat, e, que));
            }
            if let Some(codigo) = accion.strip_prefix("editar:") {
                return Some(entrar(cat, e, codigo, Modo::Editar, None));
            }
            if let Some(codigo) = accion.strip_prefix("eliminar:") {
                return Some(entrar(cat, e, codigo, Modo::Eliminar, None));
            }
            None
        }
    }
}

// Mensaje para quien intenta empezar otra cosa con una ficha a medias.
fn primero_termina(cat: &Catalogo, e: &Estado) -> Turno {
    let nombre = if e.ficha.nombre.is_empty() { "la ficha".to_string() } else { format!("la ficha de {}", rs(&e.ficha.nombre)) };
    let texto = format!("Primero terminemos {}: puedes confirmarla o descartarla.", nombre);
    let b = construir_borrador(cat, e);
    let opciones = if e.modo == Modo::Eliminar {
        vec![op("Sí, eliminar", "rd", "eliminar_si"), op("No, dejarlo", "", "descartar")]
    } else {
        let mut lista = opciones_de_ficha(cat, e, &b);
        if !lista.iter().any(|o| o.accion == "descartar") {
            lista.push(op("Descartar", "r", "descartar"));
        }
        lista
    };
    turno(salida(cat, e.clone(), texto, opciones, vec![]))
}

fn hay_trabajo_en_marcha(e: &Estado) -> bool {
    match e.modo {
        Modo::Ninguno => false,
        Modo::Crear => e.ficha != Ficha::default(),
        Modo::Editar | Modo::Eliminar => true,
    }
}

// Empieza a editar o a eliminar el servicio con ese código. «cambios» son los cambios que venían en el mismo mensaje.
fn entrar(cat: &Catalogo, actual: &Estado, codigo: &str, modo: Modo, lectura: Option<Lectura>) -> Turno {
    if hay_trabajo_en_marcha(actual) {
        return primero_termina(cat, actual);
    }
    let s = match servicio(cat, codigo) {
        Some(s) => s,
        None => {
            return turno(salida(
                cat,
                Estado::default(),
                "No encontré ese servicio. Quizá ya lo eliminaron: recarga la página y vuelve a intentarlo.".to_string(),
                menu(),
                vec![],
            ))
        }
    };
    if modo == Modo::Eliminar {
        if s.protegido {
            let texto = format!(
                "{} no se puede eliminar: la página de servicios lo usa para agendar las citas de diagnóstico. Sí puedes editar su precio o su descripción.",
                rs(&s.nombre)
            );
            return turno(salida(cat, Estado::default(), texto, vec![op("Editar", "a", &format!("editar:{}", s.codigo)), op("Listo", "", "descartar")], vec![]));
        }
        let e = Estado { modo: Modo::Eliminar, codigo: s.codigo.clone(), pregunta: String::new(), ficha: s.ficha(), original: Some(s.ficha()) };
        let texto = format!("¿Seguro que quieres eliminar {}? Dejará de verse en la página de servicios.", rs(&s.nombre));
        return turno(salida(cat, e, texto, vec![op("Sí, eliminar", "rd", "eliminar_si"), op("No, dejarlo", "", "descartar")], vec![]));
    }

    let mut e = Estado { modo: Modo::Editar, codigo: s.codigo.clone(), pregunta: String::new(), ficha: s.ficha(), original: Some(s.ficha()) };
    match lectura {
        Some(l) if !l.vacia() => {
            let a = aplicar(cat, &mut e, l);
            let b = construir_borrador(cat, &e);
            let mut texto = mensaje_de_ficha(cat, &e, &a, &b, true);
            if !a.cambios.is_empty() {
                texto = texto.replacen("Listo, ", &format!("Listo, en {} ", rs(&s.nombre)), 1);
            }
            let opciones = opciones_de_ficha(cat, &e, &b);
            let foco = foco_de(&a.cambios);
            turno(salida(cat, e, texto, opciones, foco))
        }
        _ => {
            let texto = format!("Estás editando {}. ¿Qué quieres cambiar?", rs(&s.nombre));
            turno(salida(cat, e, texto, campos(true), vec![]))
        }
    }
}

fn que_cambiar(cat: &Catalogo, e: &Estado) -> Turno {
    if e.modo != Modo::Crear && e.modo != Modo::Editar {
        return turno(salida(cat, Estado::default(), "¿Qué hacemos? Puedo agregar un servicio, cambiar sus datos o eliminarlo.".to_string(), menu(), vec![]));
    }
    let mut nuevo = e.clone();
    nuevo.pregunta = String::new();
    let texto = "¿Qué quieres cambiar? Elige un dato o dime directamente lo que quieres, por ejemplo: **el precio es 150**.".to_string();
    turno(salida(cat, nuevo, texto, campos(true), vec![]))
}

fn pedir(cat: &Catalogo, e: &Estado, que: &str) -> Turno {
    if e.modo != Modo::Crear && e.modo != Modo::Editar {
        return turno(salida(cat, Estado::default(), "Primero dime qué servicio quieres agregar o editar.".to_string(), menu(), vec![]));
    }
    let mut nuevo = e.clone();
    let (pregunta, texto, opciones): (&str, String, Vec<Opcion>) = match que {
        "nombre" => ("nombre", "¿Cómo se llamará el servicio? Escribe el nombre.".to_string(), vec![]),
        "descripcion" => (
            "descripcion",
            "Escribe la descripción tal como quieres que la lean los clientes (entre 10 y 500 caracteres).".to_string(),
            vec![],
        ),
        "precio" => (
            "precio",
            "¿Cuál es el precio base? Escríbelo en soles, o dime que se cotiza.".to_string(),
            vec![op("A cotizar", "", "")],
        ),
        "tiempo" => (
            "tiempo",
            "¿Cuánto tarda normalmente? Por ejemplo: **2 días** o **1 a 2 días**.".to_string(),
            vec![op("Sin tiempo fijo", "", "")],
        ),
        "equipos" => {
            let mut lista: Vec<Opcion> = cat.equipos.iter().take(5).map(|q| op(&q.nombre, "", "")).collect();
            if cat.equipos.len() > 1 {
                lista.push(op("Todos los equipos", "", ""));
            }
            ("equipos", "¿Para qué tipos de equipo sirve? Puedes elegir varios.".to_string(), lista)
        }
        _ => (
            "",
            "Para la imagen usa el botón **Agregar imagen** de la ficha. Se sube cuando confirmes.".to_string(),
            vec![],
        ),
    };
    nuevo.pregunta = pregunta.to_string();
    let mut opciones = opciones;
    opciones.push(op("Descartar", "r", "descartar"));
    turno(salida(cat, nuevo, texto, opciones, vec![]))
}

fn poner_aviso(cat: &Catalogo, e: &Estado, activo: bool) -> Turno {
    if e.modo != Modo::Crear && e.modo != Modo::Editar {
        return turno(salida(cat, e.clone(), "Primero dime qué servicio quieres agregar o editar.".to_string(), menu(), vec![]));
    }
    let mut nuevo = e.clone();
    nuevo.ficha.aviso_precio = activo;
    let b = construir_borrador(cat, &nuevo);
    let texto = if activo {
        "Listo, la ficha ahora avisa que **el precio puede cambiar**.".to_string()
    } else {
        "Listo, la ficha ya no avisa que el precio puede cambiar.".to_string()
    };
    let opciones = opciones_de_ficha(cat, &nuevo, &b);
    turno(salida(cat, nuevo, texto, opciones, vec![]))
}

fn imagen_nueva(cat: &Catalogo, e: &Estado) -> Turno {
    if e.modo != Modo::Crear && e.modo != Modo::Editar {
        return turno(salida(cat, e.clone(), "Primero dime qué servicio quieres agregar o editar.".to_string(), menu(), vec![]));
    }
    let mut nuevo = e.clone();
    nuevo.ficha.imagen = "nueva".to_string();
    let b = construir_borrador(cat, &nuevo);
    let texto = "Listo, la imagen quedó lista. Se sube cuando confirmes la ficha.".to_string();
    let opciones = opciones_de_ficha(cat, &nuevo, &b);
    turno(salida(cat, nuevo, texto, opciones, vec![]))
}

fn imagen_quitada(cat: &Catalogo, e: &Estado) -> Turno {
    if e.modo != Modo::Crear && e.modo != Modo::Editar {
        return turno(salida(cat, e.clone(), "Primero dime qué servicio quieres agregar o editar.".to_string(), menu(), vec![]));
    }
    let mut nuevo = e.clone();
    if !quitar_imagen(&mut nuevo) {
        // La imagen elegida en el navegador se descartó sin que Juno la conociera: se deja la ficha como estaba.
        nuevo.ficha.imagen = e.ficha.imagen.clone();
    }
    let b = construir_borrador(cat, &nuevo);
    let texto = "Listo, quité la imagen.".to_string();
    let opciones = opciones_de_ficha(cat, &nuevo, &b);
    turno(salida(cat, nuevo, texto, opciones, vec![]))
}

// «Deshacer» sobre una imagen que se iba a quitar: vuelve la que tenía.
fn imagen_restaurada(cat: &Catalogo, e: &Estado) -> Turno {
    if e.modo != Modo::Editar {
        return turno(salida(cat, e.clone(), "Primero dime qué servicio quieres editar.".to_string(), menu(), vec![]));
    }
    let mut nuevo = e.clone();
    if nuevo.ficha.imagen == "quitar" {
        nuevo.ficha.imagen = "actual".to_string();
    }
    let b = construir_borrador(cat, &nuevo);
    let texto = "Listo, la imagen se queda como estaba.".to_string();
    let opciones = opciones_de_ficha(cat, &nuevo, &b);
    turno(salida(cat, nuevo, texto, opciones, vec![]))
}

// ---------- Confirmar ----------

fn confirmar_crear(cat: &Catalogo, e: &Estado) -> Turno {
    if e.modo != Modo::Crear {
        return turno(salida(cat, e.clone(), "No hay ninguna ficha nueva para crear. ¿Qué hacemos?".to_string(), menu(), vec![]));
    }
    let b = construir_borrador(cat, e);
    if !b.listo {
        let lista: Vec<String> = faltan(&b).iter().map(|(_, f)| f.to_string()).collect();
        let texto = format!("Todavía no puedo crearla. Para terminar faltaría: {}.", unir(&lista));
        let mut nuevo = e.clone();
        nuevo.pregunta = proxima_pregunta(&b);
        let opciones = opciones_de_ficha(cat, &nuevo, &b);
        return turno(salida(cat, nuevo, texto, opciones, vec![]));
    }
    con_accion(salida(cat, e.clone(), format!("Creando {}…", rs(&e.ficha.nombre)), vec![], vec![]), Accion::Crear)
}

fn confirmar_guardar(cat: &Catalogo, e: &Estado) -> Turno {
    if e.modo != Modo::Editar {
        return turno(salida(cat, e.clone(), "No hay ningún cambio para guardar. ¿Qué hacemos?".to_string(), menu(), vec![]));
    }
    let b = construir_borrador(cat, e);
    if !b.hay_cambios {
        return turno(salida(cat, e.clone(), "Todavía no hay cambios que guardar. ¿Qué quieres cambiar?".to_string(), campos(true), vec![]));
    }
    if !b.listo {
        let lista: Vec<String> = faltan(&b).iter().map(|(_, f)| f.to_string()).collect();
        let texto = format!("Para guardar necesito {}.", unir(&lista));
        return turno(salida(cat, e.clone(), texto, campos(true), vec![]));
    }
    con_accion(salida(cat, e.clone(), format!("Guardando {}…", rs(&e.ficha.nombre)), vec![], vec![]), Accion::Editar)
}

fn confirmar_eliminar(cat: &Catalogo, e: &Estado) -> Turno {
    if e.modo != Modo::Eliminar {
        return turno(salida(cat, e.clone(), "No hay ningún servicio por eliminar. ¿Qué hacemos?".to_string(), menu(), vec![]));
    }
    con_accion(salida(cat, e.clone(), format!("Eliminando {}…", rs(&e.ficha.nombre)), vec![], vec![]), Accion::Eliminar)
}

// Lo que dice Juno cuando ya se ejecutó la acción.
pub fn tras_ejecutar(cat: &Catalogo, e: &Estado, accion: &Accion, resultado: Resultado) -> Salida {
    let nombre = e.ficha.nombre.clone();
    match resultado {
        Resultado::Hecho(codigo) => {
            let (tipo, texto) = match accion {
                Accion::Crear => ("creado", format!("Listo, {} ya está creado y se ve en la página de servicios.", rs(&nombre))),
                Accion::Editar => ("editado", format!("Listo, guardé los cambios de {}.", rs(&nombre))),
                Accion::Eliminar => ("eliminado", format!("Listo, eliminé {}.", rs(&nombre))),
            };
            let imagen = if *accion == Accion::Eliminar { String::new() } else { e.ficha.imagen.clone() };
            let hecho = Hecho { tipo: tipo.to_string(), codigo, nombre, imagen };
            let mut s = salida(cat, Estado::default(), format!("{}\n\n¿Qué más hacemos?", texto), menu(), vec![]);
            s.hecho = Some(hecho);
            s
        }
        Resultado::Rechazo(motivo) => {
            let que = match accion {
                Accion::Crear => "crearlo",
                Accion::Editar => "guardar el cambio",
                Accion::Eliminar => "eliminarlo",
            };
            let texto = format!("No pude {}. {}", que, motivo);
            if *accion == Accion::Eliminar {
                salida(cat, Estado::default(), texto, menu(), vec![])
            } else {
                let b = construir_borrador(cat, e);
                let opciones = vec![op("Cambiar algo", "a", "cambiar"), op("Descartar", "r", "descartar")];
                let mut s = salida(cat, e.clone(), texto, opciones, vec![]);
                s.borrador = Some(b);
                s
            }
        }
    }
}

// ---------- Mensajes escritos ----------

const SI: [&str; 16] = [
    "si", "ok", "dale", "listo", "claro", "crealo", "creala", "crear", "guardalo", "guardar", "confirmo", "confirmar", "adelante",
    "publicalo", "correcto", "yes",
];
const NO: [&str; 8] = ["no", "nel", "nunca", "cancela", "cancelar", "descarta", "descartar", "olvidalo"];

fn solo_palabras_de(texto: &str, lista: &[&str]) -> bool {
    let pal = lector::palabras(texto);
    !pal.is_empty() && pal.iter().all(|p| lista.contains(&p.norm.as_str()))
}

fn quiere_cancelar(texto: &str) -> bool {
    let pal = lector::palabras(texto);
    let norm: Vec<&str> = pal.iter().map(|p| p.norm.as_str()).collect();
    norm.iter().any(|p| ["cancela", "cancelar", "descarta", "descartar", "olvidalo", "olvida"].contains(p))
        && !norm.iter().any(|p| lector::CREAR.contains(p))
}

fn por_texto(cat: &Catalogo, usuario: &str, estado: Estado, texto: &str) -> Turno {
    if texto.is_empty() {
        return turno(salida(cat, estado, "Escríbeme lo que necesitas o elige una opción.".to_string(), menu(), vec![]));
    }
    let norm = normalizar(texto);
    match estado.modo {
        Modo::Eliminar => {
            if solo_palabras_de(texto, &SI) {
                return confirmar_eliminar(cat, &estado);
            }
            if solo_palabras_de(texto, &NO) || quiere_cancelar(texto) || norm.starts_with("no ") {
                return descartar(cat, &estado);
            }
            let nombre = estado.ficha.nombre.clone();
            let texto = format!("¿Eliminamos {}? Pulsa **Sí, eliminar** o **No, dejarlo**.", rs(&nombre));
            turno(salida(cat, estado, texto, vec![op("Sí, eliminar", "rd", "eliminar_si"), op("No, dejarlo", "", "descartar")], vec![]))
        }
        Modo::Crear | Modo::Editar => {
            if quiere_cancelar(texto) || solo_palabras_de(texto, &NO) {
                return descartar(cat, &estado);
            }
            if solo_palabras_de(texto, &SI) {
                return if estado.modo == Modo::Crear { confirmar_crear(cat, &estado) } else { confirmar_guardar(cat, &estado) };
            }
            if solo_palabras_de(texto, &["cambiar", "algo", "mas", "otra", "cosa", "quiero", "modificar", "editar", "corregir"]) {
                return que_cambiar(cat, &estado);
            }
            ajustar_ficha(cat, estado, texto)
        }
        Modo::Ninguno => desde_ninguno(cat, usuario, estado, texto),
    }
}

// La persona dice algo mientras hay una ficha en marcha: se lee y se aplica.
fn ajustar_ficha(cat: &Catalogo, estado: Estado, texto: &str) -> Turno {
    let mut e = estado;
    let pal = lector::palabras(texto);
    let mut libre = vec![true; pal.len()];
    let eq = equipos_del_lector(cat);
    let pregunta = e.pregunta.clone();
    let mut lectura = lector::leer(&pal, &mut libre, &pregunta, &eq, true);

    // Respuestas cortas a lo que Juno preguntó: el nombre o la descripción tal cual.
    if pregunta == "nombre" && lectura.nombre.is_none() {
        let nombre = pal.iter().map(|p| p.original.clone()).collect::<Vec<_>>().join(" ");
        if nombre.chars().count() >= 3 {
            lectura.nombre = Some(lector::primera_mayuscula(&nombre));
        }
    }
    if pregunta == "descripcion" && lectura.descripcion.is_none() {
        let dictada = pal.iter().map(|p| p.cruda.clone()).collect::<Vec<_>>().join(" ");
        if dictada.chars().count() >= 10 {
            let mut dictada = lector::primera_mayuscula(&dictada);
            if dictada.chars().last().is_some_and(|c| c.is_alphanumeric()) {
                dictada.push('.');
            }
            lectura.descripcion = Some(dictada);
        }
    }

    if lectura.vacia() {
        let b = construir_borrador(cat, &e);
        let texto = if pregunta == "descripcion" {
            "La descripción debe tener al menos 10 caracteres. Escríbela de nuevo.".to_string()
        } else {
            "No terminé de entender eso. Dime el dato y su valor, por ejemplo: **el precio es 120**, **tarda 2 días** o **sirve para laptops**.".to_string()
        };
        let opciones = if pregunta == "descripcion" { vec![op("Descartar", "r", "descartar")] } else { opciones_de_ficha(cat, &e, &b) };
        return turno(salida(cat, e, texto, opciones, vec![]));
    }

    let primera = e.ficha == Ficha::default();
    let a = aplicar(cat, &mut e, lectura);
    let b = construir_borrador(cat, &e);
    e.pregunta = if e.modo == Modo::Crear { proxima_pregunta(&b) } else { String::new() };
    let texto = mensaje_de_ficha(cat, &e, &a, &b, primera);
    let opciones = opciones_de_ficha(cat, &e, &b);
    let foco = foco_de(&a.cambios);
    turno(salida(cat, e, texto, opciones, foco))
}

fn servicios_en_botones(lista: &[&ServicioCat], accion: &str, tono: &str) -> Vec<Opcion> {
    lista.iter().take(6).map(|s| op(&s.nombre, tono, &format!("{}:{}", accion, s.codigo))).collect()
}

fn desde_ninguno(cat: &Catalogo, usuario: &str, estado: Estado, texto: &str) -> Turno {
    let pal: Vec<Palabra> = lector::palabras(texto);
    let mut libre = vec![true; pal.len()];
    let eq = equipos_del_lector(cat);
    let norm = normalizar(texto);
    let pregunta = estado.pregunta.clone();
    let nombres: Vec<String> = cat.servicios.iter().map(|s| s.nombre.clone()).collect();

    // 1. Saludos.
    if lector::hay(&pal, &libre, &["hola", "buenas", "buenos", "hey", "ola"]) && pal.len() <= 3 {
        return bienvenida(cat, usuario);
    }

    // 2. Agregar un servicio nuevo.
    if lector::hay(&pal, &libre, &lector::CREAR) && pregunta != "servicio_editar" && pregunta != "servicio_eliminar" {
        let mut e = Estado { modo: Modo::Crear, ..Estado::default() };
        let nombre = lector::nombre_tras_verbo(&pal, &libre);
        if let Some((nombre, desde, hasta)) = &nombre {
            if let Some(otro) = repetido(cat, nombre, "") {
                let texto = format!("Ya existe {}. ¿Quieres editarlo?", rs(&otro.nombre));
                let opciones = vec![op("Editar", "a", &format!("editar:{}", otro.codigo)), op("Otro nombre", "", "")];
                return turno(salida(cat, Estado::default(), texto, opciones, vec![]));
            }
            for marca in libre.iter_mut().take(*hasta).skip(*desde) {
                *marca = false;
            }
        }
        let mut lectura = lector::leer(&pal, &mut libre, "", &eq, false);
        lectura.nombre = nombre.map(|(n, _, _)| n);
        lectura.equipos_suman = false;
        let primera = true;
        let a = aplicar(cat, &mut e, lectura);
        let b = construir_borrador(cat, &e);
        e.pregunta = proxima_pregunta(&b);
        let texto = if e.ficha.nombre.is_empty() {
            let mut t = "Claro. ¿Cómo se llamará el servicio?".to_string();
            if !a.cambios.is_empty() {
                t = format!("Listo, {}. ¿Cómo se llamará el servicio?", unir(&a.cambios.iter().map(|(c, y)| frase_de(cat, &e, c, *y)).collect::<Vec<_>>()));
            }
            e.pregunta = "nombre".to_string();
            t
        } else {
            mensaje_de_ficha(cat, &e, &a, &b, primera)
        };
        let opciones = opciones_de_ficha(cat, &e, &b);
        let foco = foco_de(&a.cambios);
        return turno(salida(cat, e, texto, opciones, foco));
    }

    // 3. ¿Habla de un servicio que ya existe?
    let busqueda = lector::buscar_servicio(&pal, &libre, &nombres);
    let quiere_quitar = lector::hay(&pal, &libre, &lector::QUITAR);
    match busqueda {
        Busqueda::Uno(k, palabras_usadas) => {
            for u in &palabras_usadas {
                libre[*u] = false;
            }
            let codigo = cat.servicios[k].codigo.clone();
            let lectura = lector::leer(&pal, &mut libre, "", &eq, false);
            let sigue_quitar = lector::hay(&pal, &libre, &lector::QUITAR);
            if (sigue_quitar || (pregunta == "servicio_eliminar")) && lectura.vacia() {
                return entrar(cat, &estado, &codigo, Modo::Eliminar, None);
            }
            let verbo_editar = lector::hay(&pal, &libre, &lector::EDITAR);
            if !lectura.vacia() || verbo_editar || pregunta == "servicio_editar" {
                return entrar(cat, &estado, &codigo, Modo::Editar, Some(lectura));
            }
            let s = &cat.servicios[k];
            let texto = format!("¿Qué quieres hacer con {}?", rs(&s.nombre));
            let opciones = vec![op("Editar", "a", &format!("editar:{}", s.codigo)), op("Eliminar", "r", &format!("eliminar:{}", s.codigo))];
            return turno(salida(cat, Estado::default(), texto, opciones, vec![]));
        }
        Busqueda::Varios(ks) => {
            let lista: Vec<&ServicioCat> = ks.iter().map(|k| &cat.servicios[*k]).collect();
            let eliminar = quiere_quitar || pregunta == "servicio_eliminar";
            let opciones = servicios_en_botones(&lista, if eliminar { "eliminar" } else { "editar" }, "");
            return turno(salida(cat, estado, "¿A cuál de estos servicios te refieres?".to_string(), opciones, vec![]));
        }
        Busqueda::Ninguno => {}
    }

    // 4. Servicios a cotizar.
    let pide_sin_precio = (lector::hay(&pal, &libre, &["sin"]) && lector::hay(&pal, &libre, &["precio"]))
        || lector::hay(&pal, &libre, &["cotizar", "cotizan", "cotiza"]);
    if pide_sin_precio && lector::hay(&pal, &libre, &["servicios", "cuales", "cuantos", "que"]) {
        let sin: Vec<&ServicioCat> = cat.servicios.iter().filter(|s| s.precio.is_none()).collect();
        if sin.is_empty() {
            return turno(salida(cat, Estado::default(), "Todos tus servicios tienen precio.".to_string(), menu(), vec![]));
        }
        let nombres_sin: Vec<String> = sin.iter().map(|s| rs(&s.nombre)).collect();
        let texto = format!(
            "Hay {} {} a cotizar: {}. ¿A cuál le pongo precio?",
            sin.len(),
            if sin.len() == 1 { "servicio" } else { "servicios" },
            unir(&nombres_sin)
        );
        let mut nuevo = Estado::default();
        nuevo.pregunta = "servicio_editar".to_string();
        let opciones = servicios_en_botones(&sin, "editar", "");
        return turno(salida(cat, nuevo, texto, opciones, vec![]));
    }

    // 5. Quiere quitar o cambiar algo, pero no dijo cuál servicio.
    let todos: Vec<&ServicioCat> = cat.servicios.iter().collect();
    if quiere_quitar || pregunta == "servicio_eliminar" {
        if cat.servicios.is_empty() {
            return turno(salida(cat, Estado::default(), "Todavía no hay servicios para eliminar.".to_string(), menu(), vec![]));
        }
        let mut nuevo = Estado::default();
        nuevo.pregunta = "servicio_eliminar".to_string();
        let texto = if pregunta == "servicio_eliminar" {
            "No encontré ese servicio. Elige uno de la lista o escribe su nombre.".to_string()
        } else {
            "¿Cuál servicio quieres quitar? Elige uno o escribe su nombre.".to_string()
        };
        return turno(salida(cat, nuevo, texto, servicios_en_botones(&todos, "eliminar", "r"), vec![]));
    }
    let habla_de_editar = lector::hay(&pal, &libre, &lector::EDITAR)
        || lector::hay(&pal, &libre, &["precio", "precios", "tiempo", "descripcion", "nombre", "editar"])
        || pregunta == "servicio_editar";
    if habla_de_editar {
        if cat.servicios.is_empty() {
            return turno(salida(cat, Estado::default(), "Todavía no hay servicios para editar. ¿Agregamos uno?".to_string(), menu(), vec![]));
        }
        let mut nuevo = Estado::default();
        nuevo.pregunta = "servicio_editar".to_string();
        let texto = if pregunta == "servicio_editar" {
            "No encontré ese servicio. Elige uno de la lista o escribe su nombre.".to_string()
        } else {
            "¿De cuál servicio? Elige uno o escribe su nombre.".to_string()
        };
        return turno(salida(cat, nuevo, texto, servicios_en_botones(&todos, "editar", "a"), vec![]));
    }

    // 6. No se entendió.
    let _ = norm;
    let texto = "Puedo agregar un servicio, cambiar sus datos o eliminarlo. Por ejemplo: **agrega limpieza de ventiladores** o **súbele a 100 el cambio de teclado**.".to_string();
    turno(salida(cat, Estado::default(), texto, menu(), vec![]))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn servicio_(codigo: &str, nombre: &str, precio: Option<&str>, tiempo: Option<&str>, equipos: &[&str], imagen: bool) -> ServicioCat {
        ServicioCat {
            codigo: codigo.to_string(),
            nombre: nombre.to_string(),
            descripcion: format!("Descripción de {} que ya existía.", nombre),
            precio: precio.map(|p| p.to_string()),
            tiempo: tiempo.map(|t| t.to_string()),
            equipos: equipos.iter().map(|e| e.to_string()).collect(),
            aviso_precio: false,
            imagen: if imagen { Some("/publico/servicios/X/imagen?v=1".to_string()) } else { None },
            protegido: codigo == "DIAGNOSTI",
        }
    }

    fn catalogo() -> Catalogo {
        Catalogo {
            equipos: vec![
                EquipoCat { codigo: "EQLAPTOP1".to_string(), nombre: "Laptop".to_string() },
                EquipoCat { codigo: "EQPCESCR2".to_string(), nombre: "PC de escritorio".to_string() },
                EquipoCat { codigo: "EQIMPRES3".to_string(), nombre: "Impresora".to_string() },
            ],
            servicios: vec![
                servicio_("PANTALLA1", "Cambio de pantalla", Some("180.00"), Some("2 a 3 días"), &["EQLAPTOP1"], true),
                servicio_("TECLADO01", "Cambio de teclado", Some("80.00"), Some("1 día"), &["EQLAPTOP1"], false),
                servicio_("WINDOWS01", "Formateo e instalación de Windows", Some("60.00"), Some("1 día"), &["EQLAPTOP1", "EQPCESCR2"], false),
                servicio_("MANTENIM1", "Mantenimiento preventivo", None, None, &["EQLAPTOP1", "EQPCESCR2", "EQIMPRES3"], false),
                servicio_("DIAGNOSTI", "Diagnóstico de equipo", None, None, &["EQLAPTOP1"], false),
            ],
        }
    }

    fn paso(cat: &Catalogo, estado: &Estado, texto: &str, accion: Option<&str>) -> Turno {
        let entrada = Entrada { texto: texto.to_string(), accion: accion.map(|a| a.to_string()), estado: estado.clone() };
        responder(cat, "Pedro Guevara", &entrada)
    }

    fn decir(cat: &Catalogo, estado: &Estado, texto: &str) -> Salida {
        paso(cat, estado, texto, None).salida
    }

    fn pulsar(cat: &Catalogo, estado: &Estado, accion: &str) -> Turno {
        paso(cat, estado, "", Some(accion))
    }

    fn estado_de(d: &Borrador, clave: &str) -> String {
        d.datos.iter().find(|x| x.clave == clave).map(|x| x.estado.clone()).unwrap_or_default()
    }

    #[test]
    fn saluda_con_el_primer_nombre_en_dos_burbujas() {
        let cat = catalogo();
        let s = pulsar(&cat, &Estado::default(), "inicio").salida;
        let burbujas: Vec<&str> = s.respuesta.split("\n\n").collect();
        assert_eq!(burbujas.len(), 2);
        assert_eq!(burbujas[0], "Hola, Pedro. Cuéntame qué servicio quieres agregar y yo armo la ficha");
        assert_eq!(burbujas[1], "También puedo editar sus datos y eliminar algún servicio");
        assert_eq!(s.opciones.len(), 4);
        assert!(s.borrador.is_none());
    }

    #[test]
    fn agregar_solo_con_el_nombre() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "agrega limpieza de ventiladores");
        let b = s.borrador.as_ref().unwrap();
        assert_eq!(b.modo, Modo::Crear);
        assert_eq!(b.nombre, "Limpieza de ventiladores");
        assert_eq!(estado_de(b, "nombre"), "ok");
        assert_eq!(estado_de(b, "descripcion"), "jn");
        assert_eq!((b.completos, b.total), (2, 5));
        assert_eq!(
            s.respuesta,
            "Listo, ya tengo el nombre y agregué una descripción. Para terminar faltaría: el precio, el tiempo estimado y para qué equipos sirve."
        );
        assert_eq!(s.estado.pregunta, "precio");
        assert!(s.foco.contains(&"nombre".to_string()));
        assert!(b.descripcion.contains("limpieza de ventiladores"));
    }

    #[test]
    fn ficha_completa_en_una_sola_frase() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "Agrega cambio de bisagras para laptops, 120 soles, 2 días");
        let b = s.borrador.as_ref().unwrap();
        assert!(b.listo);
        assert_eq!(b.precio.as_deref(), Some("120.00"));
        assert_eq!(b.tiempo.as_deref(), Some("2 días"));
        assert_eq!(b.equipos, vec!["Laptop".to_string()]);
        assert_eq!(s.respuesta, "Listo, armé la ficha de **Cambio de bisagras** con su descripción. ¿La creo?");
        let tonos: Vec<&str> = s.opciones.iter().map(|o| o.accion.as_str()).collect();
        assert_eq!(tonos, vec!["crear", "cambiar", "descartar"]);
    }

    #[test]
    fn completar_paso_a_paso_y_crear() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "agrega limpieza de ventiladores");
        let s = decir(&cat, &s.estado, "120");
        assert_eq!(s.borrador.as_ref().unwrap().precio.as_deref(), Some("120.00"));
        assert_eq!(s.estado.pregunta, "tiempo");
        assert!(s.respuesta.contains("anoté el precio en **S/ 120.00**"));
        let s = decir(&cat, &s.estado, "2 días");
        assert_eq!(s.estado.pregunta, "equipos");
        assert!(s.opciones.iter().any(|o| o.texto == "Todos los equipos"));
        let s = decir(&cat, &s.estado, "laptop y pc");
        assert!(s.borrador.as_ref().unwrap().listo);
        assert!(s.respuesta.contains("¿La creo?"));

        // Un «sí» escrito también confirma.
        let t = paso(&cat, &s.estado, "sí", None);
        assert_eq!(t.accion, Some(Accion::Crear));

        let t = pulsar(&cat, &s.estado, "crear");
        assert_eq!(t.accion, Some(Accion::Crear));
        let fin = tras_ejecutar(&cat, &s.estado, &Accion::Crear, Resultado::Hecho("NUEVO0001".to_string()));
        assert_eq!(fin.estado.modo, Modo::Ninguno);
        let hecho = fin.hecho.unwrap();
        assert_eq!((hecho.tipo.as_str(), hecho.codigo.as_str()), ("creado", "NUEVO0001"));
        assert!(fin.respuesta.contains("ya está creado"));
    }

    #[test]
    fn no_crea_si_falta_algo() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "agrega limpieza de ventiladores");
        let t = pulsar(&cat, &s.estado, "crear");
        assert!(t.accion.is_none());
        assert!(t.salida.respuesta.starts_with("Todavía no puedo crearla"));
        let t = paso(&cat, &s.estado, "ok", None);
        assert!(t.accion.is_none());
    }

    #[test]
    fn quitar_un_dato_retrocede() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "Agrega cambio de bisagras para laptops, 120 soles, 2 días");
        let s = decir(&cat, &s.estado, "quita el tiempo");
        let b = s.borrador.as_ref().unwrap();
        assert!(!b.listo);
        assert_eq!(estado_de(b, "tiempo"), "falta");
        assert_eq!(b.completos, 4);
        assert!(s.respuesta.starts_with("Listo, quité el tiempo."));
        assert!(s.respuesta.contains("faltaría: el tiempo estimado"));
        assert_eq!(s.foco, vec!["tiempo".to_string()]);
    }

    #[test]
    fn sin_tiempo_fijo_cuenta_como_dicho() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "agrega limpieza de ventiladores a 50 soles para laptops");
        assert_eq!(s.estado.pregunta, "tiempo");
        let s = decir(&cat, &s.estado, "sin tiempo fijo");
        let b = s.borrador.as_ref().unwrap();
        assert!(b.listo);
        assert_eq!(b.datos[3].valor, "Sin tiempo fijo");
    }

    #[test]
    fn agregar_pregunta_el_nombre() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "Agregar un servicio");
        assert_eq!(s.estado.modo, Modo::Crear);
        assert_eq!(s.estado.pregunta, "nombre");
        assert_eq!(s.borrador.as_ref().unwrap().completos, 0);
        let s = decir(&cat, &s.estado, "Reparación de bisagras");
        assert_eq!(s.borrador.as_ref().unwrap().nombre, "Reparación de bisagras");
        assert_eq!(s.estado.pregunta, "precio");
    }

    #[test]
    fn nombre_repetido_no_se_acepta() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "agrega cambio de teclado");
        assert_eq!(s.estado.modo, Modo::Ninguno);
        assert!(s.respuesta.contains("Ya existe **Cambio de teclado**"));
        assert_eq!(s.opciones[0].accion, "editar:TECLADO01");

        let s = decir(&cat, &Estado::default(), "agrega limpieza de ventiladores");
        let s = decir(&cat, &s.estado, "cámbiale el nombre a Cambio de pantalla");
        assert_eq!(s.borrador.as_ref().unwrap().nombre, "Limpieza de ventiladores");
        assert!(s.respuesta.contains("Ya existe un servicio llamado **Cambio de pantalla**"));
    }

    #[test]
    fn cambiar_el_nombre_vuelve_a_redactar_la_descripcion() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "agrega limpieza de ventiladores");
        let antes = s.borrador.as_ref().unwrap().descripcion.clone();
        let s = decir(&cat, &s.estado, "cámbiale el nombre a Limpieza de fuentes");
        let b = s.borrador.as_ref().unwrap();
        assert_eq!(b.nombre, "Limpieza de fuentes");
        assert_ne!(b.descripcion, antes);
        assert!(s.respuesta.contains("cambié el nombre a **Limpieza de fuentes**"));
        assert!(s.respuesta.contains("ajusté la descripción"));
    }

    #[test]
    fn descripcion_dictada_se_respeta() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "agrega limpieza de ventiladores");
        let s = decir(&cat, &s.estado, "la descripción es: Limpiamos y lubricamos los ventiladores de tu equipo");
        let b = s.borrador.as_ref().unwrap();
        assert_eq!(b.descripcion, "Limpiamos y lubricamos los ventiladores de tu equipo.");
        assert_eq!(estado_de(b, "descripcion"), "ok");
        // Si luego cambia el nombre, la descripción dictada no se toca.
        let s = decir(&cat, &s.estado, "que se llame Limpieza de fuentes");
        assert_eq!(s.borrador.as_ref().unwrap().descripcion, "Limpiamos y lubricamos los ventiladores de tu equipo.");
    }

    #[test]
    fn editar_el_precio_de_un_servicio() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "súbele a 100 el cambio de teclado");
        let b = s.borrador.as_ref().unwrap();
        assert_eq!(b.modo, Modo::Editar);
        assert_eq!(s.estado.codigo, "TECLADO01");
        assert_eq!(estado_de(b, "precio"), "cambio");
        let precio = b.datos.iter().find(|d| d.clave == "precio").unwrap();
        assert_eq!((precio.valor.as_str(), precio.antes.as_str()), ("S/ 100.00", "S/ 80.00"));
        assert!(b.hay_cambios && b.listo);
        assert_eq!(s.respuesta, "Listo, en **Cambio de teclado** cambié el precio a **S/ 100.00**. ¿Guardo el cambio?");
        let acciones: Vec<&str> = s.opciones.iter().map(|o| o.accion.as_str()).collect();
        assert_eq!(acciones, vec!["guardar", "cambiar", "descartar"]);

        let t = pulsar(&cat, &s.estado, "guardar");
        assert_eq!(t.accion, Some(Accion::Editar));
        let fin = tras_ejecutar(&cat, &s.estado, &Accion::Editar, Resultado::Hecho("TECLADO01".to_string()));
        assert_eq!(fin.hecho.unwrap().tipo, "editado");
    }

    #[test]
    fn editar_y_quitar_el_tiempo() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "quita el tiempo del cambio de teclado");
        let b = s.borrador.as_ref().unwrap();
        assert_eq!(b.modo, Modo::Editar);
        assert_eq!(estado_de(b, "tiempo"), "quitado");
        assert!(b.hay_cambios);
    }

    #[test]
    fn editar_desde_el_boton_y_sin_cambios() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "Editar el servicio: Cambio de pantalla");
        assert_eq!(s.estado.modo, Modo::Editar);
        assert_eq!(s.estado.codigo, "PANTALLA1");
        assert_eq!(s.respuesta, "Estás editando **Cambio de pantalla**. ¿Qué quieres cambiar?");
        assert!(!s.borrador.as_ref().unwrap().hay_cambios);
        // Guardar sin cambios no hace nada.
        let t = pulsar(&cat, &s.estado, "guardar");
        assert!(t.accion.is_none());
        // Pedir un dato y responderlo.
        let s = pulsar(&cat, &s.estado, "pedir:precio").salida;
        assert_eq!(s.estado.pregunta, "precio");
        let s = decir(&cat, &s.estado, "200");
        assert_eq!(estado_de(s.borrador.as_ref().unwrap(), "precio"), "cambio");
        // Poner el mismo valor que tenía no es un cambio.
        let s2 = decir(&cat, &Estado::default(), "Editar el servicio: Cambio de pantalla");
        let s2 = decir(&cat, &s2.estado, "el precio es 180");
        assert!(!s2.borrador.as_ref().unwrap().hay_cambios);
        assert!(s2.respuesta.contains("Eso ya estaba así"));
    }

    #[test]
    fn editar_los_equipos() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "Editar el servicio: Cambio de teclado");
        let s = decir(&cat, &s.estado, "también para pc");
        let b = s.borrador.as_ref().unwrap();
        assert_eq!(b.equipos, vec!["Laptop".to_string(), "PC de escritorio".to_string()]);
        assert_eq!(estado_de(b, "equipos"), "cambio");
        let s = decir(&cat, &s.estado, "quítale laptop");
        assert_eq!(s.borrador.as_ref().unwrap().equipos, vec!["PC de escritorio".to_string()]);
    }

    #[test]
    fn eliminar_con_confirmacion() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "quita el servicio de cambio de teclado");
        assert_eq!(s.estado.modo, Modo::Eliminar);
        assert_eq!(s.estado.codigo, "TECLADO01");
        assert!(s.respuesta.contains("¿Seguro que quieres eliminar **Cambio de teclado**?"));
        assert_eq!(s.opciones[0].texto, "Sí, eliminar");
        assert_eq!(s.opciones[0].tono, "rd");
        assert_eq!(s.opciones[1].texto, "No, dejarlo");

        let t = pulsar(&cat, &s.estado, "eliminar_si");
        assert_eq!(t.accion, Some(Accion::Eliminar));
        let fin = tras_ejecutar(&cat, &s.estado, &Accion::Eliminar, Resultado::Hecho("TECLADO01".to_string()));
        assert_eq!(fin.hecho.as_ref().unwrap().tipo, "eliminado");
        assert_eq!(fin.estado.modo, Modo::Ninguno);

        // «No» o «descartar» lo deja todo como estaba.
        let n = decir(&cat, &s.estado, "no");
        assert_eq!(n.estado.modo, Modo::Ninguno);
        assert!(n.respuesta.contains("no eliminé nada"));
        // Un «sí» escrito también confirma.
        assert_eq!(paso(&cat, &s.estado, "sí", None).accion, Some(Accion::Eliminar));
        // Cualquier otra cosa vuelve a preguntar.
        let otra = decir(&cat, &s.estado, "mmm");
        assert_eq!(otra.estado.modo, Modo::Eliminar);
    }

    #[test]
    fn eliminar_desde_el_boton_de_la_fila() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "Eliminar el servicio: Mantenimiento preventivo");
        assert_eq!(s.estado.codigo, "MANTENIM1");
        assert_eq!(s.estado.modo, Modo::Eliminar);
        let s = pulsar(&cat, &Estado::default(), "eliminar:WINDOWS01").salida;
        assert_eq!(s.estado.codigo, "WINDOWS01");
    }

    #[test]
    fn el_diagnostico_no_se_elimina() {
        let cat = catalogo();
        let s = pulsar(&cat, &Estado::default(), "eliminar:DIAGNOSTI").salida;
        assert_eq!(s.estado.modo, Modo::Ninguno);
        assert!(s.respuesta.contains("no se puede eliminar"));
        let s = decir(&cat, &Estado::default(), "elimina el diagnóstico de equipo");
        assert_eq!(s.estado.modo, Modo::Ninguno);
        assert!(s.respuesta.contains("no se puede eliminar"));
    }

    #[test]
    fn quitar_sin_decir_cual() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "Quitar un servicio");
        assert_eq!(s.estado.pregunta, "servicio_eliminar");
        assert!(s.opciones.iter().any(|o| o.accion == "eliminar:TECLADO01"));
        let s = decir(&cat, &s.estado, "el de windows");
        assert_eq!(s.estado.modo, Modo::Eliminar);
        assert_eq!(s.estado.codigo, "WINDOWS01");
    }

    #[test]
    fn cambiar_un_precio_sin_decir_cual() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "Cambiar un precio");
        assert_eq!(s.estado.pregunta, "servicio_editar");
        let s = decir(&cat, &s.estado, "el de pantalla");
        assert_eq!(s.estado.modo, Modo::Editar);
        assert_eq!(s.estado.codigo, "PANTALLA1");
    }

    #[test]
    fn servicios_sin_precio() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "Servicios sin precio");
        assert!(s.respuesta.contains("**Mantenimiento preventivo**"));
        assert!(s.respuesta.starts_with("Hay 2 servicios a cotizar"));
        assert!(s.opciones.iter().any(|o| o.accion == "editar:MANTENIM1"));
    }

    #[test]
    fn descartar_y_cancelar() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "agrega limpieza de ventiladores");
        let n = pulsar(&cat, &s.estado, "descartar").salida;
        assert_eq!(n.estado.modo, Modo::Ninguno);
        assert!(n.borrador.is_none());
        let n = decir(&cat, &s.estado, "cancela");
        assert_eq!(n.estado.modo, Modo::Ninguno);
    }

    #[test]
    fn no_cambia_de_tarea_con_una_ficha_a_medias() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "agrega limpieza de ventiladores");
        let t = pulsar(&cat, &s.estado, "editar:TECLADO01").salida;
        assert_eq!(t.estado.modo, Modo::Crear);
        assert!(t.respuesta.starts_with("Primero terminemos la ficha de **Limpieza de ventiladores**"));
    }

    #[test]
    fn aviso_de_precio() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "agrega limpieza de ventiladores");
        let s = pulsar(&cat, &s.estado, "aviso:1").salida;
        assert!(s.borrador.as_ref().unwrap().aviso_precio);
        let s = decir(&cat, &s.estado, "quita el aviso");
        assert!(!s.borrador.as_ref().unwrap().aviso_precio);
        // Al editar, el aviso también cuenta como cambio.
        let e = decir(&cat, &Estado::default(), "Editar el servicio: Cambio de teclado");
        let e = pulsar(&cat, &e.estado, "aviso:1").salida;
        assert!(e.borrador.as_ref().unwrap().hay_cambios);
    }

    #[test]
    fn imagen_al_crear_y_al_editar() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "agrega limpieza de ventiladores");
        let s = pulsar(&cat, &s.estado, "imagen_nueva").salida;
        assert_eq!(s.borrador.as_ref().unwrap().imagen, "nueva");
        let s = pulsar(&cat, &s.estado, "imagen_quitar").salida;
        assert_eq!(s.borrador.as_ref().unwrap().imagen, "");

        // Un servicio que ya tenía imagen: quitarla es un cambio, y elegir otra la reemplaza.
        let e = decir(&cat, &Estado::default(), "Editar el servicio: Cambio de pantalla");
        assert_eq!(e.borrador.as_ref().unwrap().imagen, "actual");
        let q = decir(&cat, &e.estado, "quita la imagen");
        assert_eq!(q.borrador.as_ref().unwrap().imagen, "quitar");
        assert!(q.borrador.as_ref().unwrap().hay_cambios);
        let n = pulsar(&cat, &e.estado, "imagen_nueva").salida;
        assert_eq!(n.borrador.as_ref().unwrap().imagen, "nueva");
        // Descartar la nueva devuelve la que tenía.
        let v = pulsar(&cat, &n.estado, "imagen_quitar").salida;
        assert_eq!(v.borrador.as_ref().unwrap().imagen, "actual");
        assert!(!v.borrador.as_ref().unwrap().hay_cambios);
        // Si pide una imagen por el chat, Juno indica el botón.
        let p = decir(&cat, &s.estado, "quiero agregar una imagen");
        assert!(p.respuesta.contains("Agregar imagen"));
    }

    #[test]
    fn si_el_servidor_rechaza_la_ficha_se_conserva() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "Agrega cambio de bisagras para laptops, 120 soles, 2 días");
        let fin = tras_ejecutar(&cat, &s.estado, &Accion::Crear, Resultado::Rechazo("Ya existe un servicio con ese nombre.".to_string()));
        assert_eq!(fin.estado.modo, Modo::Crear);
        assert!(fin.hecho.is_none());
        assert!(fin.respuesta.starts_with("No pude crearlo. Ya existe"));
        assert!(fin.borrador.is_some());
    }

    #[test]
    fn mensaje_que_no_se_entiende() {
        let cat = catalogo();
        let s = decir(&cat, &Estado::default(), "qué clima hará mañana");
        assert!(s.respuesta.starts_with("Puedo agregar un servicio"));
        let c = decir(&cat, &Estado::default(), "agrega limpieza de ventiladores");
        let c = decir(&cat, &c.estado, "no sé qué poner");
        assert!(c.respuesta.starts_with("No terminé de entender"));
        assert_eq!(c.estado.modo, Modo::Crear);
    }

    #[test]
    fn el_estado_viejo_o_vacio_no_rompe() {
        let cat = catalogo();
        // Editar un servicio que ya no existe.
        let s = pulsar(&cat, &Estado::default(), "editar:NOEXISTE1").salida;
        assert!(s.respuesta.contains("No encontré ese servicio"));
        // Confirmar sin ficha.
        let t = pulsar(&cat, &Estado::default(), "crear");
        assert!(t.accion.is_none());
        let t = pulsar(&cat, &Estado::default(), "eliminar_si");
        assert!(t.accion.is_none());
    }

    #[test]
    fn las_descripciones_de_plantilla_son_validas() {
        for nombre in ["Cambio de bisagras", "Limpieza de ventiladores", "Reparación de placa", "Instalación de Windows", "Mantenimiento preventivo", "Formateo e instalación de Windows", "Diagnóstico", "Recarga de tinta", "Cambio", "Limpieza"] {
            let d = redactar_descripcion(nombre);
            let largo = d.chars().count();
            assert!((10..=500).contains(&largo), "{} -> {}", nombre, d);
            assert!(d.ends_with('.'), "{}", d);
        }
        assert_eq!(redactar_descripcion("Cambio de bisagras"), "Hacemos el cambio de bisagras en tu equipo y comprobamos que todo funcione bien antes de entregártelo.");
    }
}
