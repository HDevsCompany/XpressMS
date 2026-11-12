// rutas_ordenes.rs (servicio_tecnico): las "puertas" con las que el panel atiende las órdenes de servicio.
// Solo reciben JSON, comprueban el rol, llaman a ordenes.rs y devuelven JSON. No contienen SQL ni reglas.
// El backend decide: aunque el navegador mostrara un botón de más, aquí se vuelve a comprobar todo.
//
//   GET /panel/api/ordenes                 lista de órdenes (Administrador: todas; Técnico: solo las suyas)
//   GET /panel/api/ordenes/{codigo}        una orden con su historial y los estados a los que puede pasar
//   PUT /panel/api/ordenes/{codigo}/tecnico  asigna técnico { tecnico: código o null }   (solo Administrador)
//   PUT /panel/api/ordenes/{codigo}/estado   cambia el estado { estado, nota? }          (Administrador o Técnico dueño)
//   GET /panel/api/tecnicos                técnicos disponibles para asignar                (solo Administrador)

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use sqlx::PgPool;

use super::modelo::{EntradaEstado, EntradaTecnico};
use super::ordenes::{self, Alcance};
use super::repositorio_ordenes_panel as repositorio;
use super::rutas::{con_mensaje, error_interno, responder};
use crate::modulos::acceso::personal::modelo::UsuarioActual;
use crate::modulos::acceso::personal::roles::{ROL_ADMIN, ROL_TECNICO};
use crate::modulos::acceso::personal::sesion::{exigir_rol, PersonalAutenticado};

pub fn rutas() -> Router<PgPool> {
    Router::new()
        .route("/panel/api/ordenes", get(listar))
        .route("/panel/api/ordenes/{codigo}", get(detalle))
        .route("/panel/api/ordenes/{codigo}/tecnico", put(asignar_tecnico))
        .route("/panel/api/ordenes/{codigo}/estado", put(cambiar_estado))
        .route("/panel/api/tecnicos", get(listar_tecnicos))
}

// Administrador y Técnico pueden atender órdenes; el Vendedor no. Devuelve qué órdenes le corresponden.
fn alcance_del_personal(usuario: &UsuarioActual) -> Result<Alcance, Response> {
    exigir_rol(usuario, &[ROL_ADMIN, ROL_TECNICO]).map_err(|codigo| codigo.into_response())?;
    ordenes::alcance_de(usuario).ok_or_else(|| StatusCode::FORBIDDEN.into_response())
}

fn solo_admin(usuario: &UsuarioActual) -> Result<(), Response> {
    exigir_rol(usuario, &[ROL_ADMIN]).map_err(|codigo| codigo.into_response())
}

async fn listar(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
) -> Response {
    let alcance = match alcance_del_personal(&usuario) {
        Ok(alcance) => alcance,
        Err(rechazo) => return rechazo,
    };
    match ordenes::listar(&pool, &alcance).await {
        Ok(lista) => ([(header::CACHE_CONTROL, "no-store")], Json(lista)).into_response(),
        Err(error) => error_interno(error),
    }
}

async fn detalle(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
) -> Response {
    let alcance = match alcance_del_personal(&usuario) {
        Ok(alcance) => alcance,
        Err(rechazo) => return rechazo,
    };
    match ordenes::detalle(&pool, &alcance, &codigo).await {
        Ok(Some(detalle)) => ([(header::CACHE_CONTROL, "no-store")], Json(detalle)).into_response(),
        Ok(None) => con_mensaje(
            StatusCode::NOT_FOUND,
            "Esa orden no existe o no está asignada a ti.".to_string(),
        ),
        Err(error) => error_interno(error),
    }
}

async fn asignar_tecnico(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
    Json(entrada): Json<EntradaTecnico>,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    responder(
        ordenes::asignar_tecnico(&pool, &codigo, entrada).await,
        StatusCode::NO_CONTENT,
    )
}

async fn cambiar_estado(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
    Path(codigo): Path<String>,
    Json(entrada): Json<EntradaEstado>,
) -> Response {
    let alcance = match alcance_del_personal(&usuario) {
        Ok(alcance) => alcance,
        Err(rechazo) => return rechazo,
    };
    responder(
        ordenes::cambiar_estado(&pool, &usuario, &alcance, &codigo, entrada).await,
        StatusCode::NO_CONTENT,
    )
}

async fn listar_tecnicos(
    State(pool): State<PgPool>,
    PersonalAutenticado(usuario): PersonalAutenticado,
) -> Response {
    if let Err(rechazo) = solo_admin(&usuario) {
        return rechazo;
    }
    match repositorio::listar_tecnicos(&pool).await {
        Ok(lista) => ([(header::CACHE_CONTROL, "no-store")], Json(lista)).into_response(),
        Err(error) => error_interno(error),
    }
}
