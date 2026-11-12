// panel_interprete.rs: el «oído» de Juno en el panel de servicios (Administrador). Lee lo que escribe y saca de ahí los datos de la ficha:
// el nombre, la descripción, el precio, el tiempo, los equipos y el aviso de precio. También reconoce de qué servicio habla.
//
// Regla de este archivo (igual que motor.rs): NO toca la base de datos ni la red. Solo recibe texto y devuelve lo que entendió, por
// eso se prueba solo con «cargo test». Lo que Juno hace con lo entendido (preguntar, confirmar, guardar) vive en panel_servicios.rs.
//
// Cómo funciona: el mensaje se parte en PALABRAS (cada una con su versión original y su versión normalizada, sin tildes ni mayúsculas).
// Las palabras que ya se usaron para algo (por ejemplo las del nombre) se marcan como «no libres» para que no se lean dos veces.

use super::motor::normalizar;
use crate::modulos::servicio_tecnico::validaciones::normalizar_precio;

// Un tipo de equipo que la empresa atiende (lo que Juno conoce del catálogo).
#[derive(Clone, Debug)]
pub struct Equipo {
    pub codigo: String,
    pub nombre: String,
}

// Una palabra del mensaje.
#[derive(Clone, Debug)]
pub struct Palabra {
    pub original: String, // como la escribió, sin signos en los bordes: «Bisagras»
    pub norm: String,     // para comparar: «bisagras»
    pub cruda: String,    // como la escribió, con sus signos: «bisagras,» (sirve para copiar una descripción tal cual)
    pub corte: bool,      // true si después venía una coma, punto, punto y coma o dos puntos (termina una frase)
}

const ADORNOS: &str = ",.;:¿?¡!()\"'«»“”*";

// Parte el mensaje en palabras. «S/120» se separa en «S/» y «120»; «2-3» en «2», «a» y «3».
pub fn palabras(texto: &str) -> Vec<Palabra> {
    let mut lista = Vec::new();
    for crudo in texto.split_whitespace() {
        let corte = crudo.ends_with(',') || crudo.ends_with(';') || crudo.ends_with(':') || crudo.ends_with('.');
        let limpio = crudo.trim_matches(|c: char| ADORNOS.contains(c));
        if limpio.is_empty() {
            continue;
        }
        let norm = normalizar(limpio);

        if norm.starts_with("s/") && norm.chars().count() > 2 {
            let resto: String = limpio.chars().skip(2).collect();
            lista.push(Palabra { original: "S/".to_string(), norm: "s/".to_string(), cruda: "S/".to_string(), corte: false });
            lista.push(Palabra { norm: normalizar(&resto), original: resto, cruda: crudo.to_string(), corte });
            continue;
        }
        if let Some((izquierda, derecha)) = limpio.split_once('-') {
            let numero = |t: &str| !t.is_empty() && t.chars().all(|c| c.is_ascii_digit());
            if numero(izquierda) && numero(derecha) {
                lista.push(Palabra { original: izquierda.to_string(), norm: izquierda.to_string(), cruda: izquierda.to_string(), corte: false });
                lista.push(Palabra { original: "a".to_string(), norm: "a".to_string(), cruda: "a".to_string(), corte: false });
                lista.push(Palabra { original: derecha.to_string(), norm: derecha.to_string(), cruda: crudo.to_string(), corte });
                continue;
            }
        }
        lista.push(Palabra { original: limpio.to_string(), norm, cruda: crudo.to_string(), corte });
    }
    lista
}

// ---------- Utilidades ----------

pub fn es_numero(t: &str) -> bool {
    let separadores = t.chars().filter(|c| *c == '.' || *c == ',').count();
    t.chars().next().is_some_and(|c| c.is_ascii_digit())
        && t.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',')
        && separadores <= 1
}

fn es_entero(t: &str) -> bool {
    !t.is_empty() && t.len() <= 3 && t.chars().all(|c| c.is_ascii_digit())
}

pub fn primera_mayuscula(texto: &str) -> String {
    let mut letras = texto.chars();
    match letras.next() {
        Some(primera) => primera.to_uppercase().collect::<String>() + letras.as_str(),
        None => String::new(),
    }
}

// ¿Alguna palabra libre del mensaje está en la lista?
pub fn hay(pal: &[Palabra], libre: &[bool], lista: &[&str]) -> bool {
    pal.iter().enumerate().any(|(i, p)| libre[i] && lista.contains(&p.norm.as_str()))
}

// Cuántas letras al comienzo son iguales.
fn comun(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

// ¿Son la misma palabra, aunque cambie el final? («formatea» y «formateo», «pantalla» y «pantallas»)
fn parecidas(a: &str, b: &str) -> bool {
    let (la, lb) = (a.chars().count(), b.chars().count());
    a == b || (la >= 5 && lb >= 5 && comun(a, b) * 10 >= la.min(lb) * 7)
}

fn singular(palabra: &str) -> String {
    if palabra.chars().count() >= 3 && palabra.ends_with('s') && !palabra.ends_with("ss") {
        palabra[..palabra.len() - 1].to_string()
    } else {
        palabra.to_string()
    }
}

fn palabras_de(texto: &str) -> Vec<String> {
    normalizar(texto)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect()
}

// ---------- Lo que se entendió ----------

// Todo lo que el mensaje decía sobre la ficha. «None» = no dijo nada de eso.
#[derive(Default, Debug)]
pub struct Lectura {
    pub nombre: Option<String>,
    pub descripcion: Option<String>,
    pub precio: Option<Option<String>>, // Some(None) = «a cotizar»
    pub tiempo: Option<Option<String>>, // Some(None) = «sin tiempo fijo»
    pub equipos: Option<Vec<String>>,   // códigos de los tipos de equipo
    pub equipos_suman: bool,            // true = se agregan a los que ya había (en vez de reemplazarlos)
    pub equipos_quitar: Vec<String>,    // códigos que se quitan
    pub aviso: Option<bool>,            // «El precio puede cambiar»
    pub quitar: Vec<&'static str>,      // campos que se vacían: «tiempo», «precio»
    pub quitar_imagen: bool,
    pub pedir_imagen: bool, // quiere poner una imagen (Juno le indica el botón: él no puede recibir archivos por el chat)
}

impl Lectura {
    // ¿No se entendió nada?
    pub fn vacia(&self) -> bool {
        self.nombre.is_none()
            && self.descripcion.is_none()
            && self.precio.is_none()
            && self.tiempo.is_none()
            && self.equipos.is_none()
            && self.equipos_quitar.is_empty()
            && self.aviso.is_none()
            && self.quitar.is_empty()
            && !self.quitar_imagen
            && !self.pedir_imagen
    }
}

pub const QUITAR: [&str; 12] = [
    "quita", "quitale", "quitar", "borra", "borrale", "borrar", "elimina", "eliminar", "eliminale", "saca", "sacale", "olvida",
];
// Verbos con los que se pide agregar un servicio nuevo, o cambiar uno que ya existe.
pub const CREAR: [&str; 13] = [
    "agrega", "agregar", "agregame", "agregue", "crea", "crear", "creame", "anade", "anadir", "registra", "registrar", "nuevo", "nueva",
];
pub const EDITAR: [&str; 14] = [
    "edita", "editar", "editale", "cambia", "cambiale", "cambiar", "modifica", "modificar", "actualiza", "actualizar", "corrige",
    "corregir", "ponle", "pon",
];
const SUMAR: [&str; 8] = ["tambien", "ademas", "agrega", "agregale", "anade", "anadele", "suma", "incluye"];
const AVISO: [&str; 5] = ["aviso", "avisa", "avisar", "avisale", "advertencia"];
const PRECIO_CLAVE: [&str; 20] = [
    "precio", "cuesta", "cuestan", "vale", "valen", "cobra", "cobrar", "cobramos", "costo", "sale", "subele", "sube", "subir", "baja",
    "bajale", "bajar", "soles", "sol", "s/", "base",
];

// ---------- Equipos ----------

fn alias(palabra: &str) -> Option<&'static str> {
    match palabra {
        "computadora" | "computador" | "desktop" | "cpu" | "torre" | "ordenador" => Some("pc"),
        "notebook" | "portatil" | "netbook" => Some("laptop"),
        _ => None,
    }
}

fn coincide(a: &str, b: &str) -> bool {
    a == b || (a.chars().count() >= 4 && b.chars().count() >= 4 && (a.starts_with(b) || b.starts_with(a)))
}

// Las palabras que identifican a un tipo de equipo: «PC de escritorio» -> «pc», «escritorio».
fn claves_de(nombre: &str) -> Vec<String> {
    const SIN_VALOR: [&str; 8] = ["de", "del", "la", "el", "los", "las", "y", "para"];
    palabras_de(nombre)
        .into_iter()
        .filter(|p| !SIN_VALOR.contains(&p.as_str()))
        .map(|p| singular(&p))
        .collect()
}

// Los tipos de equipo nombrados entre las palabras [desde, hasta) que sigan libres. Devuelve sus códigos (en el orden del catálogo) y
// la posición de las palabras que los nombraron.
fn equipos_en(pal: &[Palabra], libre: &[bool], desde: usize, hasta: usize, equipos: &[Equipo]) -> (Vec<String>, Vec<usize>) {
    let claves: Vec<Vec<String>> = equipos.iter().map(|e| claves_de(&e.nombre)).collect();
    let mut encontrados: Vec<String> = Vec::new();
    let mut usadas: Vec<usize> = Vec::new();
    for i in desde..hasta.min(pal.len()) {
        if !libre[i] {
            continue;
        }
        let texto = singular(&pal[i].norm);
        let alterna = alias(&texto).map(|a| a.to_string());
        for (k, equipo) in equipos.iter().enumerate() {
            let coincidencia = claves[k]
                .iter()
                .any(|clave| coincide(&texto, clave) || alterna.as_deref().is_some_and(|a| coincide(a, clave)));
            if coincidencia {
                if !encontrados.contains(&equipo.codigo) {
                    encontrados.push(equipo.codigo.clone());
                }
                if !usadas.contains(&i) {
                    usadas.push(i);
                }
            }
        }
    }
    // En el orden del catálogo, no en el orden en que los dijo.
    let orden: Vec<String> = equipos.iter().map(|e| e.codigo.clone()).filter(|c| encontrados.contains(c)).collect();
    (orden, usadas)
}

// ---------- Nombre ----------

const SALTAR: [&str; 50] = [
    "quiero", "quisiera", "necesito", "me", "puedes", "puedo", "podrias", "podria", "por", "favor", "un", "una", "el", "la", "los", "las",
    "nuevo", "nueva", "servicio", "servicios", "tecnico", "tecnicos", "llamado", "llamada", "de", "del", "que", "se", "llame", "llama",
    "con", "nombre", "agrega", "agregar", "agregame", "agregue", "crea", "crear", "creame", "anade", "anadir", "registra", "registrar",
    "ponle", "pon", "y", "tambien", "ahora", "otro", "otra",
];
const PARO: [&str; 22] = [
    "para", "cuesta", "cuestan", "vale", "valen", "precio", "cobra", "cobramos", "tiempo", "demora", "dura", "tarda", "sirve", "sirven",
    "sirva", "soles", "sol", "s/", "que", "mismo", "sin", "cotizar",
];

// El nombre de un servicio que empieza en la palabra «inicio» (saltando los adornos como «un servicio de»). Se detiene en el precio,
// el tiempo, «para …», un número o una coma. Devuelve el nombre y la posición [desde, hasta) de sus palabras.
pub fn nombre_desde(pal: &[Palabra], libre: &[bool], inicio: usize) -> Option<(String, usize, usize)> {
    let n = pal.len();
    let mut i = inicio;
    while i < n && (!libre[i] || SALTAR.contains(&pal[i].norm.as_str())) {
        i += 1;
    }
    let desde = i;
    let mut fin = i;
    let mut j = i;
    while j < n {
        if !libre[j] {
            break;
        }
        let w = pal[j].norm.as_str();
        if es_numero(w) || PARO.contains(&w) {
            break;
        }
        let preposicion = w == "a" || w == "en" || w == "de" || w == "por";
        if preposicion && j + 1 < n && (es_numero(&pal[j + 1].norm) || pal[j + 1].norm == "s/") {
            break;
        }
        fin = j + 1;
        if pal[j].corte {
            break;
        }
        j += 1;
    }
    while fin > desde && ["a", "en", "de", "por", "con", "y", "e"].contains(&pal[fin - 1].norm.as_str()) {
        fin -= 1;
    }
    if fin <= desde {
        return None;
    }
    let nombre = pal[desde..fin].iter().map(|p| p.original.clone()).collect::<Vec<_>>().join(" ");
    if nombre.chars().count() < 3 {
        return None;
    }
    Some((primera_mayuscula(&nombre), desde, fin))
}

// El nombre de un servicio nuevo en «agrega cambio de bisagras para laptops…»: lo que sigue al verbo.
pub fn nombre_tras_verbo(pal: &[Palabra], libre: &[bool]) -> Option<(String, usize, usize)> {
    const VERBOS: [&str; 11] =
        ["agrega", "agregar", "agregame", "agregue", "crea", "crear", "creame", "anade", "anadir", "registra", "registrar"];
    let verbo = (0..pal.len()).find(|&i| libre[i] && VERBOS.contains(&pal[i].norm.as_str()));
    nombre_desde(pal, libre, verbo.map(|v| v + 1).unwrap_or(0))
}

// El nombre dicho de forma explícita: «el nombre es X», «que se llame X», «cámbiale el nombre a X».
fn nombre_explicito(pal: &[Palabra], libre: &[bool]) -> Option<(String, usize, usize)> {
    const RELLENO: [&str; 12] = ["a", "en", "es", "sera", "sea", "como", "de", "del", "que", "se", "llame", "el"];
    let marca = (0..pal.len()).find(|&i| {
        libre[i] && (pal[i].norm.starts_with("nombre") || ["llama", "llamado", "llamarse", "renombra", "renombrar", "renombralo"].contains(&pal[i].norm.as_str()))
    })?;
    let mut inicio = marca + 1;
    while inicio < pal.len() && (RELLENO.contains(&pal[inicio].norm.as_str()) || !libre[inicio]) {
        inicio += 1;
    }
    nombre_desde(pal, libre, inicio).map(|(nombre, _, fin)| (nombre, marca, fin))
}

// ---------- Lectura principal ----------

fn consumir(libre: &mut [bool], desde: usize, hasta: usize) {
    for marca in libre.iter_mut().take(hasta).skip(desde) {
        *marca = false;
    }
}

fn unidad(palabra: &str) -> Option<(&'static str, &'static str)> {
    match palabra {
        "dia" | "dias" => Some(("día", "días")),
        "hora" | "horas" => Some(("hora", "horas")),
        "semana" | "semanas" => Some(("semana", "semanas")),
        "minuto" | "minutos" => Some(("minuto", "minutos")),
        "mes" | "meses" => Some(("mes", "meses")),
        _ => None,
    }
}

fn cantidad(texto: &str, singular_: &str, plural: &str) -> String {
    let unidad = if texto == "1" { singular_ } else { plural };
    format!("{} {}", texto, unidad)
}

// Lee el mensaje y devuelve lo que decía sobre la ficha. «libre» marca las palabras que todavía se pueden usar (las ya usadas, por
// ejemplo en el nombre, se ignoran). «pregunta» es lo que Juno estaba preguntando («precio», «tiempo», «equipos», «nombre»): permite
// entender respuestas cortas como «120» o «2 días».
pub fn leer(pal: &[Palabra], libre: &mut [bool], pregunta: &str, equipos: &[Equipo], con_nombre: bool) -> Lectura {
    let n = pal.len();
    let mut lectura = Lectura::default();

    // 1. «quita el tiempo», «borra la imagen», «quítale laptop»: se revisa primero porque nombra los mismos campos.
    for i in 0..n {
        if !(libre[i] && QUITAR.contains(&pal[i].norm.as_str())) {
            continue;
        }
        let limite = (i + 5).min(n);
        let mut encontro = false;
        for j in (i + 1)..limite {
            if !libre[j] {
                continue;
            }
            match pal[j].norm.as_str() {
                "tiempo" | "plazo" | "demora" => {
                    lectura.quitar.push("tiempo");
                    libre[j] = false;
                    encontro = true;
                }
                "precio" | "costo" => {
                    lectura.quitar.push("precio");
                    libre[j] = false;
                    encontro = true;
                }
                "imagen" | "imagenes" | "foto" | "fotografia" => {
                    lectura.quitar_imagen = true;
                    libre[j] = false;
                    encontro = true;
                }
                "aviso" | "advertencia" => {
                    lectura.aviso = Some(false);
                    libre[j] = false;
                    encontro = true;
                }
                _ => {}
            }
        }
        let (codigos, usadas) = equipos_en(pal, libre, i + 1, limite, equipos);
        if !codigos.is_empty() {
            lectura.equipos_quitar.extend(codigos);
            for u in usadas {
                libre[u] = false;
            }
            encontro = true;
        }
        if encontro {
            libre[i] = false;
        }
    }

    // 2. La descripción: lo que viene después de la palabra «descripción» es el texto, tal como lo escribió.
    if let Some(i) = (0..n).find(|&i| libre[i] && pal[i].norm.starts_with("descrip")) {
        let mut j = i + 1;
        while j < n && ["es", "sera", "seria", "diga", "debe", "decir", "como", "que", "sea", "para"].contains(&pal[j].norm.as_str()) {
            j += 1;
        }
        let texto = pal[j.min(n)..].iter().map(|p| p.cruda.clone()).collect::<Vec<_>>().join(" ");
        let texto = texto.trim().trim_start_matches(':').trim().to_string();
        consumir(libre, i, n);
        if texto.chars().count() >= 10 {
            let mut texto = primera_mayuscula(&texto);
            if texto.chars().last().is_some_and(|c| c.is_alphanumeric()) {
                texto.push('.');
            }
            lectura.descripcion = Some(texto);
        }
    }

    // 3. El nombre dicho de forma explícita (solo cuando ya hay una ficha en marcha).
    if con_nombre {
        if let Some((nombre, desde, hasta)) = nombre_explicito(pal, libre) {
            consumir(libre, desde, hasta);
            lectura.nombre = Some(nombre);
        }
    }

    // 4. El aviso «El precio puede cambiar».
    let pide_aviso = (0..n).any(|i| libre[i] && AVISO.contains(&pal[i].norm.as_str()))
        || (0..n).any(|i| libre[i] && pal[i].norm == "puede" && i + 1 < n && ["cambiar", "variar"].contains(&pal[i + 1].norm.as_str()));
    if pide_aviso && lectura.aviso.is_none() {
        let negado = hay(pal, libre, &["sin", "no", "desactiva", "desactivar", "apaga", "apagar"]);
        lectura.aviso = Some(!negado);
        for i in 0..n {
            if libre[i] && (AVISO.contains(&pal[i].norm.as_str()) || ["puede", "cambiar", "variar"].contains(&pal[i].norm.as_str())) {
                libre[i] = false;
            }
        }
    }

    // 5. La imagen (ponerla se hace con el botón de la ficha; Juno solo lo indica).
    for i in 0..n {
        if libre[i] && ["imagen", "imagenes", "foto", "fotografia"].contains(&pal[i].norm.as_str()) {
            lectura.pedir_imagen = true;
            libre[i] = false;
        }
    }

    // 6. El tiempo: «2 días», «1 a 2 días», «3 horas», «mismo día», «sin tiempo fijo».
    let mut i = 0;
    while i < n && lectura.tiempo.is_none() {
        if !libre[i] {
            i += 1;
            continue;
        }
        if pal[i].norm == "mismo" && i + 1 < n && libre[i + 1] && pal[i + 1].norm == "dia" {
            lectura.tiempo = Some(Some("Mismo día".to_string()));
            consumir(libre, i, i + 2);
            break;
        }
        if pal[i].norm == "sin" && i + 1 < n && libre[i + 1] && ["tiempo", "plazo"].contains(&pal[i + 1].norm.as_str()) {
            let mut fin = i + 2;
            if fin < n && libre[fin] && ["fijo", "definido", "estimado"].contains(&pal[fin].norm.as_str()) {
                fin += 1;
            }
            lectura.tiempo = Some(None);
            consumir(libre, i, fin);
            break;
        }
        if es_entero(&pal[i].norm) {
            let mut j = i + 1;
            let mut ultimo = pal[i].norm.clone();
            let mut primero = None;
            if j + 1 < n && libre[j] && libre[j + 1] && ["a", "o", "y", "hasta"].contains(&pal[j].norm.as_str()) && es_entero(&pal[j + 1].norm) {
                primero = Some(pal[i].norm.clone());
                ultimo = pal[j + 1].norm.clone();
                j += 2;
            }
            if j < n && libre[j] {
                if let Some((una, varias)) = unidad(&pal[j].norm) {
                    let texto = match primero {
                        Some(a) => format!("{} a {}", a, cantidad(&ultimo, una, varias)),
                        None => cantidad(&ultimo, una, varias),
                    };
                    lectura.tiempo = Some(Some(texto));
                    consumir(libre, i, j + 1);
                    break;
                }
            }
        }
        i += 1;
    }
    // Respuesta corta a «¿cuánto tarda?»: «2» o «1 a 2» (sin unidad) = días.
    if lectura.tiempo.is_none() && pregunta == "tiempo" {
        if let Some(i) = (0..n).find(|&i| libre[i] && es_entero(&pal[i].norm)) {
            let mut fin = i + 1;
            let mut texto = cantidad(&pal[i].norm, "día", "días");
            if i + 2 < n && libre[i + 1] && libre[i + 2] && ["a", "o", "y", "hasta"].contains(&pal[i + 1].norm.as_str()) && es_entero(&pal[i + 2].norm) {
                texto = format!("{} a {}", pal[i].norm, cantidad(&pal[i + 2].norm, "día", "días"));
                fin = i + 3;
            }
            lectura.tiempo = Some(Some(texto));
            consumir(libre, i, fin);
        }
    }

    // 7. El precio: «120 soles», «S/ 120», «cuesta 120», «a cotizar», «sin precio».
    for i in 0..n {
        if libre[i] && ["cotizar", "cotiza", "cotizacion", "cotizable"].contains(&pal[i].norm.as_str()) {
            lectura.precio = Some(None);
            libre[i] = false;
        }
        if libre[i] && pal[i].norm == "sin" && i + 1 < n && libre[i + 1] && pal[i + 1].norm == "precio" {
            lectura.precio = Some(None);
            consumir(libre, i, i + 2);
        }
    }
    if lectura.precio.is_none() {
        let con_clave = hay(pal, libre, &PRECIO_CLAVE) || pregunta == "precio";
        if con_clave {
            if let Some(i) = (0..n).find(|&i| libre[i] && es_numero(&pal[i].norm)) {
                if let Ok(Some(valor)) = normalizar_precio(&pal[i].norm) {
                    lectura.precio = Some(Some(valor));
                    libre[i] = false;
                }
            }
        }
    }

    // 8. Los equipos: «para laptops y PC», «todos los equipos».
    let todos = (0..n).any(|i| {
        libre[i]
            && ["todos", "todo", "cualquier", "cualquiera", "ambos"].contains(&pal[i].norm.as_str())
            && ((i + 1)..(i + 4).min(n)).any(|j| ["equipo", "equipos", "tipo", "tipos", "dispositivo", "dispositivos"].contains(&pal[j].norm.as_str()))
    }) || (pregunta == "equipos" && hay(pal, libre, &["todos", "todo", "ambos"]));
    if todos {
        lectura.equipos = Some(equipos.iter().map(|e| e.codigo.clone()).collect());
        for i in 0..n {
            if libre[i] && ["todos", "todo", "cualquier", "cualquiera", "ambos", "equipo", "equipos", "tipo", "tipos", "dispositivo", "dispositivos", "los"].contains(&pal[i].norm.as_str()) {
                libre[i] = false;
            }
        }
    } else {
        let (codigos, usadas) = equipos_en(pal, libre, 0, n, equipos);
        if !codigos.is_empty() {
            lectura.equipos_suman = hay(pal, libre, &SUMAR);
            lectura.equipos = Some(codigos);
            for u in usadas {
                libre[u] = false;
            }
        }
    }

    lectura
}

// ---------- ¿De qué servicio habla? ----------

pub enum Busqueda {
    Uno(usize, Vec<usize>), // la posición del servicio en la lista y las palabras del mensaje que lo nombraron
    Varios(Vec<usize>),     // pudo ser cualquiera de estos
    Ninguno,
}

// Busca entre los nombres de servicios el que el mensaje menciona: primero el nombre completo y, si no, sus palabras distintivas
// («el teclado» encuentra «Cambio de teclado»).
pub fn buscar_servicio(pal: &[Palabra], libre: &[bool], nombres: &[String]) -> Busqueda {
    let tw: Vec<&str> = pal.iter().map(|p| p.norm.as_str()).collect();

    // 1. El nombre completo, tal cual, entre las palabras del mensaje.
    let mut completos: Vec<(usize, usize, usize)> = Vec::new(); // (servicio, dónde empieza, cuántas palabras)
    for (k, nombre) in nombres.iter().enumerate() {
        let nw = palabras_de(nombre);
        if nw.is_empty() || nw.len() > tw.len() {
            continue;
        }
        for i in 0..=(tw.len() - nw.len()) {
            if (0..nw.len()).all(|d| libre[i + d] && tw[i + d] == nw[d]) {
                completos.push((k, i, nw.len()));
                break;
            }
        }
    }
    if let Some(maximo) = completos.iter().map(|c| c.2).max() {
        let mejores: Vec<&(usize, usize, usize)> = completos.iter().filter(|c| c.2 == maximo).collect();
        if mejores.len() == 1 {
            let (k, desde, largo) = *mejores[0];
            return Busqueda::Uno(k, (desde..desde + largo).collect());
        }
        return Busqueda::Varios(mejores.iter().map(|c| c.0).collect());
    }

    // 2. Las palabras que distinguen a cada servicio de los demás.
    const GENERICAS: [&str; 18] = [
        "cambio", "servicio", "reparacion", "instalacion", "mantenimiento", "limpieza", "de", "del", "la", "el", "los", "las", "y", "e",
        "para", "con", "en", "equipo",
    ];
    let mut candidatos: Vec<(usize, usize, Vec<usize>)> = Vec::new(); // (servicio, cuántas palabras distintivas se dijeron, dónde se dijeron)
    for (k, nombre) in nombres.iter().enumerate() {
        let nw = palabras_de(nombre);
        let mut distintivas: Vec<String> = nw.iter().filter(|p| !GENERICAS.contains(&p.as_str())).cloned().collect();
        if distintivas.is_empty() {
            distintivas = nw.iter().filter(|p| p.chars().count() >= 4).cloned().collect();
        }
        if distintivas.is_empty() {
            continue;
        }
        let mut posiciones = Vec::new();
        for d in &distintivas {
            if let Some(i) = (0..tw.len()).find(|&i| libre[i] && parecidas(tw[i], d) && !posiciones.contains(&i)) {
                posiciones.push(i);
            }
        }
        if !posiciones.is_empty() {
            candidatos.push((k, posiciones.len(), posiciones));
        }
    }
    match candidatos.iter().map(|c| c.1).max() {
        None => Busqueda::Ninguno,
        Some(maximo) => {
            let mejores: Vec<&(usize, usize, Vec<usize>)> = candidatos.iter().filter(|c| c.1 == maximo).collect();
            if mejores.len() == 1 {
                Busqueda::Uno(mejores[0].0, mejores[0].2.clone())
            } else {
                Busqueda::Varios(mejores.iter().map(|c| c.0).collect())
            }
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn equipos() -> Vec<Equipo> {
        vec![
            Equipo { codigo: "EQLAPTOP1".to_string(), nombre: "Laptop".to_string() },
            Equipo { codigo: "EQPCESCR2".to_string(), nombre: "PC de escritorio".to_string() },
            Equipo { codigo: "EQIMPRES3".to_string(), nombre: "Impresora".to_string() },
        ]
    }

    fn leer_todo(texto: &str, pregunta: &str) -> Lectura {
        let pal = palabras(texto);
        let mut libre = vec![true; pal.len()];
        leer(&pal, &mut libre, pregunta, &equipos(), false)
    }

    #[test]
    fn parte_en_palabras_con_precio_pegado() {
        let pal = palabras("Cuesta S/120, mismo día");
        let normas: Vec<&str> = pal.iter().map(|p| p.norm.as_str()).collect();
        assert_eq!(normas, vec!["cuesta", "s/", "120", "mismo", "dia"]);
        assert!(pal[2].corte);
    }

    #[test]
    fn nombre_de_un_servicio_nuevo() {
        let pal = palabras("Agrega cambio de bisagras para laptops, 120 soles, 2 días");
        let libre = vec![true; pal.len()];
        let (nombre, desde, hasta) = nombre_tras_verbo(&pal, &libre).unwrap();
        assert_eq!(nombre, "Cambio de bisagras");
        assert_eq!((desde, hasta), (1, 4));
    }

    #[test]
    fn nombre_sin_nada_mas() {
        let pal = palabras("agrega limpieza de ventiladores");
        let libre = vec![true; pal.len()];
        assert_eq!(nombre_tras_verbo(&pal, &libre).unwrap().0, "Limpieza de ventiladores");
        let pal = palabras("quiero agregar un servicio de Formateo e instalación de Windows a 60 soles");
        let libre = vec![true; pal.len()];
        assert_eq!(nombre_tras_verbo(&pal, &libre).unwrap().0, "Formateo e instalación de Windows");
    }

    #[test]
    fn sin_nombre_no_inventa() {
        let pal = palabras("quiero agregar un servicio");
        let libre = vec![true; pal.len()];
        assert!(nombre_tras_verbo(&pal, &libre).is_none());
    }

    #[test]
    fn lee_precio_tiempo_y_equipos_de_una_frase() {
        let pal = palabras("Agrega cambio de bisagras para laptops, 120 soles, 2 días");
        let mut libre = vec![true; pal.len()];
        let (_, desde, hasta) = nombre_tras_verbo(&pal, &libre).unwrap();
        consumir(&mut libre, desde, hasta);
        let lectura = leer(&pal, &mut libre, "", &equipos(), false);
        assert_eq!(lectura.precio, Some(Some("120.00".to_string())));
        assert_eq!(lectura.tiempo, Some(Some("2 días".to_string())));
        assert_eq!(lectura.equipos, Some(vec!["EQLAPTOP1".to_string()]));
    }

    #[test]
    fn tiempos_variados() {
        assert_eq!(leer_todo("1 día", "").tiempo, Some(Some("1 día".to_string())));
        assert_eq!(leer_todo("de 2 a 3 días", "").tiempo, Some(Some("2 a 3 días".to_string())));
        assert_eq!(leer_todo("2-3 dias", "").tiempo, Some(Some("2 a 3 días".to_string())));
        assert_eq!(leer_todo("tarda 3 horas", "").tiempo, Some(Some("3 horas".to_string())));
        assert_eq!(leer_todo("el mismo día", "").tiempo, Some(Some("Mismo día".to_string())));
        assert_eq!(leer_todo("sin tiempo fijo", "").tiempo, Some(None));
        assert_eq!(leer_todo("2", "tiempo").tiempo, Some(Some("2 días".to_string())));
        assert_eq!(leer_todo("1 a 2", "tiempo").tiempo, Some(Some("1 a 2 días".to_string())));
        assert_eq!(leer_todo("hola", "").tiempo, None);
    }

    #[test]
    fn precios_variados() {
        assert_eq!(leer_todo("S/ 80", "").precio, Some(Some("80.00".to_string())));
        assert_eq!(leer_todo("cuesta 99,5", "").precio, Some(Some("99.50".to_string())));
        assert_eq!(leer_todo("a cotizar", "").precio, Some(None));
        assert_eq!(leer_todo("sin precio", "").precio, Some(None));
        assert_eq!(leer_todo("150", "precio").precio, Some(Some("150.00".to_string())));
        assert_eq!(leer_todo("150", "").precio, None); // un número suelto, sin saber que es un precio, no se adivina
        assert_eq!(leer_todo("2 días", "").precio, None);
    }

    #[test]
    fn equipos_con_sinonimos_y_todos() {
        assert_eq!(leer_todo("para laptops y pc", "").equipos, Some(vec!["EQLAPTOP1".to_string(), "EQPCESCR2".to_string()]));
        assert_eq!(leer_todo("sirve para computadoras", "").equipos, Some(vec!["EQPCESCR2".to_string()]));
        assert_eq!(leer_todo("todos los equipos", "").equipos.map(|e| e.len()), Some(3));
        assert_eq!(leer_todo("Laptop y PC", "equipos").equipos.map(|e| e.len()), Some(2));
        assert_eq!(leer_todo("también impresoras", "").equipos_suman, true);
        assert_eq!(leer_todo("para laptop", "").equipos_suman, false);
    }

    #[test]
    fn quitar_campos_y_equipos() {
        let l = leer_todo("quita el tiempo, mejor que no se prometa", "");
        assert_eq!(l.quitar, vec!["tiempo"]);
        assert!(l.tiempo.is_none());
        let l = leer_todo("quítale laptop", "");
        assert_eq!(l.equipos_quitar, vec!["EQLAPTOP1".to_string()]);
        assert!(l.equipos.is_none());
        assert!(leer_todo("quita la imagen", "").quitar_imagen);
    }

    #[test]
    fn aviso_de_precio() {
        assert_eq!(leer_todo("avisa que el precio puede cambiar", "").aviso, Some(true));
        assert_eq!(leer_todo("activa el aviso", "").aviso, Some(true));
        assert_eq!(leer_todo("quita el aviso", "").aviso, Some(false));
        assert_eq!(leer_todo("sin aviso", "").aviso, Some(false));
        assert_eq!(leer_todo("el precio puede variar", "").aviso, Some(true));
    }

    #[test]
    fn descripcion_dictada() {
        let l = leer_todo("la descripción es: Cambiamos tus bisagras, revisamos el marco y la tapa", "");
        assert_eq!(l.descripcion.as_deref(), Some("Cambiamos tus bisagras, revisamos el marco y la tapa."));
        // Lo que viene después de «descripción» es texto, no precios ni equipos.
        assert!(l.precio.is_none() && l.equipos.is_none());
        assert!(leer_todo("descripción: corta", "").descripcion.is_none());
    }

    #[test]
    fn imagen() {
        assert!(leer_todo("quiero agregar una imagen", "").pedir_imagen);
        assert!(leer_todo("pon una foto", "").pedir_imagen);
    }

    #[test]
    fn nombre_explicito_en_una_ficha_en_marcha() {
        let pal = palabras("cámbiale el nombre a Reparación de bisagras");
        let mut libre = vec![true; pal.len()];
        let l = leer(&pal, &mut libre, "", &equipos(), true);
        assert_eq!(l.nombre.as_deref(), Some("Reparación de bisagras"));
    }

    fn nombres() -> Vec<String> {
        vec![
            "Cambio de pantalla".to_string(),
            "Cambio de teclado".to_string(),
            "Formateo e instalación de Windows".to_string(),
            "Mantenimiento preventivo".to_string(),
        ]
    }

    fn buscar(texto: &str) -> Busqueda {
        let pal = palabras(texto);
        let libre = vec![true; pal.len()];
        buscar_servicio(&pal, &libre, &nombres())
    }

    #[test]
    fn encuentra_el_servicio_por_su_nombre() {
        match buscar("súbele a 100 el cambio de teclado") {
            Busqueda::Uno(1, palabras) => assert_eq!(palabras.len(), 3),
            _ => panic!("debía encontrar «Cambio de teclado»"),
        }
        assert!(matches!(buscar("quita el servicio de Mantenimiento preventivo"), Busqueda::Uno(3, _)));
    }

    #[test]
    fn encuentra_el_servicio_por_una_palabra_distintiva() {
        assert!(matches!(buscar("cambia el precio del teclado"), Busqueda::Uno(1, _)));
        assert!(matches!(buscar("el de windows"), Busqueda::Uno(2, _)));
        assert!(matches!(buscar("la pantalla cuesta 200"), Busqueda::Uno(0, _)));
        assert!(matches!(buscar("quiero formatear"), Busqueda::Uno(2, _)));
    }

    #[test]
    fn sin_pistas_no_hay_servicio() {
        assert!(matches!(buscar("hola, buenos días"), Busqueda::Ninguno));
    }
}
