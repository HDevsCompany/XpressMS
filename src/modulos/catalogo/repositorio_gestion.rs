// repositorio_gestion.rs (catalogo): el SQL que ESCRIBE (altas, cambios y bajas) categorías y productos,
// más la lista de categorías del panel. Aquí no se decide nada: las reglas están en gestion.rs.
//
// "Eliminar" nunca borra la fila: pone Estado = 'E'. Los apartados conservan su historial.
// Los nombres repetidos los frena la base con índices únicos entre registros activos (08_catalogo.sql);
// aquí se reconoce ese error y se convierte en Resultado::NombreRepetido.

use sqlx::error::ErrorKind;
use sqlx::PgPool;

use super::apartados::PENDIENTE;
use super::modelo::{CategoriaPanel, Resultado};
use super::validaciones::DatosProducto;

// ¿El error de la base es "ya existe un registro con ese valor único"?
fn es_nombre_repetido(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|e| matches!(e.kind(), ErrorKind::UniqueViolation))
}

// ---------------------------------------------------------------------------------------------
// Categorías
// ---------------------------------------------------------------------------------------------

// Las categorías activas con cuántos productos activos tiene cada una.
pub async fn listar_categorias(pool: &PgPool) -> Result<Vec<CategoriaPanel>, sqlx::Error> {
    sqlx::query_as::<_, CategoriaPanel>(
        "select c.CategoriasCodigo::text as codigo, c.CategoriasNombre as nombre, \
                (select count(*) from Catalogo.Productos p \
                 where p.ProductosCategoria = c.CategoriasCodigo and p.ProductosEstado = 'A')::bigint as productos \
         from Catalogo.Categorias c \
         where c.CategoriasEstado = 'A' \
         order by c.CategoriasNombre",
    )
    .fetch_all(pool)
    .await
}

pub async fn crear_categoria(
    pool: &PgPool,
    codigo: &str,
    nombre: &str,
) -> Result<Resultado, sqlx::Error> {
    let resultado = sqlx::query(
        "insert into Catalogo.Categorias (CategoriasCodigo, CategoriasNombre) values ($1, $2)",
    )
    .bind(codigo)
    .bind(nombre)
    .execute(pool)
    .await;
    match resultado {
        Ok(_) => Ok(Resultado::Hecho),
        Err(error) if es_nombre_repetido(&error) => Ok(Resultado::NombreRepetido),
        Err(error) => Err(error),
    }
}

pub async fn renombrar_categoria(
    pool: &PgPool,
    codigo: &str,
    nombre: &str,
) -> Result<Resultado, sqlx::Error> {
    let resultado = sqlx::query(
        "update Catalogo.Categorias set CategoriasNombre = $2 \
         where CategoriasCodigo = $1 and CategoriasEstado = 'A'",
    )
    .bind(codigo)
    .bind(nombre)
    .execute(pool)
    .await;
    match resultado {
        Ok(filas) if filas.rows_affected() == 0 => Ok(Resultado::NoEncontrado),
        Ok(_) => Ok(Resultado::Hecho),
        Err(error) if es_nombre_repetido(&error) => Ok(Resultado::NombreRepetido),
        Err(error) => Err(error),
    }
}

// Solo se elimina una categoría sin productos activos (si los tiene, devuelve EnUso).
pub async fn eliminar_categoria(pool: &PgPool, codigo: &str) -> Result<Resultado, sqlx::Error> {
    let filas = sqlx::query(
        "update Catalogo.Categorias set CategoriasEstado = 'E' \
         where CategoriasCodigo = $1 and CategoriasEstado = 'A' \
           and not exists (select 1 from Catalogo.Productos p \
                           where p.ProductosCategoria = $1 and p.ProductosEstado = 'A')",
    )
    .bind(codigo)
    .execute(pool)
    .await?;
    if filas.rows_affected() > 0 {
        return Ok(Resultado::Hecho);
    }
    // No se eliminó: o no existe, o tiene productos.
    let (existe,): (bool,) = sqlx::query_as(
        "select exists (select 1 from Catalogo.Categorias \
                        where CategoriasCodigo = $1 and CategoriasEstado = 'A')",
    )
    .bind(codigo)
    .fetch_one(pool)
    .await?;
    Ok(if existe { Resultado::EnUso } else { Resultado::NoEncontrado })
}

// ---------------------------------------------------------------------------------------------
// Productos (la lista para el panel está en repositorio.rs: listar_productos)
// ---------------------------------------------------------------------------------------------

// "$5::text::numeric": el precio viaja como texto y la base lo convierte a número.
// El INSERT sale de un SELECT sobre la categoría: si la categoría no existe o fue eliminada, no inserta nada.
pub async fn crear_producto(
    pool: &PgPool,
    codigo: &str,
    datos: &DatosProducto,
) -> Result<Resultado, sqlx::Error> {
    let resultado = sqlx::query(
        "insert into Catalogo.Productos \
            (ProductosCodigo, ProductosCategoria, ProductosNombre, ProductosDescripcion, \
             ProductosPrecio, ProductosStock) \
         select $1, c.CategoriasCodigo, $3, $4, $5::text::numeric, $6 \
         from Catalogo.Categorias c \
         where c.CategoriasCodigo = $2 and c.CategoriasEstado = 'A'",
    )
    .bind(codigo)
    .bind(&datos.categoria)
    .bind(&datos.nombre)
    .bind(&datos.descripcion)
    .bind(&datos.precio)
    .bind(datos.stock)
    .execute(pool)
    .await;
    match resultado {
        Ok(filas) if filas.rows_affected() == 0 => Ok(Resultado::CategoriaInvalida),
        Ok(_) => Ok(Resultado::Hecho),
        Err(error) if es_nombre_repetido(&error) => Ok(Resultado::NombreRepetido),
        Err(error) => Err(error),
    }
}

pub async fn actualizar_producto(
    pool: &PgPool,
    codigo: &str,
    datos: &DatosProducto,
) -> Result<Resultado, sqlx::Error> {
    let resultado = sqlx::query(
        "update Catalogo.Productos p \
         set ProductosCategoria = c.CategoriasCodigo, ProductosNombre = $3, ProductosDescripcion = $4, \
             ProductosPrecio = $5::text::numeric, ProductosStock = $6, ProductosFechaActualizacion = now() \
         from Catalogo.Categorias c \
         where p.ProductosCodigo = $1 and p.ProductosEstado = 'A' \
           and c.CategoriasCodigo = $2 and c.CategoriasEstado = 'A'",
    )
    .bind(codigo)
    .bind(&datos.categoria)
    .bind(&datos.nombre)
    .bind(&datos.descripcion)
    .bind(&datos.precio)
    .bind(datos.stock)
    .execute(pool)
    .await;
    match resultado {
        Ok(filas) if filas.rows_affected() > 0 => Ok(Resultado::Hecho),
        Ok(_) => {
            // No se cambió nada: o el producto no existe, o la categoría elegida ya no es válida.
            let (existe,): (bool,) = sqlx::query_as(
                "select exists (select 1 from Catalogo.Productos \
                                where ProductosCodigo = $1 and ProductosEstado = 'A')",
            )
            .bind(codigo)
            .fetch_one(pool)
            .await?;
            Ok(if existe { Resultado::CategoriaInvalida } else { Resultado::NoEncontrado })
        }
        Err(error) if es_nombre_repetido(&error) => Ok(Resultado::NombreRepetido),
        Err(error) => Err(error),
    }
}

// Solo se elimina un producto sin apartados pendientes (si los tiene, devuelve EnUso).
pub async fn eliminar_producto(pool: &PgPool, codigo: &str) -> Result<Resultado, sqlx::Error> {
    let filas = sqlx::query(
        "update Catalogo.Productos set ProductosEstado = 'E', ProductosFechaActualizacion = now() \
         where ProductosCodigo = $1 and ProductosEstado = 'A' \
           and not exists (select 1 from Catalogo.Reservas r \
                           where r.ReservasProducto = $1 and r.ReservasEstadoReserva = $2)",
    )
    .bind(codigo)
    .bind(PENDIENTE)
    .execute(pool)
    .await?;
    if filas.rows_affected() > 0 {
        return Ok(Resultado::Hecho);
    }
    let (existe,): (bool,) = sqlx::query_as(
        "select exists (select 1 from Catalogo.Productos \
                        where ProductosCodigo = $1 and ProductosEstado = 'A')",
    )
    .bind(codigo)
    .fetch_one(pool)
    .await?;
    Ok(if existe { Resultado::EnUso } else { Resultado::NoEncontrado })
}
