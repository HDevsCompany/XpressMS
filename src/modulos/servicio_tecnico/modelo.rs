// modelo.rs (servicio_tecnico): estructuras del módulo de servicios técnicos.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Un servicio que ofrece la empresa (fila de ServicioTecnico.Servicios).
// Los nombres de los campos coinciden con los alias ("as nombre") de las consultas SQL.
#[derive(FromRow, Serialize)]
pub struct Servicio {
    pub codigo: String,
    pub nombre: String,
    pub descripcion: String,
    pub precio_base: Option<String>, // el precio llega como texto ("60.00"); vacío = "a cotizar"
    pub tiempo_estimado: Option<String>,
    pub aviso_precio: bool, // true = la ficha avisa que el precio puede cambiar (el costo final se confirma al revisar el equipo)
}

// Un tipo de equipo que atiende la empresa (fila de ServicioTecnico.TiposEquipo): Laptop, Impresora...
#[derive(FromRow, Serialize)]
pub struct TipoEquipo {
    pub codigo: String,
    pub nombre: String,
}

// ---------- Estructuras del panel de gestión (solo las usa el personal) ----------

// Un tipo de equipo con la cantidad de servicios activos que lo usan (para mostrarlo en el panel).
#[derive(FromRow, Serialize)]
pub struct TipoEquipoPanel {
    pub codigo: String,
    pub nombre: String,
    pub servicios: i64,
}

// Un servicio junto con los tipos de equipo para los que sirve.
// "flatten" pone los campos del servicio al mismo nivel que "equipos" en el JSON.
#[derive(Serialize)]
pub struct ServicioPanel {
    #[serde(flatten)]
    pub servicio: Servicio,
    pub equipos: Vec<TipoEquipo>,
    // La dirección de su imagen ("/publico/servicios/AAAAAAAAA/imagen?v=1730000000000"); None = no tiene imagen.
    pub imagen: Option<String>,
}

// Lo que envía el navegador al crear o editar un servicio (todavía sin revisar).
#[derive(Deserialize)]
pub struct EntradaServicio {
    pub nombre: String,
    pub descripcion: String,
    pub precio_base: Option<String>,
    pub tiempo_estimado: Option<String>,
    pub equipos: Vec<String>, // códigos de los tipos de equipo elegidos
    // «El precio puede cambiar». Si no viene, al crear queda apagado y al editar se conserva el que ya tenía.
    #[serde(default)]
    pub aviso_precio: Option<bool>,
}

// Lo que envía el navegador al crear o renombrar un tipo de equipo.
#[derive(Deserialize)]
pub struct EntradaEquipo {
    pub nombre: String,
}

// Cómo terminó una operación sobre la base.
pub enum Resultado {
    Hecho,
    NombreRepetido, // ya hay otro registro activo con ese nombre
    EquipoInvalido, // algún tipo de equipo elegido no existe o fue eliminado
    NoEncontrado,   // el registro que se quería cambiar no existe o ya fue eliminado
}

// ---------- Órdenes de servicio (citas y seguimiento) ----------

// Un estado del catálogo ServicioTecnico.EstadosOrden: ("CIT", "Cita agendada").
#[derive(FromRow, Serialize, Clone)]
pub struct EstadoOrden {
    pub codigo: String,
    pub nombre: String,
}

// Una fila del historial de una orden: a qué estado pasó y cuándo (la fecha ya llega como texto, hora de Lima).
#[derive(FromRow)]
pub struct RegistroHistorial {
    pub estado: String, // código del estado: "CIT", "REC"...
    pub fecha: String,  // "03/11 09:05"
}

// Un paso del seguimiento que ve el cliente. "estado" es "hecho", "actual" o "pendiente".
#[derive(Serialize, Clone)]
pub struct Hito {
    pub nombre: String,
    pub fecha: Option<String>,
    pub estado: String,
}

// Una orden en forma resumida (para elegir entre varias): su código, el servicio y en qué va.
#[derive(FromRow)]
pub struct ResumenOrden {
    pub codigo: String,
    pub servicio: String,
    pub estado_nombre: String,
    pub fecha_cita: String, // "2026-11-09 15:00" (hora de Lima)
}

// Todo lo que el cliente ve de una orden suya.
#[derive(FromRow)]
pub struct DetalleOrden {
    pub codigo: String,
    pub servicio: String,
    pub equipo: Option<String>,
    pub problema: String,
    pub fecha_cita: String, // "2026-11-09 15:00" (hora de Lima)
    pub estado_codigo: String,
    pub estado_nombre: String,
}

// Los datos con los que se agenda una cita (ya revisados por el backend).
pub struct NuevaOrden {
    pub cliente: String,
    pub servicio: String,
    pub tipo_equipo: Option<String>,
    pub problema: String,
    pub fecha_hora: String, // "2026-11-09 15:00" (hora de Lima)
}

// Cómo terminó el intento de agendar.
pub enum ResultadoCita {
    Creada(String), // el código de la orden
    SinCupo,        // esa hora se llenó mientras la persona decidía
}

// ---------- Panel de órdenes (personal) ----------

// Una orden tal como la ve el personal en el panel: incluye al cliente (nombre y teléfono, para contactarlo) y al técnico.
#[derive(FromRow, Serialize)]
pub struct OrdenPanel {
    pub codigo: String,
    pub cliente: String,
    pub telefono: String,
    pub servicio: String,
    pub equipo: Option<String>,
    pub problema: String,
    pub fecha_cita: String, // "2026-11-09 15:00" (hora de Lima)
    pub estado_codigo: String,
    pub estado_nombre: String,
    pub tecnico_codigo: Option<String>,
    pub tecnico_nombre: Option<String>, // "PEDRO JUAN GUEVARA"
}

// Una fila del historial de una orden, para el detalle del panel.
#[derive(FromRow, Serialize)]
pub struct HistorialPanel {
    pub estado: String,          // nombre del estado: "En revisión"
    pub fecha: String,           // "03/11/2026 09:05"
    pub usuario: Option<String>, // quién lo cambió (vacío si fue el propio cliente)
    pub nota: Option<String>,
}

// Un técnico al que se le puede asignar una orden.
#[derive(FromRow, Serialize)]
pub struct TecnicoOpcion {
    pub codigo: String,
    pub nombre: String,
}

// Lo que se envía al abrir una orden en el panel: la orden, su historial y a qué estados puede pasar.
#[derive(Serialize)]
pub struct DetalleOrdenPanel {
    pub orden: OrdenPanel,
    pub historial: Vec<HistorialPanel>,
    pub siguientes: Vec<EstadoOrden>, // vacío si la orden ya terminó (entregada o cancelada)
}

// Lo que envía el navegador al asignar un técnico (None = dejarla sin técnico).
#[derive(Deserialize)]
pub struct EntradaTecnico {
    pub tecnico: Option<String>,
}

// Lo que envía el navegador al cambiar el estado de una orden.
#[derive(Deserialize)]
pub struct EntradaEstado {
    pub estado: String,
    pub nota: Option<String>,
}
