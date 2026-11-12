// repositorio.rs (catalogo): el SQL de lectura del catálogo y todo el SQL de los apartados.
// Aquí no se decide nada: los plazos y las reglas están en apartados.rs. Las categorías y productos
// "eliminados" (Estado = 'E') siguen en la base pero no se muestran. Las horas salen en hora de Lima.
// Las consultas son textos completos escritos en el código (sqlx 0.9 no acepta SQL armado con format!).

use sqlx::PgPool;

use super::apartados::{CANCELADO, ENTREGADO, PENDIENTE, VENCIDO};
use super::modelo::{Apartado, Caracteristica, Categoria, Imagen, Producto, ResultadoApartado, ResultadoPedido};
use crate::modulos::comun::codigos::generar_codigo;

// El SELECT común de un producto (con el nombre de su categoría). "$n::text" entrega el precio como texto.
macro_rules! select_producto {
    () => {
        "select p.ProductosCodigo::text as codigo, p.ProductosCategoria::text as categoria_codigo, \
                c.CategoriasNombre as categoria_nombre, p.ProductosNombre as nombre, \
                p.ProductosDescripcion as descripcion, p.ProductosPrecio::text as precio, \
                p.ProductosStock as stock \
         from Catalogo.Productos p \
         join Catalogo.Categorias c on c.CategoriasCodigo = p.ProductosCategoria "
    };
}

// El SELECT común de un apartado (con producto, estado y cliente).
macro_rules! select_apartado {
    () => {
        "select r.ReservasCodigo::text as codigo, coalesce(r.ReservasPedido::text, '') as pedido, \
                p.ProductosNombre as producto, \
                r.ReservasCantidad as cantidad, r.ReservasPrecioUnitario::text as precio_unitario, \
                (r.ReservasPrecioUnitario * r.ReservasCantidad)::text as total, \
                r.ReservasEstadoReserva::text as estado_codigo, e.EstadosReservaNombre as estado_nombre, \
                to_char(r.ReservasFechaVence at time zone 'America/Lima', 'YYYY-MM-DD HH24:MI') as vence, \
                c.ClientesNombreCompleto::text as cliente, c.ClientesTelefono::text as telefono \
         from Catalogo.Reservas r \
         join Catalogo.Productos p on p.ProductosCodigo = r.ReservasProducto \
         join Catalogo.EstadosReserva e on e.EstadosReservaCodigo = r.ReservasEstadoReserva \
         join Acceso.Clientes c on c.ClientesCodigo = r.ReservasCliente "
    };
}

// ---------------------------------------------------------------------------------------------
// Catálogo
// ---------------------------------------------------------------------------------------------

// Las categorías activas que tienen al menos un producto activo, ordenadas por nombre.
pub async fn listar_categorias(pool: &PgPool) -> Result<Vec<Categoria>, sqlx::Error> {
    sqlx::query_as::<_, Categoria>(
        "select c.CategoriasCodigo::text as codigo, c.CategoriasNombre as nombre \
         from Catalogo.Categorias c \
         where c.CategoriasEstado = 'A' \
           and exists (select 1 from Catalogo.Productos p \
                       where p.ProductosCategoria = c.CategoriasCodigo and p.ProductosEstado = 'A') \
         order by c.CategoriasNombre",
    )
    .fetch_all(pool)
    .await
}

// Todos los productos activos (para el panel), por categoría y nombre.
pub async fn listar_productos(pool: &PgPool) -> Result<Vec<Producto>, sqlx::Error> {
    sqlx::query_as::<_, Producto>(concat!(
        select_producto!(),
        "where p.ProductosEstado = 'A' and c.CategoriasEstado = 'A' \
         order by c.CategoriasNombre, p.ProductosNombre"
    ))
    .fetch_all(pool)
    .await
}

// Los productos activos de una categoría (para el asistente). Los agotados también salen, marcados por su stock.
pub async fn listar_productos_de_categoria(
    pool: &PgPool,
    categoria: &str,
) -> Result<Vec<Producto>, sqlx::Error> {
    sqlx::query_as::<_, Producto>(concat!(
        select_producto!(),
        "where p.ProductosEstado = 'A' and p.ProductosCategoria = $1 \
         order by p.ProductosNombre"
    ))
    .bind(categoria)
    .fetch_all(pool)
    .await
}

// Un producto activo por su código (None si no existe o fue eliminado).
pub async fn obtener_producto(pool: &PgPool, codigo: &str) -> Result<Option<Producto>, sqlx::Error> {
    sqlx::query_as::<_, Producto>(concat!(
        select_producto!(),
        "where p.ProductosCodigo = $1 and p.ProductosEstado = 'A'"
    ))
    .bind(codigo)
    .fetch_optional(pool)
    .await
}

// Todas las características activas de los productos (para armar las fichas), en el orden en que se muestran.
pub async fn listar_caracteristicas(pool: &PgPool) -> Result<Vec<Caracteristica>, sqlx::Error> {
    sqlx::query_as::<_, Caracteristica>(
        "select c.CaracteristicasProducto::text as producto, c.CaracteristicasGrupo as grupo, \
                c.CaracteristicasTitulo as titulo, c.CaracteristicasValor as valor, c.CaracteristicasIcono as icono \
         from Catalogo.Caracteristicas c \
         where c.CaracteristicasEstado = 'A' \
         order by c.CaracteristicasProducto, c.CaracteristicasOrden, c.CaracteristicasTitulo",
    )
    .fetch_all(pool)
    .await
}

// Todas las imágenes activas de los productos (la galería de cada ficha), en su orden.
pub async fn listar_imagenes(pool: &PgPool) -> Result<Vec<Imagen>, sqlx::Error> {
    sqlx::query_as::<_, Imagen>(
        "select i.ImagenesProducto::text as producto, i.ImagenesEtiqueta as etiqueta, i.ImagenesUrl as url \
         from Catalogo.Imagenes i \
         where i.ImagenesEstado = 'A' \
         order by i.ImagenesProducto, i.ImagenesOrden",
    )
    .fetch_all(pool)
    .await
}

// ---------------------------------------------------------------------------------------------
// Apartados
// ---------------------------------------------------------------------------------------------

// Marca como "Vencido" los apartados pendientes cuyo plazo pasó y devuelve sus unidades al stock.
// Todo en UNA sola sentencia: o se hacen las dos cosas o ninguna. Devuelve cuántos venció.
pub async fn vencer_apartados(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let resultado = sqlx::query(
        "with vencidos as ( \
             update Catalogo.Reservas \
             set ReservasEstadoReserva = $1, ReservasFechaActualizacion = now() \
             where ReservasEstadoReserva = $2 and ReservasFechaVence < now() \
             returning ReservasProducto, ReservasCantidad \
         ) \
         update Catalogo.Productos p \
         set ProductosStock = p.ProductosStock + v.total, ProductosFechaActualizacion = now() \
         from (select ReservasProducto as producto, sum(ReservasCantidad)::int as total \
               from vencidos group by ReservasProducto) v \
         where p.ProductosCodigo = v.producto",
    )
    .bind(VENCIDO)
    .bind(PENDIENTE)
    .execute(pool)
    .await?;
    Ok(resultado.rows_affected())
}

// Aparta unidades de un producto para un cliente: descuenta el stock y crea el apartado, en una transacción.
// El descuento solo ocurre si todavía hay stock suficiente (la condición "stock >= cantidad" va en el
// mismo UPDATE, así dos clientes no pueden apartar la misma última unidad). El precio queda fijado.
pub async fn crear_apartado(
    pool: &PgPool,
    cliente: &str,
    producto: &str,
    cantidad: i32,
    validez_dias: i32,
) -> Result<ResultadoApartado, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let descontado: Option<(String,)> = sqlx::query_as(
        "update Catalogo.Productos \
         set ProductosStock = ProductosStock - $2, ProductosFechaActualizacion = now() \
         where ProductosCodigo = $1 and ProductosEstado = 'A' and ProductosStock >= $2 \
         returning ProductosPrecio::text",
    )
    .bind(producto)
    .bind(cantidad)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((precio,)) = descontado else {
        return Ok(ResultadoApartado::SinStock); // sin commit: no se toca nada
    };

    let codigo = generar_codigo();
    sqlx::query(
        "insert into Catalogo.Reservas \
            (ReservasCodigo, ReservasCliente, ReservasProducto, ReservasCantidad, \
             ReservasPrecioUnitario, ReservasEstadoReserva, ReservasFechaVence) \
         values ($1, $2, $3, $4, $5::text::numeric, $6, now() + ($7::int * interval '1 day'))",
    )
    .bind(&codigo)
    .bind(cliente)
    .bind(producto)
    .bind(cantidad)
    .bind(&precio)
    .bind(PENDIENTE)
    .bind(validez_dias)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(ResultadoApartado::Creado(codigo))
}

// Aparta todo un carrito para un cliente en UNA transacción: o se aparta todo o no se aparta nada. Cada producto se descuenta
// del stock con la misma condición de siempre («stock >= cantidad» en el mismo UPDATE) y se crea un apartado por producto;
// todos comparten el mismo código de pedido, que es el código de retiro que recibe el cliente. "items" son (producto, cantidad).
pub async fn crear_pedido(
    pool: &PgPool,
    cliente: &str,
    items: &[(String, i32)],
    validez_dias: i32,
) -> Result<ResultadoPedido, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let pedido = generar_codigo();

    for (producto, cantidad) in items {
        let descontado: Option<(String,)> = sqlx::query_as(
            "update Catalogo.Productos \
             set ProductosStock = ProductosStock - $2, ProductosFechaActualizacion = now() \
             where ProductosCodigo = $1 and ProductosEstado = 'A' and ProductosStock >= $2 \
             returning ProductosPrecio::text",
        )
        .bind(producto)
        .bind(*cantidad)
        .fetch_optional(&mut *tx)
        .await?;
        let Some((precio,)) = descontado else {
            return Ok(ResultadoPedido::SinStock(producto.clone())); // sin commit: no se toca nada
        };

        sqlx::query(
            "insert into Catalogo.Reservas \
                (ReservasCodigo, ReservasPedido, ReservasCliente, ReservasProducto, ReservasCantidad, \
                 ReservasPrecioUnitario, ReservasEstadoReserva, ReservasFechaVence) \
             values ($1, $2, $3, $4, $5, $6::text::numeric, $7, now() + ($8::int * interval '1 day'))",
        )
        .bind(generar_codigo())
        .bind(&pedido)
        .bind(cliente)
        .bind(producto)
        .bind(*cantidad)
        .bind(&precio)
        .bind(PENDIENTE)
        .bind(validez_dias)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(ResultadoPedido::Creado(pedido))
}

// Los apartados de un pedido, solo si son de ese cliente.
pub async fn listar_pedido_de_cliente(
    pool: &PgPool,
    pedido: &str,
    cliente: &str,
) -> Result<Vec<Apartado>, sqlx::Error> {
    sqlx::query_as::<_, Apartado>(concat!(
        select_apartado!(),
        "where r.ReservasPedido = $1 and r.ReservasCliente = $2 \
         order by p.ProductosNombre"
    ))
    .bind(pedido)
    .bind(cliente)
    .fetch_all(pool)
    .await
}

// Los últimos apartados de un cliente, el más reciente primero.
pub async fn listar_apartados_de_cliente(
    pool: &PgPool,
    cliente: &str,
    limite: i64,
) -> Result<Vec<Apartado>, sqlx::Error> {
    sqlx::query_as::<_, Apartado>(concat!(
        select_apartado!(),
        "where r.ReservasCliente = $1 \
         order by r.ReservasFechaRegistro desc, r.ReservasCodigo \
         limit $2"
    ))
    .bind(cliente)
    .bind(limite)
    .fetch_all(pool)
    .await
}

// Un apartado por su código, solo si es de ese cliente (nadie ve apartados ajenos adivinando códigos).
pub async fn obtener_apartado_de_cliente(
    pool: &PgPool,
    codigo: &str,
    cliente: &str,
) -> Result<Option<Apartado>, sqlx::Error> {
    sqlx::query_as::<_, Apartado>(concat!(
        select_apartado!(),
        "where r.ReservasCodigo = $1 and r.ReservasCliente = $2"
    ))
    .bind(codigo)
    .bind(cliente)
    .fetch_optional(pool)
    .await
}

// Los apartados para el panel: primero los pendientes (el que vence antes, arriba) y después los demás
// (el más reciente arriba). Se limita a 300.
pub async fn listar_apartados_panel(pool: &PgPool) -> Result<Vec<Apartado>, sqlx::Error> {
    sqlx::query_as::<_, Apartado>(concat!(
        select_apartado!(),
        "order by (r.ReservasEstadoReserva <> $1), \
                  (case when r.ReservasEstadoReserva = $1 then 1 else -1 end) \
                  * extract(epoch from (case when r.ReservasEstadoReserva = $1 \
                                             then r.ReservasFechaVence else r.ReservasFechaActualizacion end)) \
         limit 300"
    ))
    .bind(PENDIENTE)
    .fetch_all(pool)
    .await
}

// Un apartado por su código, sin importar de quién sea (solo para el personal).
pub async fn obtener_apartado(pool: &PgPool, codigo: &str) -> Result<Option<Apartado>, sqlx::Error> {
    sqlx::query_as::<_, Apartado>(concat!(select_apartado!(), "where r.ReservasCodigo = $1"))
        .bind(codigo)
        .fetch_optional(pool)
        .await
}

// Cancela un apartado pendiente y devuelve sus unidades al stock (transacción).
// - El cliente cancela el suyo: cliente = Some(código), usuario = None.
// - El personal cancela cualquiera: cliente = None, usuario = Some(código).
// Devuelve false si no existe, no es de ese cliente o ya no está pendiente.
pub async fn cancelar_apartado(
    pool: &PgPool,
    codigo: &str,
    cliente: Option<&str>,
    usuario: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let cancelado: Option<(String, i32)> = sqlx::query_as(
        "update Catalogo.Reservas \
         set ReservasEstadoReserva = $3, ReservasUsuario = $4, ReservasFechaActualizacion = now() \
         where ReservasCodigo = $1 and ReservasEstadoReserva = $5 \
           and ($2::text is null or ReservasCliente::text = $2::text) \
         returning ReservasProducto::text, ReservasCantidad",
    )
    .bind(codigo)
    .bind(cliente)
    .bind(CANCELADO)
    .bind(usuario)
    .bind(PENDIENTE)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((producto, cantidad)) = cancelado else {
        return Ok(false);
    };

    sqlx::query(
        "update Catalogo.Productos \
         set ProductosStock = ProductosStock + $2, ProductosFechaActualizacion = now() \
         where ProductosCodigo = $1",
    )
    .bind(producto)
    .bind(cantidad)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

// El personal entrega un apartado pendiente. El stock ya estaba descontado, así que no cambia.
// Devuelve false si no existe o ya no está pendiente.
pub async fn entregar_apartado(
    pool: &PgPool,
    codigo: &str,
    usuario: &str,
) -> Result<bool, sqlx::Error> {
    let resultado = sqlx::query(
        "update Catalogo.Reservas \
         set ReservasEstadoReserva = $3, ReservasUsuario = $2, ReservasFechaActualizacion = now() \
         where ReservasCodigo = $1 and ReservasEstadoReserva = $4",
    )
    .bind(codigo)
    .bind(usuario)
    .bind(ENTREGADO)
    .bind(PENDIENTE)
    .execute(pool)
    .await?;
    Ok(resultado.rows_affected() > 0)
}
