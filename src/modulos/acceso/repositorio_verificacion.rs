// repositorio_verificacion.rs: TODAS las consultas SQL de la verificación del celular (tabla
// Acceso.VerificacionesTelefono). Ningún otro archivo escribe SQL sobre esa tabla.
//
// Estado de cada código: 'P' pendiente (se puede usar), 'V' verificado (ya se usó), 'E' expirado.
// Los tiempos (vencimiento, ventana de 10 minutos) los calcula la propia base con now(), así no dependen
// del reloj del servidor ni de la zona horaria.

use sqlx::{FromRow, PgPool};

// Lo que hace falta de un código pendiente para poder compararlo.
#[derive(FromRow)]
pub struct CodigoPendiente {
    pub codigo: String,
    pub sal: String,
    pub hash: String,
    pub intentos: i16,
    pub vigente: bool, // true mientras no haya llegado su fecha de vencimiento
}

// Guarda un código nuevo (solo su huella) SI TODAVÍA NO se pasó el límite de pedidos.
// El conteo y el guardado ocurren en una sola instrucción: así no hay un "hueco" entre revisar y guardar.
//   - $1 código de la fila   - $2 conversación   - $3 celular E.164   - $4 sal   - $5 huella
//   - $6 red de origen       - $7 vigencia (min) - $8 ventana (min)   - $9 máximo de pedidos
// Devuelve true si se guardó, y false si el celular o la red ya pidieron el máximo en la ventana.
pub async fn crear_pendiente(
    pool: &PgPool,
    codigo: &str,
    conversacion: &str,
    telefono: &str,
    sal: &str,
    hash: &str,
    red: &str,
    vigencia_minutos: i32,
    ventana_minutos: i32,
    maximo: i64,
) -> Result<bool, sqlx::Error> {
    let resultado = sqlx::query(
        "insert into Acceso.VerificacionesTelefono \
         (VerificacionesTelefonoCodigo, VerificacionesTelefonoConversacion, VerificacionesTelefonoTelefono, \
          VerificacionesTelefonoSal, VerificacionesTelefonoHash, VerificacionesTelefonoRed, \
          VerificacionesTelefonoFechaExpiracion) \
         select $1::text, $2::text, $3::text, $4::text, $5::text, $6::text, now() + ($7::int * interval '1 minute') \
         where (select count(*) from Acceso.VerificacionesTelefono \
                where VerificacionesTelefonoTelefono = $3::text \
                  and VerificacionesTelefonoFechaCreacion > now() - ($8::int * interval '1 minute')) < $9::bigint \
           and (select count(*) from Acceso.VerificacionesTelefono \
                where VerificacionesTelefonoRed = $6::text \
                  and VerificacionesTelefonoFechaCreacion > now() - ($8::int * interval '1 minute')) < $9::bigint",
    )
    .bind(codigo)
    .bind(conversacion)
    .bind(telefono)
    .bind(sal)
    .bind(hash)
    .bind(red)
    .bind(vigencia_minutos)
    .bind(ventana_minutos)
    .bind(maximo)
    .execute(pool)
    .await?;

    Ok(resultado.rows_affected() == 1)
}

// Borra un código que NO llegó a enviarse (falló el SMS): así no le cuenta a la persona como un pedido gastado.
pub async fn anular(pool: &PgPool, codigo: &str) -> Result<(), sqlx::Error> {
    sqlx::query("delete from Acceso.VerificacionesTelefono where VerificacionesTelefonoCodigo = $1 and VerificacionesTelefonoEstado = 'P'")
        .bind(codigo)
        .execute(pool)
        .await?;
    Ok(())
}

// Cuando sale un código nuevo, los anteriores de la misma conversación y celular dejan de servir.
pub async fn expirar_anteriores(
    pool: &PgPool,
    conversacion: &str,
    telefono: &str,
    excepto: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "update Acceso.VerificacionesTelefono set VerificacionesTelefonoEstado = 'E' \
         where VerificacionesTelefonoConversacion = $1 and VerificacionesTelefonoTelefono = $2 \
           and VerificacionesTelefonoEstado = 'P' and VerificacionesTelefonoCodigo <> $3",
    )
    .bind(conversacion)
    .bind(telefono)
    .bind(excepto)
    .execute(pool)
    .await?;
    Ok(())
}

// El código pendiente más reciente de esta conversación para ese celular (o None si no hay).
pub async fn buscar_pendiente(
    pool: &PgPool,
    conversacion: &str,
    telefono: &str,
) -> Result<Option<CodigoPendiente>, sqlx::Error> {
    sqlx::query_as::<_, CodigoPendiente>(
        "select VerificacionesTelefonoCodigo as codigo, VerificacionesTelefonoSal as sal, \
                VerificacionesTelefonoHash as hash, VerificacionesTelefonoIntentos as intentos, \
                (VerificacionesTelefonoFechaExpiracion > now()) as vigente \
         from Acceso.VerificacionesTelefono \
         where VerificacionesTelefonoConversacion = $1 and VerificacionesTelefonoTelefono = $2 \
           and VerificacionesTelefonoEstado = 'P' \
         order by VerificacionesTelefonoFechaCreacion desc \
         limit 1",
    )
    .bind(conversacion)
    .bind(telefono)
    .fetch_optional(pool)
    .await
}

// Marca un código pendiente como expirado (venció o se agotaron los intentos).
pub async fn marcar_expirado(pool: &PgPool, codigo: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "update Acceso.VerificacionesTelefono set VerificacionesTelefonoEstado = 'E' \
         where VerificacionesTelefonoCodigo = $1 and VerificacionesTelefonoEstado = 'P'",
    )
    .bind(codigo)
    .execute(pool)
    .await?;
    Ok(())
}

// Suma un intento y devuelve cuántos lleva (None si el código ya no estaba pendiente).
// Se suma ANTES de comparar: así, aunque lleguen muchos intentos a la vez, todos quedan contados.
pub async fn sumar_intento(pool: &PgPool, codigo: &str) -> Result<Option<i16>, sqlx::Error> {
    sqlx::query_scalar::<_, i16>(
        "update Acceso.VerificacionesTelefono \
         set VerificacionesTelefonoIntentos = VerificacionesTelefonoIntentos + 1 \
         where VerificacionesTelefonoCodigo = $1 and VerificacionesTelefonoEstado = 'P' \
         returning VerificacionesTelefonoIntentos",
    )
    .bind(codigo)
    .fetch_optional(pool)
    .await
}

// Marca el código como usado. Devuelve false si en ese instante ya no estaba pendiente o ya venció
// (la condición está en el SQL, así un mismo código no puede validarse dos veces).
pub async fn marcar_verificado(pool: &PgPool, codigo: &str) -> Result<bool, sqlx::Error> {
    let resultado = sqlx::query(
        "update Acceso.VerificacionesTelefono \
         set VerificacionesTelefonoEstado = 'V', VerificacionesTelefonoFechaVerificacion = now() \
         where VerificacionesTelefonoCodigo = $1 and VerificacionesTelefonoEstado = 'P' \
           and VerificacionesTelefonoFechaExpiracion > now()",
    )
    .bind(codigo)
    .execute(pool)
    .await?;
    Ok(resultado.rows_affected() == 1)
}
