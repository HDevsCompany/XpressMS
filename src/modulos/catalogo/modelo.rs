// modelo.rs (catalogo): estructuras del catálogo de productos y de los apartados.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Una categoría de productos (fila de Catalogo.Categorias).
#[derive(FromRow, Serialize, Clone)]
pub struct Categoria {
    pub codigo: String,
    pub nombre: String,
}

// Una categoría con cuántos productos activos tiene (para el panel).
#[derive(FromRow, Serialize)]
pub struct CategoriaPanel {
    pub codigo: String,
    pub nombre: String,
    pub productos: i64,
}

// Un producto con el nombre de su categoría (los nombres coinciden con los alias del SQL).
#[derive(FromRow, Serialize, Clone)]
pub struct Producto {
    pub codigo: String,
    pub categoria_codigo: String,
    pub categoria_nombre: String,
    pub nombre: String,
    pub descripcion: String,
    pub precio: String, // llega como texto ("1299.00"), así no hacen falta decimales de computadora
    pub stock: i32,     // unidades disponibles para apartar
}

// Un apartado: lo que ve el cliente y, además, quién es el cliente (para el personal).
#[derive(FromRow, Serialize)]
pub struct Apartado {
    pub codigo: String,
    pub pedido: String, // código del pedido (carrito) al que pertenece; vacío si se apartó un solo producto
    pub producto: String,
    pub cantidad: i32,
    pub precio_unitario: String,
    pub total: String,
    pub estado_codigo: String, // "PEN", "ENT", "CAN", "VEN"
    pub estado_nombre: String, // "Apartado", "Entregado"...
    pub vence: String,         // "2026-11-05 18:46" (hora de Lima)
    pub cliente: String,
    pub telefono: String,
}

// ---------- Lo que envía el navegador al crear o editar (todavía sin revisar) ----------

#[derive(Deserialize)]
pub struct EntradaCategoria {
    pub nombre: String,
}

#[derive(Deserialize)]
pub struct EntradaProducto {
    pub categoria: String, // código de la categoría
    pub nombre: String,
    pub descripcion: String,
    pub precio: String,
    pub stock: i64,
}

// Cómo terminó una operación del panel sobre la base.
pub enum Resultado {
    Hecho,
    NombreRepetido,     // ya hay otro registro activo con ese nombre
    CategoriaInvalida,  // la categoría elegida no existe o fue eliminada
    NoEncontrado,       // el registro que se quería cambiar no existe o ya fue eliminado
    EnUso,              // no se puede eliminar: tiene productos o apartados pendientes
}

// Cómo terminó el intento de apartar.
pub enum ResultadoApartado {
    Creado(String), // el código del apartado
    SinStock,       // ya no quedan suficientes unidades (o el producto dejó de estar disponible)
}

// Un dato de la ficha de un producto («MEMORIA» · «CAPACIDAD» · «16 GB»). Si tiene ícono, también sale como caja destacada.
#[derive(FromRow, Clone)]
pub struct Caracteristica {
    pub producto: String, // código del producto
    pub grupo: String,
    pub titulo: String,
    pub valor: String,
    pub icono: Option<String>,
}

// Una vista de la galería de un producto (la url puede ser una dirección o «dibujo:lateral»).
#[derive(FromRow, Clone)]
pub struct Imagen {
    pub producto: String, // código del producto
    pub etiqueta: String,
    pub url: String,
}

// Cómo terminó el intento de apartar todo un carrito.
pub enum ResultadoPedido {
    Creado(String),   // el código del pedido (el código de retiro)
    SinStock(String), // el código del primer producto que ya no alcanzó (no se apartó nada)
}
