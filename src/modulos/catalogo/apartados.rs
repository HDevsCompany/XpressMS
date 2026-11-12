// apartados.rs (catalogo): las reglas de los apartados (cuánto duran, cuántas unidades, quién hace qué).
// Aquí se decide; el SQL vive en repositorio.rs. Las funciones "puras" se prueban solas con "cargo test".
//
// Un apartado nace "Apartado" (PEN) con el stock ya descontado y puede terminar de tres formas:
//   PEN -> ENT  el personal lo entrega y el cliente paga en tienda
//   PEN -> CAN  lo cancela el cliente o el personal (el stock vuelve)
//   PEN -> VEN  pasó el plazo sin recogerlo (el stock vuelve solo; lo hace vencer_apartados)

use sqlx::PgPool;

use super::modelo::{Apartado, ResultadoApartado, ResultadoPedido};
use super::repositorio;
use crate::modulos::servicio_tecnico::gestion::Gestion;

pub const PENDIENTE: &str = "PEN";
pub const ENTREGADO: &str = "ENT";
pub const CANCELADO: &str = "CAN";
pub const VENCIDO: &str = "VEN";

// Cuántos días tiene el cliente para recoger lo que apartó.
pub const VALIDEZ_DIAS: i32 = 2;
// Máximo de unidades que se pueden apartar de un mismo producto en un apartado.
pub const MAX_UNIDADES: i32 = 5;

// Las cantidades que se ofrecen al cliente: de 1 hasta lo que haya (con tope MAX_UNIDADES).
pub fn cantidades_ofrecidas(stock: i32) -> Vec<i32> {
    (1..=stock.min(MAX_UNIDADES)).collect()
}

// ¿La cantidad pedida es válida para ese stock?
pub fn cantidad_valida(cantidad: i32, stock: i32) -> bool {
    (1..=MAX_UNIDADES).contains(&cantidad) && cantidad <= stock
}

// El total de un apartado: precio unitario ("45.50") por cantidad, siempre con dos decimales ("91.00").
// Se calcula con enteros (céntimos) para no tener errores de redondeo. Si el precio no se entiende, devuelve "0.00".
pub fn total_texto(precio_unitario: &str, cantidad: i32) -> String {
    let (soles, centimos) = precio_unitario.split_once('.').unwrap_or((precio_unitario, "0"));
    let soles: i64 = soles.trim().parse().unwrap_or(0);
    // Solo se usan los dos primeros decimales ("5" cuenta como 50 céntimos).
    let centimos: i64 = format!("{:0<2}", centimos.trim()).chars().take(2).collect::<String>().parse().unwrap_or(0);
    let total = (soles * 100 + centimos) * cantidad.max(0) as i64;
    format!("{}.{:02}", total / 100, total % 100)
}

// ¿El apartado sigue pendiente (se puede entregar o cancelar)?
pub fn esta_pendiente(estado: &str) -> bool {
    estado == PENDIENTE
}

// ---------- Lo que usa el agente ----------

// El cliente aparta un producto. Primero se hace vencer lo que ya pasó de plazo (así su stock vuelve a estar
// disponible), luego se intenta apartar.
pub async fn apartar(
    pool: &PgPool,
    cliente: &str,
    producto: &str,
    cantidad: i32,
) -> Result<ResultadoApartado, sqlx::Error> {
    repositorio::vencer_apartados(pool).await?;
    repositorio::crear_apartado(pool, cliente, producto, cantidad, VALIDEZ_DIAS).await
}

// El cliente aparta todo su carrito de una vez. "items" son (código del producto, cantidad). Igual que en apartar(): primero se
// hace vencer lo que ya pasó de plazo, y después se aparta TODO o NADA.
pub async fn apartar_pedido(
    pool: &PgPool,
    cliente: &str,
    items: &[(String, i32)],
) -> Result<ResultadoPedido, sqlx::Error> {
    repositorio::vencer_apartados(pool).await?;
    repositorio::crear_pedido(pool, cliente, items, VALIDEZ_DIAS).await
}

// Los apartados de un pedido del cliente (nunca de otro).
pub async fn listar_pedido_de_cliente(
    pool: &PgPool,
    pedido: &str,
    cliente: &str,
) -> Result<Vec<Apartado>, sqlx::Error> {
    repositorio::listar_pedido_de_cliente(pool, pedido, cliente).await
}

// Los apartados de un cliente (los más recientes primero), con los vencidos ya actualizados.
pub async fn listar_de_cliente(
    pool: &PgPool,
    cliente: &str,
    limite: i64,
) -> Result<Vec<Apartado>, sqlx::Error> {
    repositorio::vencer_apartados(pool).await?;
    repositorio::listar_apartados_de_cliente(pool, cliente, limite).await
}

// Un apartado del cliente (nunca de otro: la consulta exige que sea suyo).
pub async fn obtener_de_cliente(
    pool: &PgPool,
    codigo: &str,
    cliente: &str,
) -> Result<Option<Apartado>, sqlx::Error> {
    repositorio::vencer_apartados(pool).await?;
    repositorio::obtener_apartado_de_cliente(pool, codigo, cliente).await
}

// El cliente cancela su propio apartado. Devuelve false si no es suyo o ya no está pendiente.
pub async fn cancelar_de_cliente(
    pool: &PgPool,
    codigo: &str,
    cliente: &str,
) -> Result<bool, sqlx::Error> {
    repositorio::vencer_apartados(pool).await?;
    repositorio::cancelar_apartado(pool, codigo, Some(cliente), None).await
}

// ---------- Lo que usa el panel (Vendedor o Administrador) ----------

// La lista de apartados del panel, con los vencidos ya actualizados.
pub async fn listar_para_panel(pool: &PgPool) -> Result<Vec<Apartado>, sqlx::Error> {
    repositorio::vencer_apartados(pool).await?;
    repositorio::listar_apartados_panel(pool).await
}

// El mensaje cuando un apartado ya no se puede entregar ni cancelar.
async fn motivo_no_pendiente(pool: &PgPool, codigo: &str) -> Result<Gestion, sqlx::Error> {
    Ok(match repositorio::obtener_apartado(pool, codigo).await? {
        None => Gestion::NoEncontrado,
        Some(apartado) => Gestion::Invalido(format!(
            "Ese apartado ya está «{}». Recarga la página.",
            apartado.estado_nombre
        )),
    })
}

pub async fn entregar(pool: &PgPool, usuario: &str, codigo: &str) -> Result<Gestion, sqlx::Error> {
    repositorio::vencer_apartados(pool).await?;
    if repositorio::entregar_apartado(pool, codigo, usuario).await? {
        Ok(Gestion::Hecho)
    } else {
        motivo_no_pendiente(pool, codigo).await
    }
}

pub async fn cancelar(pool: &PgPool, usuario: &str, codigo: &str) -> Result<Gestion, sqlx::Error> {
    repositorio::vencer_apartados(pool).await?;
    if repositorio::cancelar_apartado(pool, codigo, None, Some(usuario)).await? {
        Ok(Gestion::Hecho)
    } else {
        motivo_no_pendiente(pool, codigo).await
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn cantidades_segun_stock() {
        assert_eq!(cantidades_ofrecidas(0), Vec::<i32>::new());
        assert_eq!(cantidades_ofrecidas(2), vec![1, 2]);
        assert_eq!(cantidades_ofrecidas(50), vec![1, 2, 3, 4, 5]); // tope
    }

    #[test]
    fn total_con_dos_decimales() {
        assert_eq!(total_texto("45.50", 2), "91.00");
        assert_eq!(total_texto("2199.00", 1), "2199.00");
        assert_eq!(total_texto("19.99", 3), "59.97");
        assert_eq!(total_texto("10", 3), "30.00"); // sin decimales
        assert_eq!(total_texto("0.5", 2), "1.00"); // un solo decimal
        assert_eq!(total_texto("basura", 2), "0.00");
    }

    #[test]
    fn validar_cantidad() {
        assert!(cantidad_valida(1, 3));
        assert!(cantidad_valida(3, 3));
        assert!(!cantidad_valida(4, 3)); // más que el stock
        assert!(!cantidad_valida(0, 3));
        assert!(!cantidad_valida(6, 100)); // más que el tope
        assert!(!cantidad_valida(-1, 3));
    }

    #[test]
    fn solo_lo_pendiente_se_atiende() {
        assert!(esta_pendiente("PEN"));
        assert!(!esta_pendiente("ENT"));
        assert!(!esta_pendiente("CAN"));
        assert!(!esta_pendiente("VEN"));
    }
}
