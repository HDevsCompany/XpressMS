// repositorio_ordenes.rs (servicio_tecnico): el SQL de las órdenes de servicio (citas y seguimiento).
// Aquí no se decide nada: los horarios están en agenda.rs y los cambios de estado en estados.rs.
//
// Las horas se guardan con zona horaria (timestamptz). Lima es siempre UTC-5, y todas las consultas
// convierten con 'America/Lima' para que "las 3 de la tarde" sea siempre las 3 de la tarde de Lima,
// sin importar dónde esté el servidor. Las fechas viajan como texto ("2026-11-09 15:00").

use sqlx::PgPool;

use super::estados::{CANCELADA, CITA};
use super::modelo::{
    DetalleOrden, EstadoOrden, NuevaOrden, RegistroHistorial, ResultadoCita, ResumenOrden,
};
use crate::modulos::comun::codigos::generar_codigo;

// ---------------------------------------------------------------------------------------------
// Agenda
// ---------------------------------------------------------------------------------------------

// Cuántas citas (no canceladas) hay ya en cada hora de un día. Devuelve parejas (hora, cantidad);
// las horas sin citas no aparecen. "fecha_iso" es "2026-11-09".
pub async fn contar_citas_por_hora(
    pool: &PgPool,
    fecha_iso: &str,
) -> Result<Vec<(i32, i64)>, sqlx::Error> {
    sqlx::query_as::<_, (i32, i64)>(
        "select extract(hour from OrdenesFechaCita at time zone 'America/Lima')::int, count(*) \
         from ServicioTecnico.Ordenes \
         where (OrdenesFechaCita at time zone 'America/Lima')::date = $1::text::date \
           and OrdenesEstadoOrden <> $2 \
         group by 1",
    )
    .bind(fecha_iso)
    .bind(CANCELADA)
    .fetch_all(pool)
    .await
}

// ---------------------------------------------------------------------------------------------
// Agendar y cancelar
// ---------------------------------------------------------------------------------------------

// Crea la orden en estado "Cita agendada" y su primera fila de historial, todo en una transacción
// (o se guardan las dos cosas o ninguna). Antes de contar el cupo se toma un "candado" por hora
// (pg_advisory_xact_lock): así, si dos personas eligen la última plaza al mismo tiempo, la segunda
// espera, ve el cupo lleno y recibe SinCupo en lugar de pasarse del límite.
pub async fn crear_orden(
    pool: &PgPool,
    nueva: &NuevaOrden,
    cupo_por_hora: i64,
) -> Result<ResultadoCita, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("select pg_advisory_xact_lock(hashtext($1))")
        .bind(&nueva.fecha_hora)
        .execute(&mut *tx)
        .await?;

    let codigo = generar_codigo();
    let creada: Option<(String,)> = sqlx::query_as(
        "insert into ServicioTecnico.Ordenes \
            (OrdenesCodigo, OrdenesCliente, OrdenesServicio, OrdenesTipoEquipo, OrdenesProblema, \
             OrdenesFechaCita, OrdenesEstadoOrden) \
         select $1, $2, $3, $4, $5, h.fecha, $7 \
         from (select (($6::text)::timestamp at time zone 'America/Lima') as fecha) h \
         where (select count(*) from ServicioTecnico.Ordenes \
                where OrdenesFechaCita = h.fecha and OrdenesEstadoOrden <> $8) < $9::bigint \
         returning OrdenesCodigo::text",
    )
    .bind(&codigo)
    .bind(&nueva.cliente)
    .bind(&nueva.servicio)
    .bind(&nueva.tipo_equipo)
    .bind(&nueva.problema)
    .bind(&nueva.fecha_hora)
    .bind(CITA)
    .bind(CANCELADA)
    .bind(cupo_por_hora)
    .fetch_optional(&mut *tx)
    .await?;

    if creada.is_none() {
        return Ok(ResultadoCita::SinCupo); // al salir sin commit, la transacción se deshace sola
    }

    registrar_historial(&mut *tx, &codigo, CITA, None, None).await?;
    tx.commit().await?;
    Ok(ResultadoCita::Creada(codigo))
}

// Una fila de historial dentro de una transacción en curso (la usan también los cambios de estado del panel).
pub async fn registrar_historial(
    tx: &mut sqlx::PgConnection,
    orden: &str,
    estado: &str,
    usuario: Option<&str>,
    nota: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "insert into ServicioTecnico.OrdenesHistorial \
            (OrdenesHistorialCodigo, OrdenesHistorialOrden, OrdenesHistorialEstadoOrden, \
             OrdenesHistorialUsuario, OrdenesHistorialNota) \
         values ($1, $2, $3, $4, $5)",
    )
    .bind(generar_codigo())
    .bind(orden)
    .bind(estado)
    .bind(usuario)
    .bind(nota)
    .execute(tx)
    .await?;
    Ok(())
}

// El cliente cancela su propia cita. Solo funciona si la orden es suya y sigue en "Cita agendada".
// Devuelve true si se canceló.
pub async fn cancelar_cita_de_cliente(
    pool: &PgPool,
    codigo: &str,
    cliente: &str,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let cancelada: Option<(String,)> = sqlx::query_as(
        "update ServicioTecnico.Ordenes \
         set OrdenesEstadoOrden = $3, OrdenesFechaActualizacion = now() \
         where OrdenesCodigo = $1 and OrdenesCliente = $2 and OrdenesEstadoOrden = $4 \
         returning OrdenesCodigo::text",
    )
    .bind(codigo)
    .bind(cliente)
    .bind(CANCELADA)
    .bind(CITA)
    .fetch_optional(&mut *tx)
    .await?;

    if cancelada.is_none() {
        return Ok(false);
    }
    registrar_historial(&mut *tx, codigo, CANCELADA, None, Some("Cancelada por el cliente")).await?;
    tx.commit().await?;
    Ok(true)
}

// ---------------------------------------------------------------------------------------------
// Consultas del cliente (siempre limitadas a SUS órdenes)
// ---------------------------------------------------------------------------------------------

// Las últimas órdenes de un cliente, la más reciente primero.
pub async fn listar_ordenes_de_cliente(
    pool: &PgPool,
    cliente: &str,
    limite: i64,
) -> Result<Vec<ResumenOrden>, sqlx::Error> {
    sqlx::query_as::<_, ResumenOrden>(
        "select o.OrdenesCodigo::text as codigo, s.ServiciosNombre as servicio, \
                e.EstadosOrdenNombre as estado_nombre, \
                to_char(o.OrdenesFechaCita at time zone 'America/Lima', 'YYYY-MM-DD HH24:MI') as fecha_cita \
         from ServicioTecnico.Ordenes o \
         join ServicioTecnico.Servicios s on s.ServiciosCodigo = o.OrdenesServicio \
         join ServicioTecnico.EstadosOrden e on e.EstadosOrdenCodigo = o.OrdenesEstadoOrden \
         where o.OrdenesCliente = $1 \
         order by o.OrdenesFechaRegistro desc, o.OrdenesCodigo \
         limit $2",
    )
    .bind(cliente)
    .bind(limite)
    .fetch_all(pool)
    .await
}

// Una orden por su código, solo si pertenece a ese cliente (así nadie ve órdenes ajenas adivinando códigos).
pub async fn obtener_orden_de_cliente(
    pool: &PgPool,
    codigo: &str,
    cliente: &str,
) -> Result<Option<DetalleOrden>, sqlx::Error> {
    sqlx::query_as::<_, DetalleOrden>(
        "select o.OrdenesCodigo::text as codigo, s.ServiciosNombre as servicio, \
                t.TiposEquipoNombre as equipo, o.OrdenesProblema as problema, \
                to_char(o.OrdenesFechaCita at time zone 'America/Lima', 'YYYY-MM-DD HH24:MI') as fecha_cita, \
                o.OrdenesEstadoOrden::text as estado_codigo, e.EstadosOrdenNombre as estado_nombre \
         from ServicioTecnico.Ordenes o \
         join ServicioTecnico.Servicios s on s.ServiciosCodigo = o.OrdenesServicio \
         join ServicioTecnico.EstadosOrden e on e.EstadosOrdenCodigo = o.OrdenesEstadoOrden \
         left join ServicioTecnico.TiposEquipo t on t.TiposEquipoCodigo = o.OrdenesTipoEquipo \
         where o.OrdenesCodigo = $1 and o.OrdenesCliente = $2",
    )
    .bind(codigo)
    .bind(cliente)
    .fetch_optional(pool)
    .await
}

// Los cambios de estado de una orden, del más antiguo al más reciente, con la fecha en hora de Lima.
pub async fn historial_de_orden(
    pool: &PgPool,
    orden: &str,
) -> Result<Vec<RegistroHistorial>, sqlx::Error> {
    sqlx::query_as::<_, RegistroHistorial>(
        "select OrdenesHistorialEstadoOrden::text as estado, \
                to_char(OrdenesHistorialFecha at time zone 'America/Lima', 'DD/MM HH24:MI') as fecha \
         from ServicioTecnico.OrdenesHistorial \
         where OrdenesHistorialOrden = $1 \
         order by OrdenesHistorialFecha, OrdenesHistorialCodigo",
    )
    .bind(orden)
    .fetch_all(pool)
    .await
}

// El catálogo de estados, en el orden en que ocurren.
pub async fn listar_estados(pool: &PgPool) -> Result<Vec<EstadoOrden>, sqlx::Error> {
    sqlx::query_as::<_, EstadoOrden>(
        "select EstadosOrdenCodigo::text as codigo, EstadosOrdenNombre as nombre \
         from ServicioTecnico.EstadosOrden \
         where EstadosOrdenEstado = 'A' \
         order by EstadosOrdenSecuencia",
    )
    .fetch_all(pool)
    .await
}
