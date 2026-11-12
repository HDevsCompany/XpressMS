// repositorio_ordenes_panel.rs (servicio_tecnico): el SQL que usa el panel del personal para ver y atender órdenes.
// Aquí no se decide nada: quién puede hacer qué y los cambios de estado permitidos están en ordenes.rs y estados.rs.
// Las horas salen convertidas a Lima ('America/Lima'), igual que en repositorio_ordenes.rs.

use sqlx::PgPool;

use super::estados::{CANCELADA, ENTREGADO};
use super::modelo::{HistorialPanel, OrdenPanel, TecnicoOpcion};
use super::repositorio_ordenes::registrar_historial;
use crate::modulos::acceso::personal::roles::ROL_TECNICO;

// El SELECT común de una orden del panel (con cliente, servicio, equipo, estado y técnico).
// Es una macro (y no un texto armado con format!) porque sqlx 0.9 solo acepta consultas escritas
// completas en el código: así nunca se arma SQL con datos de la persona.
macro_rules! select_orden {
    () => {
        "\
    select o.OrdenesCodigo::text as codigo, c.ClientesNombreCompleto::text as cliente, \
           c.ClientesTelefono::text as telefono, s.ServiciosNombre as servicio, \
           t.TiposEquipoNombre as equipo, o.OrdenesProblema as problema, \
           to_char(o.OrdenesFechaCita at time zone 'America/Lima', 'YYYY-MM-DD HH24:MI') as fecha_cita, \
           o.OrdenesEstadoOrden::text as estado_codigo, e.EstadosOrdenNombre as estado_nombre, \
           o.OrdenesTecnico::text as tecnico_codigo, \
           (emp.EmpleadosNombres || ' ' || emp.EmpleadosPaterno) as tecnico_nombre \
    from ServicioTecnico.Ordenes o \
    join Acceso.Clientes c on c.ClientesCodigo = o.OrdenesCliente \
    join ServicioTecnico.Servicios s on s.ServiciosCodigo = o.OrdenesServicio \
    join ServicioTecnico.EstadosOrden e on e.EstadosOrdenCodigo = o.OrdenesEstadoOrden \
    left join ServicioTecnico.TiposEquipo t on t.TiposEquipoCodigo = o.OrdenesTipoEquipo \
    left join Acceso.Usuarios u on u.UsuariosCodigo = o.OrdenesTecnico \
    left join Acceso.Empleados emp on emp.EmpleadosCodigo = u.UsuariosEmpleado "
    };
}

// Las órdenes para el panel: primero las que siguen en curso (la cita más próxima arriba) y después las
// terminadas (la más reciente arriba). Con "tecnico" solo salen las asignadas a esa persona; con None, todas.
// Se limita a 300 para que la pantalla no se haga pesada.
pub async fn listar_ordenes(
    pool: &PgPool,
    tecnico: Option<&str>,
) -> Result<Vec<OrdenPanel>, sqlx::Error> {
    sqlx::query_as::<_, OrdenPanel>(concat!(
        select_orden!(),
        " where ($1::text is null or o.OrdenesTecnico::text = $1::text) \
         order by (o.OrdenesEstadoOrden in ($2, $3)), \
                  (case when o.OrdenesEstadoOrden in ($2, $3) then -1 else 1 end) \
                  * extract(epoch from o.OrdenesFechaCita) \
         limit 300"
    ))
    .bind(tecnico)
    .bind(ENTREGADO)
    .bind(CANCELADA)
    .fetch_all(pool)
    .await
}

// Una orden por su código (None si no existe). Quién puede verla lo decide ordenes.rs.
pub async fn obtener_orden(pool: &PgPool, codigo: &str) -> Result<Option<OrdenPanel>, sqlx::Error> {
    sqlx::query_as::<_, OrdenPanel>(concat!(
        select_orden!(),
        " where o.OrdenesCodigo::text = $1::text"
    ))
    .bind(codigo)
    .fetch_optional(pool)
    .await
}

// Los cambios de estado de una orden, del más antiguo al más reciente, con quién los hizo.
pub async fn historial_de_orden(
    pool: &PgPool,
    orden: &str,
) -> Result<Vec<HistorialPanel>, sqlx::Error> {
    sqlx::query_as::<_, HistorialPanel>(
        "select e.EstadosOrdenNombre as estado, \
                to_char(h.OrdenesHistorialFecha at time zone 'America/Lima', 'DD/MM/YYYY HH24:MI') as fecha, \
                (emp.EmpleadosNombres || ' ' || emp.EmpleadosPaterno) as usuario, \
                h.OrdenesHistorialNota as nota \
         from ServicioTecnico.OrdenesHistorial h \
         join ServicioTecnico.EstadosOrden e on e.EstadosOrdenCodigo = h.OrdenesHistorialEstadoOrden \
         left join Acceso.Usuarios u on u.UsuariosCodigo = h.OrdenesHistorialUsuario \
         left join Acceso.Empleados emp on emp.EmpleadosCodigo = u.UsuariosEmpleado \
         where h.OrdenesHistorialOrden = $1 \
         order by h.OrdenesHistorialFecha, h.OrdenesHistorialCodigo",
    )
    .bind(orden)
    .fetch_all(pool)
    .await
}

// Las personas activas con rol de Técnico (a quienes se puede asignar una orden).
pub async fn listar_tecnicos(pool: &PgPool) -> Result<Vec<TecnicoOpcion>, sqlx::Error> {
    sqlx::query_as::<_, TecnicoOpcion>(
        "select u.UsuariosCodigo::text as codigo, \
                (e.EmpleadosNombres || ' ' || e.EmpleadosPaterno) as nombre \
         from Acceso.Usuarios u \
         join Acceso.Empleados e on e.EmpleadosCodigo = u.UsuariosEmpleado \
         join Acceso.UsuariosRoles r on r.UsuariosRolesUsuario = u.UsuariosCodigo \
         where r.UsuariosRolesRol = $1 and u.UsuariosEstado = 'A' and e.EmpleadosEstado = 'A' \
         order by e.EmpleadosPaterno, e.EmpleadosNombres",
    )
    .bind(ROL_TECNICO)
    .fetch_all(pool)
    .await
}

// Asigna (o quita, con None) el técnico de una orden que sigue en curso. Devuelve false si la orden no
// existe o ya terminó (entregada o cancelada).
pub async fn asignar_tecnico(
    pool: &PgPool,
    codigo: &str,
    tecnico: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let resultado = sqlx::query(
        "update ServicioTecnico.Ordenes \
         set OrdenesTecnico = $2, OrdenesFechaActualizacion = now() \
         where OrdenesCodigo = $1 and OrdenesEstadoOrden not in ($3, $4)",
    )
    .bind(codigo)
    .bind(tecnico)
    .bind(ENTREGADO)
    .bind(CANCELADA)
    .execute(pool)
    .await?;
    Ok(resultado.rows_affected() > 0)
}

// Pasa la orden de "esperado" a "nuevo" y deja constancia en el historial (quién y cuándo), todo en una
// transacción. Si la orden ya no estaba en "esperado" (alguien más la cambió antes), no toca nada y devuelve false.
pub async fn cambiar_estado(
    pool: &PgPool,
    codigo: &str,
    esperado: &str,
    nuevo: &str,
    usuario: &str,
    nota: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let cambiada: Option<(String,)> = sqlx::query_as(
        "update ServicioTecnico.Ordenes \
         set OrdenesEstadoOrden = $3, OrdenesFechaActualizacion = now() \
         where OrdenesCodigo = $1 and OrdenesEstadoOrden = $2 \
         returning OrdenesCodigo::text",
    )
    .bind(codigo)
    .bind(esperado)
    .bind(nuevo)
    .fetch_optional(&mut *tx)
    .await?;

    if cambiada.is_none() {
        return Ok(false); // sin commit: la transacción se deshace sola
    }
    registrar_historial(&mut *tx, codigo, nuevo, Some(usuario), nota).await?;
    tx.commit().await?;
    Ok(true)
}
