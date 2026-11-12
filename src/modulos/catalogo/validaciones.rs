// validaciones.rs (catalogo): reglas para revisar lo que escribe el personal al crear o editar categorías
// y productos. Son funciones "puras": se prueban solas con "cargo test". La base solo guarda lo que ya pasó por aquí.

use crate::modulos::servicio_tecnico::validaciones::{limpiar_texto, normalizar_precio};

const STOCK_MAXIMO: i64 = 100_000;

// Un producto ya revisado y limpio, listo para guardarse.
pub struct DatosProducto {
    pub categoria: String,
    pub nombre: String,
    pub descripcion: String,
    pub precio: String, // "1299.00"
    pub stock: i32,
}

// Nombre de una categoría: entre 2 y 60 caracteres.
pub fn validar_categoria(nombre: &str) -> Result<String, String> {
    limpiar_texto(nombre, 2, 60)
        .ok_or_else(|| "El nombre de la categoría debe tener entre 2 y 60 caracteres.".to_string())
}

// Revisa todos los campos de un producto. Si hay un error, devuelve el mensaje para mostrarle a la persona.
pub fn validar_producto(
    categoria: &str,
    nombre: &str,
    descripcion: &str,
    precio: &str,
    stock: i64,
) -> Result<DatosProducto, String> {
    let categoria = categoria.trim().to_string();
    if categoria.chars().count() != 9 {
        return Err("Elige una categoría.".to_string());
    }
    let nombre = limpiar_texto(nombre, 3, 120)
        .ok_or("El nombre del producto debe tener entre 3 y 120 caracteres.")?;
    let descripcion = limpiar_texto(descripcion, 10, 500)
        .ok_or("La descripción debe tener entre 10 y 500 caracteres.")?;
    // A diferencia de los servicios, un producto siempre tiene precio.
    let precio = normalizar_precio(precio)?.ok_or("El precio es obligatorio.")?;
    if !(0..=STOCK_MAXIMO).contains(&stock) {
        return Err("El stock debe ser un número entre 0 y 100000.".to_string());
    }
    Ok(DatosProducto {
        categoria,
        nombre,
        descripcion,
        precio,
        stock: stock as i32,
    })
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn categorias() {
        assert_eq!(validar_categoria("  Laptops  "), Ok("Laptops".to_string()));
        assert!(validar_categoria("a").is_err());
        assert!(validar_categoria(&"a".repeat(61)).is_err());
    }

    #[test]
    fn producto_valido_se_limpia() {
        let datos = validar_producto(
            "CAT000001",
            "  Mouse   inalámbrico  ",
            "Mouse óptico inalámbrico de 1600 DPI con receptor USB.",
            "S/ 45,5",
            12,
        )
        .ok()
        .unwrap();
        assert_eq!(datos.nombre, "Mouse inalámbrico");
        assert_eq!(datos.precio, "45.50");
        assert_eq!(datos.stock, 12);
    }

    #[test]
    fn producto_invalido() {
        let ok = |c: &str, n: &str, d: &str, p: &str, s: i64| validar_producto(c, n, d, p, s).is_ok();
        let desc = "Descripción suficientemente larga.";
        assert!(ok("CAT000001", "Mouse", desc, "45", 0)); // stock 0 es válido (agotado)
        assert!(!ok("CAT1", "Mouse", desc, "45", 1)); // categoría mal
        assert!(!ok("CAT000001", "Mo", desc, "45", 1)); // nombre corto
        assert!(!ok("CAT000001", "Mouse", "corta", "45", 1)); // descripción corta
        assert!(!ok("CAT000001", "Mouse", desc, "", 1)); // sin precio
        assert!(!ok("CAT000001", "Mouse", desc, "abc", 1)); // precio inválido
        assert!(!ok("CAT000001", "Mouse", desc, "45", -1)); // stock negativo
        assert!(!ok("CAT000001", "Mouse", desc, "45", 100_001)); // stock enorme
    }
}
