// gestion.rs (catalogo): las reglas de crear, editar y eliminar categorías y productos.
// Aquí se decide: se revisa lo que llegó, se genera el código y se llama al repositorio. Sin SQL.
// Quién puede hacerlo (Vendedor o Administrador) lo decide rutas.rs antes de llegar aquí.
// Devuelve el mismo "Gestion" que usa servicio_tecnico, así rutas.rs lo convierte en respuesta HTTP igual.

use sqlx::PgPool;

use super::modelo::{EntradaCategoria, EntradaProducto, Resultado};
use super::repositorio_gestion as repositorio;
use super::validaciones::{validar_categoria, validar_producto, DatosProducto};
use crate::modulos::comun::codigos::generar_codigo;
use crate::modulos::servicio_tecnico::gestion::Gestion;

// Convierte el resultado del repositorio en el de la gestión, con el mensaje para la persona.
// "que" es lo que se estaba guardando ("una categoría") y "en_uso" el motivo si no se puede eliminar.
fn traducir(resultado: Resultado, que: &str, en_uso: &str) -> Gestion {
    match resultado {
        Resultado::Hecho => Gestion::Hecho,
        Resultado::NombreRepetido => Gestion::Repetido(format!("Ya existe {} con ese nombre.", que)),
        Resultado::CategoriaInvalida => Gestion::Invalido(
            "La categoría elegida ya no existe. Recarga la página e inténtalo de nuevo.".to_string(),
        ),
        Resultado::NoEncontrado => Gestion::NoEncontrado,
        Resultado::EnUso => Gestion::Invalido(en_uso.to_string()),
    }
}

// ---------- Categorías ----------

pub async fn crear_categoria(pool: &PgPool, entrada: EntradaCategoria) -> Result<Gestion, sqlx::Error> {
    let nombre = match validar_categoria(&entrada.nombre) {
        Ok(nombre) => nombre,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };
    let resultado = repositorio::crear_categoria(pool, &generar_codigo(), &nombre).await?;
    Ok(traducir(resultado, "una categoría", ""))
}

pub async fn renombrar_categoria(
    pool: &PgPool,
    codigo: &str,
    entrada: EntradaCategoria,
) -> Result<Gestion, sqlx::Error> {
    let nombre = match validar_categoria(&entrada.nombre) {
        Ok(nombre) => nombre,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };
    let resultado = repositorio::renombrar_categoria(pool, codigo, &nombre).await?;
    Ok(traducir(resultado, "una categoría", ""))
}

pub async fn eliminar_categoria(pool: &PgPool, codigo: &str) -> Result<Gestion, sqlx::Error> {
    let resultado = repositorio::eliminar_categoria(pool, codigo).await?;
    Ok(traducir(
        resultado,
        "una categoría",
        "Esa categoría todavía tiene productos. Muévelos a otra categoría o elimínalos primero.",
    ))
}

// ---------- Productos ----------

fn revisar_producto(entrada: &EntradaProducto) -> Result<DatosProducto, String> {
    validar_producto(
        &entrada.categoria,
        &entrada.nombre,
        &entrada.descripcion,
        &entrada.precio,
        entrada.stock,
    )
}

pub async fn crear_producto(pool: &PgPool, entrada: EntradaProducto) -> Result<Gestion, sqlx::Error> {
    let datos = match revisar_producto(&entrada) {
        Ok(datos) => datos,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };
    let resultado = repositorio::crear_producto(pool, &generar_codigo(), &datos).await?;
    Ok(traducir(resultado, "un producto", ""))
}

pub async fn actualizar_producto(
    pool: &PgPool,
    codigo: &str,
    entrada: EntradaProducto,
) -> Result<Gestion, sqlx::Error> {
    let datos = match revisar_producto(&entrada) {
        Ok(datos) => datos,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };
    let resultado = repositorio::actualizar_producto(pool, codigo, &datos).await?;
    Ok(traducir(resultado, "un producto", ""))
}

pub async fn eliminar_producto(pool: &PgPool, codigo: &str) -> Result<Gestion, sqlx::Error> {
    let resultado = repositorio::eliminar_producto(pool, codigo).await?;
    Ok(traducir(
        resultado,
        "un producto",
        "Ese producto tiene apartados pendientes. Entrégalos o cancélalos primero.",
    ))
}
