// estados.rs: los estados de una orden de servicio y a cuáles puede pasar desde cada uno.
// Son reglas "puras" (sin base de datos): se prueban solas con "cargo test".
// El catálogo con los nombres vive en la tabla ServicioTecnico.EstadosOrden; aquí solo están los
// códigos y las reglas de cambio, que son decisión del negocio y por eso viven en el backend.
//
//   CIT -> REC -> REV -> REP -> LIS -> ENT        (camino normal)
//                  REV -> LIS                     (si solo era revisar, sin reparar)
//   cualquiera (menos ENT y CAN) -> CAN           (cancelada)

use super::modelo::{EstadoOrden, Hito, RegistroHistorial};

pub const CITA: &str = "CIT";
pub const RECIBIDO: &str = "REC";
pub const REVISION: &str = "REV";
pub const REPARACION: &str = "REP";
pub const LISTO: &str = "LIS";
pub const ENTREGADO: &str = "ENT";
pub const CANCELADA: &str = "CAN";

// El camino normal, en orden (sin la cancelación).
const CAMINO: [&str; 6] = [CITA, RECIBIDO, REVISION, REPARACION, LISTO, ENTREGADO];

// Los estados a los que se puede pasar desde el actual.
pub fn siguientes_permitidos(actual: &str) -> Vec<&'static str> {
    match actual {
        CITA => vec![RECIBIDO, CANCELADA],
        RECIBIDO => vec![REVISION, CANCELADA],
        REVISION => vec![REPARACION, LISTO, CANCELADA],
        REPARACION => vec![LISTO, CANCELADA],
        LISTO => vec![ENTREGADO, CANCELADA],
        _ => Vec::new(), // ENTREGADO y CANCELADA son finales
    }
}

pub fn puede_pasar(actual: &str, nuevo: &str) -> bool {
    siguientes_permitidos(actual).contains(&nuevo)
}

// ¿El cliente puede cancelar su propia cita? Solo mientras sigue siendo una cita: una vez que
// el equipo llegó al taller, cancelar lo decide el personal.
pub fn cliente_puede_cancelar(actual: &str) -> bool {
    actual == CITA
}

// Arma el seguimiento que ve el cliente: los pasos ya vividos (con su fecha), el actual y los que faltan.
// - "catalogo" son los estados con su nombre (de la base de datos).
// - "actual" es el código del estado de la orden.
// - "historial" son los cambios ya registrados, con su fecha.
// Un paso que se saltó (por ejemplo, "En reparación" cuando solo hubo revisión) no aparece.
// Si la orden está cancelada, no se muestran pasos pendientes: termina en "Cancelada".
pub fn seguimiento(
    catalogo: &[EstadoOrden],
    actual: &str,
    historial: &[RegistroHistorial],
) -> Vec<Hito> {
    let nombre_de = |codigo: &str| {
        catalogo
            .iter()
            .find(|e| e.codigo == codigo)
            .map(|e| e.nombre.clone())
            .unwrap_or_else(|| codigo.to_string())
    };
    // La fecha en que la orden entró a ese estado (la más reciente si hubiera repetidos).
    let fecha_de = |codigo: &str| {
        historial
            .iter()
            .rev()
            .find(|h| h.estado == codigo)
            .map(|h| h.fecha.clone())
    };

    let posicion_actual = CAMINO.iter().position(|c| *c == actual);
    let mut hitos = Vec::new();

    for (posicion, codigo) in CAMINO.iter().enumerate() {
        let fecha = fecha_de(codigo);
        let estado = if fecha.is_some() {
            if *codigo == actual { "actual" } else { "hecho" }
        } else if posicion_actual.is_some_and(|p| posicion > p) {
            "pendiente"
        } else {
            continue; // saltado, o no aplica porque la orden se canceló
        };
        hitos.push(Hito {
            nombre: nombre_de(codigo),
            fecha,
            estado: estado.to_string(),
        });
    }

    if actual == CANCELADA {
        hitos.push(Hito {
            nombre: nombre_de(CANCELADA),
            fecha: fecha_de(CANCELADA),
            estado: "actual".to_string(),
        });
    }
    hitos
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn catalogo() -> Vec<EstadoOrden> {
        [
            (CITA, "Cita agendada"),
            (RECIBIDO, "Recibido en taller"),
            (REVISION, "En revisión"),
            (REPARACION, "En reparación"),
            (LISTO, "Listo para recoger"),
            (ENTREGADO, "Entregado"),
            (CANCELADA, "Cancelada"),
        ]
        .iter()
        .map(|(c, n)| EstadoOrden { codigo: c.to_string(), nombre: n.to_string() })
        .collect()
    }

    fn registro(estado: &str, fecha: &str) -> RegistroHistorial {
        RegistroHistorial { estado: estado.to_string(), fecha: fecha.to_string() }
    }

    #[test]
    fn transiciones_permitidas() {
        assert!(puede_pasar(CITA, RECIBIDO));
        assert!(puede_pasar(CITA, CANCELADA));
        assert!(puede_pasar(REVISION, LISTO)); // solo revisión
        assert!(puede_pasar(LISTO, ENTREGADO));
        assert!(!puede_pasar(CITA, LISTO)); // no se puede saltar pasos
        assert!(!puede_pasar(REPARACION, REVISION)); // no se retrocede
        assert!(!puede_pasar(ENTREGADO, CANCELADA)); // ya entregada
        assert!(!puede_pasar(CANCELADA, CITA)); // ya cancelada
        assert!(siguientes_permitidos("XXX").is_empty());
    }

    #[test]
    fn solo_se_cancela_como_cliente_siendo_cita() {
        assert!(cliente_puede_cancelar(CITA));
        assert!(!cliente_puede_cancelar(RECIBIDO));
    }

    #[test]
    fn seguimiento_de_una_cita_nueva() {
        let h = seguimiento(&catalogo(), CITA, &[registro(CITA, "02/11 10:00")]);
        assert_eq!(h.len(), 6);
        assert_eq!(h[0].estado, "actual");
        assert_eq!(h[0].fecha.as_deref(), Some("02/11 10:00"));
        assert!(h[1..].iter().all(|x| x.estado == "pendiente" && x.fecha.is_none()));
    }

    #[test]
    fn seguimiento_a_mitad_de_camino() {
        let hist = [
            registro(CITA, "02/11 10:00"),
            registro(RECIBIDO, "03/11 09:05"),
            registro(REVISION, "03/11 11:00"),
        ];
        let h = seguimiento(&catalogo(), REVISION, &hist);
        let estados: Vec<&str> = h.iter().map(|x| x.estado.as_str()).collect();
        assert_eq!(estados, ["hecho", "hecho", "actual", "pendiente", "pendiente", "pendiente"]);
        assert_eq!(h[2].nombre, "En revisión");
    }

    #[test]
    fn seguimiento_omite_el_paso_saltado() {
        let hist = [
            registro(CITA, "a"),
            registro(RECIBIDO, "b"),
            registro(REVISION, "c"),
            registro(LISTO, "d"),
        ];
        let h = seguimiento(&catalogo(), LISTO, &hist);
        assert!(!h.iter().any(|x| x.nombre == "En reparación"));
        assert_eq!(h.last().unwrap().nombre, "Entregado");
        assert_eq!(h.last().unwrap().estado, "pendiente");
    }

    #[test]
    fn seguimiento_de_una_cancelada() {
        let hist = [registro(CITA, "a"), registro(CANCELADA, "b")];
        let h = seguimiento(&catalogo(), CANCELADA, &hist);
        let nombres: Vec<&str> = h.iter().map(|x| x.nombre.as_str()).collect();
        assert_eq!(nombres, ["Cita agendada", "Cancelada"]);
        assert_eq!(h[0].estado, "hecho");
        assert_eq!(h[1].estado, "actual");
    }
}
