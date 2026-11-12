// ordenes.rs (servicio_tecnico): las reglas con las que el personal atiende las órdenes desde el panel.
// Aquí se decide: quién ve qué orden, cuándo se puede asignar un técnico y qué cambios de estado se aceptan.
// El SQL vive en repositorio_ordenes_panel.rs y las transiciones permitidas en estados.rs.
// Qué ROLES pueden entrar a estas rutas lo comprueba rutas_ordenes.rs antes de llegar aquí.
//
//   Administrador: ve todas las órdenes, asigna técnicos y cambia estados de cualquiera.
//   Técnico:       ve y atiende solo las órdenes asignadas a él (las demás "no existen" para él).

use sqlx::PgPool;

use super::estados::{self, CANCELADA};
use super::gestion::Gestion;
use super::modelo::{DetalleOrdenPanel, EntradaEstado, EntradaTecnico, OrdenPanel};
use super::repositorio_ordenes as repositorio_citas;
use super::repositorio_ordenes_panel as repositorio;
use crate::modulos::acceso::personal::modelo::UsuarioActual;
use crate::modulos::acceso::personal::roles::{tiene_algun_rol, ROL_ADMIN, ROL_TECNICO};

const MAX_NOTA: usize = 300;

// Qué órdenes puede ver y atender la persona.
pub enum Alcance {
    Todas,             // Administrador
    Propias(String),   // Técnico: el código de su usuario
}

// El alcance de una persona según sus roles (None si no tiene ninguno que atienda órdenes).
// Si es Administrador y Técnico a la vez, manda el de Administrador.
pub fn alcance_de(usuario: &UsuarioActual) -> Option<Alcance> {
    if tiene_algun_rol(&usuario.roles, &[ROL_ADMIN]) {
        Some(Alcance::Todas)
    } else if tiene_algun_rol(&usuario.roles, &[ROL_TECNICO]) {
        Some(Alcance::Propias(usuario.codigo.clone()))
    } else {
        None
    }
}

impl Alcance {
    // Para filtrar la lista: el técnico dueño de las órdenes (None = todas).
    fn tecnico(&self) -> Option<&str> {
        match self {
            Alcance::Todas => None,
            Alcance::Propias(codigo) => Some(codigo.as_str()),
        }
    }

    // ¿Puede ver esta orden?
    fn permite(&self, orden: &OrdenPanel) -> bool {
        match self {
            Alcance::Todas => true,
            Alcance::Propias(codigo) => orden.tecnico_codigo.as_deref() == Some(codigo.as_str()),
        }
    }
}

// ---------- Reglas puras (se prueban solas) ----------

// ¿Se puede pasar la orden a ese estado? Devuelve el mensaje para la persona si no.
fn revisar_cambio(
    estado_actual: &str,
    nombre_actual: &str,
    tiene_tecnico: bool,
    nuevo: &str,
) -> Result<(), String> {
    if !estados::puede_pasar(estado_actual, nuevo) {
        return Err(format!(
            "La orden está en «{}» y no puede pasar a ese estado.",
            nombre_actual
        ));
    }
    // Cancelar siempre se puede; avanzar exige que alguien se haga cargo.
    if nuevo != CANCELADA && !tiene_tecnico {
        return Err("Antes de avanzar la orden, asígnale un técnico.".to_string());
    }
    Ok(())
}

// Limpia la nota opcional: sin espacios repetidos, vacía = sin nota, máximo 300 caracteres.
fn revisar_nota(nota: Option<String>) -> Result<Option<String>, String> {
    let Some(texto) = nota else {
        return Ok(None);
    };
    let limpia = texto.split_whitespace().collect::<Vec<_>>().join(" ");
    if limpia.is_empty() {
        return Ok(None);
    }
    if limpia.chars().count() > MAX_NOTA {
        return Err("La nota es muy larga: usa máximo 300 caracteres.".to_string());
    }
    Ok(Some(limpia))
}

// ---------- Consultas ----------

pub async fn listar(pool: &PgPool, alcance: &Alcance) -> Result<Vec<OrdenPanel>, sqlx::Error> {
    repositorio::listar_ordenes(pool, alcance.tecnico()).await
}

// La orden con su historial y los estados a los que puede pasar. None si no existe o no es de esta persona.
pub async fn detalle(
    pool: &PgPool,
    alcance: &Alcance,
    codigo: &str,
) -> Result<Option<DetalleOrdenPanel>, sqlx::Error> {
    let Some(orden) = repositorio::obtener_orden(pool, codigo).await? else {
        return Ok(None);
    };
    if !alcance.permite(&orden) {
        return Ok(None);
    }
    let historial = repositorio::historial_de_orden(pool, &orden.codigo).await?;
    let siguientes = repositorio_citas::listar_estados(pool)
        .await?
        .into_iter()
        .filter(|estado| estados::puede_pasar(&orden.estado_codigo, &estado.codigo))
        .collect();
    Ok(Some(DetalleOrdenPanel { orden, historial, siguientes }))
}

// ---------- Cambios ----------

// Asigna el técnico (o lo quita). Solo el Administrador llega aquí (lo comprueba la ruta).
pub async fn asignar_tecnico(
    pool: &PgPool,
    codigo: &str,
    entrada: EntradaTecnico,
) -> Result<Gestion, sqlx::Error> {
    let Some(orden) = repositorio::obtener_orden(pool, codigo).await? else {
        return Ok(Gestion::NoEncontrado);
    };
    let terminada = estados::siguientes_permitidos(&orden.estado_codigo).is_empty();
    if terminada {
        return Ok(Gestion::Invalido(
            "Una orden entregada o cancelada ya no se puede reasignar.".to_string(),
        ));
    }

    let tecnico = entrada
        .tecnico
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    if let Some(codigo_tecnico) = &tecnico {
        let disponibles = repositorio::listar_tecnicos(pool).await?;
        if !disponibles.iter().any(|t| &t.codigo == codigo_tecnico) {
            return Ok(Gestion::Invalido(
                "Ese técnico ya no está disponible. Recarga la página.".to_string(),
            ));
        }
    }

    if repositorio::asignar_tecnico(pool, &orden.codigo, tecnico.as_deref()).await? {
        Ok(Gestion::Hecho)
    } else {
        Ok(Gestion::Invalido(
            "La orden ya terminó y no se puede reasignar. Recarga la página.".to_string(),
        ))
    }
}

// Cambia el estado de una orden y deja constancia de quién lo hizo.
pub async fn cambiar_estado(
    pool: &PgPool,
    usuario: &UsuarioActual,
    alcance: &Alcance,
    codigo: &str,
    entrada: EntradaEstado,
) -> Result<Gestion, sqlx::Error> {
    let Some(orden) = repositorio::obtener_orden(pool, codigo).await? else {
        return Ok(Gestion::NoEncontrado);
    };
    // Una orden de otro técnico "no existe" para quien no es su dueño.
    if !alcance.permite(&orden) {
        return Ok(Gestion::NoEncontrado);
    }

    let nuevo = entrada.estado.trim().to_string();
    if let Err(mensaje) = revisar_cambio(
        &orden.estado_codigo,
        &orden.estado_nombre,
        orden.tecnico_codigo.is_some(),
        &nuevo,
    ) {
        return Ok(Gestion::Invalido(mensaje));
    }
    let nota = match revisar_nota(entrada.nota) {
        Ok(nota) => nota,
        Err(mensaje) => return Ok(Gestion::Invalido(mensaje)),
    };

    let hecho = repositorio::cambiar_estado(
        pool,
        &orden.codigo,
        &orden.estado_codigo,
        &nuevo,
        &usuario.codigo,
        nota.as_deref(),
    )
    .await?;
    if hecho {
        Ok(Gestion::Hecho)
    } else {
        Ok(Gestion::Repetido(
            "La orden cambió mientras la mirabas. Recarga la página.".to_string(),
        ))
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn usuario(roles: &[&str]) -> UsuarioActual {
        UsuarioActual {
            codigo: "USU000001".to_string(),
            usuario: "pedro".to_string(),
            nombre: "Pedro".to_string(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
            debe_cambiar_clave: false,
        }
    }

    fn orden(tecnico: Option<&str>) -> OrdenPanel {
        OrdenPanel {
            codigo: "ORD000001".to_string(),
            cliente: "JUAN".to_string(),
            telefono: "987654321".to_string(),
            servicio: "Formateo".to_string(),
            equipo: None,
            problema: "No enciende".to_string(),
            fecha_cita: "2026-11-09 15:00".to_string(),
            estado_codigo: "CIT".to_string(),
            estado_nombre: "Cita agendada".to_string(),
            tecnico_codigo: tecnico.map(|t| t.to_string()),
            tecnico_nombre: None,
        }
    }

    #[test]
    fn el_alcance_depende_del_rol() {
        assert!(matches!(alcance_de(&usuario(&["ADM"])), Some(Alcance::Todas)));
        assert!(matches!(alcance_de(&usuario(&["ADM", "TEC"])), Some(Alcance::Todas)));
        assert!(matches!(alcance_de(&usuario(&["TEC"])), Some(Alcance::Propias(c)) if c == "USU000001"));
        assert!(alcance_de(&usuario(&["VEN"])).is_none());
        assert!(alcance_de(&usuario(&[])).is_none());
    }

    #[test]
    fn el_tecnico_solo_ve_sus_ordenes() {
        let propias = Alcance::Propias("USU000001".to_string());
        assert!(propias.permite(&orden(Some("USU000001"))));
        assert!(!propias.permite(&orden(Some("USU000002"))));
        assert!(!propias.permite(&orden(None)));
        assert!(Alcance::Todas.permite(&orden(None)));
        assert_eq!(propias.tecnico(), Some("USU000001"));
        assert_eq!(Alcance::Todas.tecnico(), None);
    }

    #[test]
    fn avanzar_exige_tecnico_y_transicion_valida() {
        assert!(revisar_cambio("CIT", "Cita agendada", true, "REC").is_ok());
        assert!(revisar_cambio("CIT", "Cita agendada", false, "REC").is_err()); // sin técnico
        assert!(revisar_cambio("CIT", "Cita agendada", false, "CAN").is_ok()); // cancelar siempre se puede
        assert!(revisar_cambio("CIT", "Cita agendada", true, "LIS").is_err()); // salta pasos
        assert!(revisar_cambio("ENT", "Entregado", true, "CAN").is_err()); // ya terminó
        assert!(revisar_cambio("REV", "En revisión", true, "XXX").is_err()); // estado inventado
    }

    #[test]
    fn notas() {
        assert_eq!(revisar_nota(None), Ok(None));
        assert_eq!(revisar_nota(Some("   ".to_string())), Ok(None));
        assert_eq!(
            revisar_nota(Some("  Cambié   la pasta\ntérmica ".to_string())),
            Ok(Some("Cambié la pasta térmica".to_string()))
        );
        assert!(revisar_nota(Some("a".repeat(301))).is_err());
        assert!(revisar_nota(Some("a".repeat(300))).is_ok());
    }
}
