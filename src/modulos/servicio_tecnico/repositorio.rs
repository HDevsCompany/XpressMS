// repositorio.rs (servicio_tecnico): consultas SQL de servicios y tipos de equipo.
// Ningún otro módulo escribe SQL sobre el esquema ServicioTecnico.
// Solo se leen los registros activos (Estado = 'A'): los "eliminados" siguen en la base pero no se muestran.

use sqlx::PgPool;

use super::modelo::{Servicio, TipoEquipo};

// "ServiciosPrecioBase::text" entrega el precio como texto ("60.00"), así no necesitamos una librería de decimales.

// Todos los servicios activos, ordenados por nombre.
pub async fn listar_servicios(pool: &PgPool) -> Result<Vec<Servicio>, sqlx::Error> {
    sqlx::query_as::<_, Servicio>(
        "select ServiciosCodigo as codigo, ServiciosNombre as nombre, ServiciosDescripcion as descripcion, \
                ServiciosPrecioBase::text as precio_base, ServiciosTiempoEstimado as tiempo_estimado \
         from ServicioTecnico.Servicios \
         where ServiciosEstado = 'A' \
         order by ServiciosNombre",
    )
    .fetch_all(pool)
    .await
}

// Un servicio activo por su código (None si no existe o fue eliminado).
pub async fn obtener_servicio(
    pool: &PgPool,
    codigo: &str,
) -> Result<Option<Servicio>, sqlx::Error> {
    sqlx::query_as::<_, Servicio>(
        "select ServiciosCodigo as codigo, ServiciosNombre as nombre, ServiciosDescripcion as descripcion, \
                ServiciosPrecioBase::text as precio_base, ServiciosTiempoEstimado as tiempo_estimado \
         from ServicioTecnico.Servicios \
         where ServiciosCodigo = $1 and ServiciosEstado = 'A'",
    )
    .bind(codigo)
    .fetch_optional(pool)
    .await
}

// Los tipos de equipo activos para los que sirve un servicio (tabla ServiciosEquipos), ordenados por nombre.
pub async fn listar_equipos_de_servicio(
    pool: &PgPool,
    servicio: &str,
) -> Result<Vec<TipoEquipo>, sqlx::Error> {
    sqlx::query_as::<_, TipoEquipo>(
        "select t.TiposEquipoCodigo as codigo, t.TiposEquipoNombre as nombre \
         from ServicioTecnico.ServiciosEquipos se \
         join ServicioTecnico.TiposEquipo t on t.TiposEquipoCodigo = se.ServiciosEquiposTipoEquipo \
         where se.ServiciosEquiposServicio = $1 and t.TiposEquipoEstado = 'A' \
         order by t.TiposEquipoNombre",
    )
    .bind(servicio)
    .fetch_all(pool)
    .await
}
