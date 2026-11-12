// repositorio_gestion.rs (servicio_tecnico): el SQL que ESCRIBE (altas, cambios y bajas) y las listas del panel.
// repositorio.rs sigue siendo el que lee para el agente. Aquí no se decide nada: las reglas están en gestion.rs.
//
// "Eliminar" nunca borra la fila: pone Estado = 'E'. Así las citas y órdenes futuras conservan su historial.
// Los nombres repetidos los frena la base con índices únicos entre registros activos (05_servicio_tecnico_nombres.sql);
// aquí se reconoce ese error y se convierte en Resultado::NombreRepetido.

use std::collections::HashMap;

use sqlx::error::ErrorKind;
use sqlx::{PgConnection, PgPool};

use super::modelo::{Resultado, ServicioPanel, TipoEquipo, TipoEquipoPanel};
use super::repositorio;
use super::validaciones::DatosServicio;

// ¿El error de la base es "ya existe un registro con ese valor único"?
fn es_nombre_repetido(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|e| matches!(e.kind(), ErrorKind::UniqueViolation))
}

// ---------------------------------------------------------------------------------------------
// Listas para el panel
// ---------------------------------------------------------------------------------------------

// Los tipos de equipo activos con cuántos servicios activos los usan.
pub async fn listar_equipos(pool: &PgPool) -> Result<Vec<TipoEquipoPanel>, sqlx::Error> {
    sqlx::query_as::<_, TipoEquipoPanel>(
        "select t.TiposEquipoCodigo as codigo, t.TiposEquipoNombre as nombre, \
                (select count(*) from ServicioTecnico.ServiciosEquipos se \
                 join ServicioTecnico.Servicios s on s.ServiciosCodigo = se.ServiciosEquiposServicio \
                 where se.ServiciosEquiposTipoEquipo = t.TiposEquipoCodigo and s.ServiciosEstado = 'A')::bigint as servicios \
         from ServicioTecnico.TiposEquipo t \
         where t.TiposEquipoEstado = 'A' \
         order by t.TiposEquipoNombre",
    )
    .fetch_all(pool)
    .await
}

// Los servicios activos, cada uno con sus tipos de equipo.
pub async fn listar_servicios(pool: &PgPool) -> Result<Vec<ServicioPanel>, sqlx::Error> {
    let servicios = repositorio::listar_servicios(pool).await?;

    // Todas las parejas (servicio, equipo) en una sola consulta; luego se agrupan por servicio.
    let parejas = sqlx::query_as::<_, (String, String, String)>(
        "select se.ServiciosEquiposServicio, t.TiposEquipoCodigo, t.TiposEquipoNombre \
         from ServicioTecnico.ServiciosEquipos se \
         join ServicioTecnico.TiposEquipo t on t.TiposEquipoCodigo = se.ServiciosEquiposTipoEquipo \
         where t.TiposEquipoEstado = 'A' \
         order by t.TiposEquipoNombre",
    )
    .fetch_all(pool)
    .await?;
    let mut por_servicio: HashMap<String, Vec<TipoEquipo>> = HashMap::new();
    for (servicio, codigo, nombre) in parejas {
        por_servicio
            .entry(servicio)
            .or_default()
            .push(TipoEquipo { codigo, nombre });
    }

    // Los servicios que tienen imagen, con un número que cambia cada vez que se cambia la imagen: así el navegador no se queda con
    // la anterior guardada.
    let con_imagen = sqlx::query_as::<_, (String, i64)>(
        "select ServiciosCodigo, (extract(epoch from coalesce(ServiciosFechaActualizacion, now())) * 1000)::bigint \
         from ServicioTecnico.Servicios \
         where ServiciosEstado = 'A' and ServiciosImagen is not null",
    )
    .fetch_all(pool)
    .await?;
    let mut imagenes: HashMap<String, i64> = con_imagen.into_iter().collect();

    Ok(servicios
        .into_iter()
        .map(|servicio| {
            let equipos = por_servicio.remove(&servicio.codigo).unwrap_or_default();
            let imagen = imagenes
                .remove(&servicio.codigo)
                .map(|version| format!("/publico/servicios/{}/imagen?v={}", servicio.codigo, version));
            ServicioPanel { servicio, equipos, imagen }
        })
        .collect())
}

// ---------------------------------------------------------------------------------------------
// Tipos de equipo
// ---------------------------------------------------------------------------------------------

pub async fn crear_tipo_equipo(
    pool: &PgPool,
    codigo: &str,
    nombre: &str,
) -> Result<Resultado, sqlx::Error> {
    let resultado = sqlx::query(
        "insert into ServicioTecnico.TiposEquipo (TiposEquipoCodigo, TiposEquipoNombre) values ($1, $2)",
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

pub async fn renombrar_tipo_equipo(
    pool: &PgPool,
    codigo: &str,
    nombre: &str,
) -> Result<Resultado, sqlx::Error> {
    let resultado = sqlx::query(
        "update ServicioTecnico.TiposEquipo set TiposEquipoNombre = $2 \
         where TiposEquipoCodigo = $1 and TiposEquipoEstado = 'A'",
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

// Devuelve false si no existía (o ya estaba eliminado).
pub async fn eliminar_tipo_equipo(pool: &PgPool, codigo: &str) -> Result<bool, sqlx::Error> {
    let filas = sqlx::query(
        "update ServicioTecnico.TiposEquipo set TiposEquipoEstado = 'E' \
         where TiposEquipoCodigo = $1 and TiposEquipoEstado = 'A'",
    )
    .bind(codigo)
    .execute(pool)
    .await?;
    Ok(filas.rows_affected() > 0)
}

// ---------------------------------------------------------------------------------------------
// Servicios
// ---------------------------------------------------------------------------------------------

// Guarda las parejas (servicio, equipo). Solo acepta tipos de equipo activos; devuelve false si
// alguno de los códigos enviados no corresponde a un tipo de equipo activo.
// "$4::text::numeric" y similares: el precio viaja como texto y la base lo convierte a número.
async fn guardar_equipos(
    conexion: &mut PgConnection,
    servicio: &str,
    equipos: &[String],
) -> Result<bool, sqlx::Error> {
    let filas = sqlx::query(
        "insert into ServicioTecnico.ServiciosEquipos (ServiciosEquiposServicio, ServiciosEquiposTipoEquipo) \
         select $1::text, TiposEquipoCodigo from ServicioTecnico.TiposEquipo \
         where TiposEquipoCodigo = any($2::text[]) and TiposEquipoEstado = 'A'",
    )
    .bind(servicio)
    .bind(equipos)
    .execute(&mut *conexion)
    .await?;
    Ok(filas.rows_affected() as usize == equipos.len())
}

// Crea un servicio y sus equipos, todo o nada (una transacción).
pub async fn crear_servicio(
    pool: &PgPool,
    codigo: &str,
    datos: &DatosServicio,
) -> Result<Resultado, sqlx::Error> {
    let mut transaccion = pool.begin().await?;
    let insertado = sqlx::query(
        "insert into ServicioTecnico.Servicios \
                (ServiciosCodigo, ServiciosNombre, ServiciosDescripcion, ServiciosPrecioBase, ServiciosTiempoEstimado, ServiciosAvisoPrecio) \
         values ($1, $2, $3, $4::text::numeric, $5, coalesce($6::boolean, false))",
    )
    .bind(codigo)
    .bind(&datos.nombre)
    .bind(&datos.descripcion)
    .bind(datos.precio_base.as_deref())
    .bind(datos.tiempo_estimado.as_deref())
    .bind(datos.aviso_precio)
    .execute(&mut *transaccion)
    .await;
    match insertado {
        Ok(_) => {}
        Err(error) if es_nombre_repetido(&error) => return Ok(Resultado::NombreRepetido),
        Err(error) => return Err(error),
    }
    if !guardar_equipos(&mut transaccion, codigo, &datos.equipos).await? {
        return Ok(Resultado::EquipoInvalido); // al salir sin "commit", todo se deshace
    }
    transaccion.commit().await?;
    Ok(Resultado::Hecho)
}

// Cambia los datos de un servicio y reemplaza sus equipos, todo o nada.
pub async fn actualizar_servicio(
    pool: &PgPool,
    codigo: &str,
    datos: &DatosServicio,
) -> Result<Resultado, sqlx::Error> {
    let mut transaccion = pool.begin().await?;
    let actualizado = sqlx::query(
        "update ServicioTecnico.Servicios set ServiciosNombre = $2, ServiciosDescripcion = $3, \
                ServiciosPrecioBase = $4::text::numeric, ServiciosTiempoEstimado = $5, \
                ServiciosAvisoPrecio = coalesce($6::boolean, ServiciosAvisoPrecio), \
                ServiciosFechaActualizacion = now() \
         where ServiciosCodigo = $1 and ServiciosEstado = 'A'",
    )
    .bind(codigo)
    .bind(&datos.nombre)
    .bind(&datos.descripcion)
    .bind(datos.precio_base.as_deref())
    .bind(datos.tiempo_estimado.as_deref())
    .bind(datos.aviso_precio)
    .execute(&mut *transaccion)
    .await;
    match actualizado {
        Ok(filas) if filas.rows_affected() == 0 => return Ok(Resultado::NoEncontrado),
        Ok(_) => {}
        Err(error) if es_nombre_repetido(&error) => return Ok(Resultado::NombreRepetido),
        Err(error) => return Err(error),
    }
    sqlx::query("delete from ServicioTecnico.ServiciosEquipos where ServiciosEquiposServicio = $1")
        .bind(codigo)
        .execute(&mut *transaccion)
        .await?;
    if !guardar_equipos(&mut transaccion, codigo, &datos.equipos).await? {
        return Ok(Resultado::EquipoInvalido);
    }
    transaccion.commit().await?;
    Ok(Resultado::Hecho)
}

// Devuelve false si no existía (o ya estaba eliminado).
pub async fn eliminar_servicio(pool: &PgPool, codigo: &str) -> Result<bool, sqlx::Error> {
    let filas = sqlx::query(
        "update ServicioTecnico.Servicios set ServiciosEstado = 'E', ServiciosFechaActualizacion = now() \
         where ServiciosCodigo = $1 and ServiciosEstado = 'A'",
    )
    .bind(codigo)
    .execute(pool)
    .await?;
    Ok(filas.rows_affected() > 0)
}

// ---------------------------------------------------------------------------------------------
// Imagen del servicio (se guarda en la propia base; ver 13_servicios_imagen_y_aviso.sql)
// ---------------------------------------------------------------------------------------------

// Guarda (o reemplaza) la imagen de un servicio. Devuelve false si el servicio no existe o fue eliminado.
pub async fn guardar_imagen(pool: &PgPool, codigo: &str, tipo: &str, bytes: &[u8]) -> Result<bool, sqlx::Error> {
    let filas = sqlx::query(
        "update ServicioTecnico.Servicios set ServiciosImagen = $2, ServiciosImagenTipo = $3, ServiciosFechaActualizacion = now() \
         where ServiciosCodigo = $1 and ServiciosEstado = 'A'",
    )
    .bind(codigo)
    .bind(bytes)
    .bind(tipo)
    .execute(pool)
    .await?;
    Ok(filas.rows_affected() > 0)
}

// Le quita la imagen a un servicio. Devuelve false si el servicio no existe o fue eliminado.
pub async fn quitar_imagen(pool: &PgPool, codigo: &str) -> Result<bool, sqlx::Error> {
    let filas = sqlx::query(
        "update ServicioTecnico.Servicios set ServiciosImagen = null, ServiciosImagenTipo = null, ServiciosFechaActualizacion = now() \
         where ServiciosCodigo = $1 and ServiciosEstado = 'A'",
    )
    .bind(codigo)
    .execute(pool)
    .await?;
    Ok(filas.rows_affected() > 0)
}

// La imagen de un servicio activo: (tipo, bytes). None si no tiene imagen o el servicio no existe.
pub async fn leer_imagen(pool: &PgPool, codigo: &str) -> Result<Option<(String, Vec<u8>)>, sqlx::Error> {
    sqlx::query_as::<_, (String, Vec<u8>)>(
        "select ServiciosImagenTipo, ServiciosImagen from ServicioTecnico.Servicios \
         where ServiciosCodigo = $1 and ServiciosEstado = 'A' and ServiciosImagen is not null and ServiciosImagenTipo is not null",
    )
    .bind(codigo)
    .fetch_optional(pool)
    .await
}
