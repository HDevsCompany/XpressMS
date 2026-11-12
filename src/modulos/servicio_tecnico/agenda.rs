// agenda.rs: las reglas de la agenda de citas (qué días y horas se pueden reservar).
// Son funciones "puras": reciben la fecha y hora de Lima y devuelven resultados, sin tocar la base de datos.
// Por eso se prueban solas con "cargo test". El repositorio solo aporta cuántas citas hay por hora.
//
// IMPORTANTE: los horarios de abajo son un supuesto razonable. Hay que confirmar con H&Devs los
// horarios reales de atención y cuántas personas pueden atenderse a la vez; basta cambiar las
// constantes y la función horas_del_dia (nada más depende de ellas).

use crate::modulos::comun::fechas::{self, Fecha, Momento};

// Hasta cuántos días hacia adelante se puede agendar (contando hoy).
pub const DIAS_ADELANTE: i64 = 14;
// Cuántos días se muestran como botones a la vez (para no llenar el chat).
pub const MAX_DIAS_MOSTRADOS: usize = 6;
// Cuántas citas caben en una misma hora (técnicos que pueden recibir equipos a la vez).
pub const CITAS_POR_HORARIO: i64 = 2;
// Una cita debe pedirse con al menos este tiempo de anticipación.
pub const ANTICIPACION_MINUTOS: i64 = 60;

// Las horas de inicio de cita de cada día de la semana (0 = lunes ... 6 = domingo).
// Supuesto: lunes a viernes 9-13 h y 15-18 h; sábado 9-13 h; domingo cerrado.
pub fn horas_del_dia(dia_semana: u32) -> &'static [u32] {
    match dia_semana {
        0..=4 => &[9, 10, 11, 12, 15, 16, 17],
        5 => &[9, 10, 11, 12],
        _ => &[],
    }
}

// Las horas de inicio como rangos de atención: [9, 10, 11, 12, 15, 16, 17] -> "9–13 h y 15–18 h".
fn rangos_de_atencion(horas: &[u32]) -> String {
    let mut rangos: Vec<(u32, u32)> = Vec::new();
    for &hora in horas {
        match rangos.last_mut() {
            Some((_, fin)) if *fin == hora => *fin = hora + 1,
            _ => rangos.push((hora, hora + 1)),
        }
    }
    rangos.iter().map(|(a, b)| format!("{}–{} h", a, b)).collect::<Vec<_>>().join(" y ")
}

// El horario de atención en una frase, sacado de horas_del_dia (así nunca difiere de lo que realmente se puede reservar):
// "Atención de lunes a viernes 9–13 h y 15–18 h, sábados 9–13 h."
pub fn horario_texto() -> String {
    let entre_semana = rangos_de_atencion(horas_del_dia(0));
    let sabado = rangos_de_atencion(horas_del_dia(5));
    match (entre_semana.is_empty(), sabado.is_empty()) {
        (false, false) => format!("Atención de lunes a viernes {}, sábados {}.", entre_semana, sabado),
        (false, true) => format!("Atención de lunes a viernes {}.", entre_semana),
        _ => String::new(),
    }
}

// Minutos transcurridos desde el 1/1/1970 (hora de Lima) hasta ese día y hora: sirve para comparar momentos.
fn minutos_totales(fecha: Fecha, hora: u32, minuto: u32) -> i64 {
    fechas::a_dias(fecha) * 1440 + (hora as i64) * 60 + minuto as i64
}

// Las horas de ese día que todavía se pueden reservar (descartando las pasadas o muy cercanas),
// sin mirar cuántas citas hay ya. Si el día está fuera del rango permitido, no hay ninguna.
pub fn horas_reservables(ahora: Momento, fecha: Fecha) -> Vec<u32> {
    let dias_desde_hoy = fechas::a_dias(fecha) - fechas::a_dias(ahora.fecha);
    if !(0..=DIAS_ADELANTE).contains(&dias_desde_hoy) {
        return Vec::new();
    }
    let limite = minutos_totales(ahora.fecha, ahora.hora, ahora.minuto) + ANTICIPACION_MINUTOS;
    horas_del_dia(fechas::dia_semana(fecha))
        .iter()
        .copied()
        .filter(|&hora| minutos_totales(fecha, hora, 0) >= limite)
        .collect()
}

// Los próximos días con al menos una hora reservable (máximo MAX_DIAS_MOSTRADOS).
pub fn dias_disponibles(ahora: Momento) -> Vec<Fecha> {
    (0..=DIAS_ADELANTE)
        .map(|n| fechas::sumar_dias(ahora.fecha, n))
        .filter(|&dia| !horas_reservables(ahora, dia).is_empty())
        .take(MAX_DIAS_MOSTRADOS)
        .collect()
}

// De las horas reservables, deja las que aún tienen cupo. "ocupadas" son parejas (hora, citas ya agendadas).
pub fn horas_libres(ahora: Momento, fecha: Fecha, ocupadas: &[(i32, i64)]) -> Vec<u32> {
    horas_reservables(ahora, fecha)
        .into_iter()
        .filter(|&hora| {
            let citas = ocupadas
                .iter()
                .find(|(h, _)| *h == hora as i32)
                .map_or(0, |(_, cantidad)| *cantidad);
            citas < CITAS_POR_HORARIO
        })
        .collect()
}

// El texto "2026-11-09 15:00" que se envía a la base de datos (siempre es hora de Lima).
pub fn fecha_hora_texto(fecha: Fecha, hora: u32) -> String {
    format!("{} {:02}:00", fechas::iso(fecha), hora)
}

// Las horas como botones: "09:00", "10:00"...
pub fn botones_de_horas(horas: &[u32]) -> Vec<String> {
    horas.iter().map(|&h| fechas::hora_texto(h, 0)).collect()
}

// Lee una hora escrita por la persona: "09:00", "9:00", "9" o "15". Devuelve solo la hora en punto.
// (Las citas empiezan siempre en punto; otra cosa, como "9:30", no se acepta.)
pub fn hora_desde_texto(texto: &str) -> Option<u32> {
    let limpio = texto.trim().to_lowercase().replace(['h', ' '], "");
    let entero = limpio.strip_suffix(":00").unwrap_or(&limpio);
    if entero.is_empty() || entero.len() > 2 || !entero.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let hora: u32 = entero.parse().ok()?;
    (hora < 24).then_some(hora)
}

// Lee un día escrito por la persona entre los días ofrecidos: "lun 09/11", "09/11", "9/11" o "9-11".
// Devuelve la posición del día en la lista ofrecida.
pub fn dia_desde_texto(texto: &str, ofrecidos: &[Fecha]) -> Option<usize> {
    // Se queda con lo que parece "día/mes": números separados por / o -.
    let candidato = texto
        .split_whitespace()
        .find(|parte| parte.contains('/') || parte.contains('-'))?;
    let numeros: Vec<&str> = candidato.split(['/', '-']).collect();
    if numeros.len() < 2 {
        return None;
    }
    let dia: u32 = numeros[0].parse().ok()?;
    let mes: u32 = numeros[1].parse().ok()?;
    ofrecidos
        .iter()
        .position(|f| f.dia == dia && f.mes == mes)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn f(anio: i32, mes: u32, dia: u32) -> Fecha {
        Fecha { anio, mes, dia }
    }

    // Lunes 2 de noviembre de 2026 a la hora indicada.
    fn lunes(hora: u32, minuto: u32) -> Momento {
        Momento { fecha: f(2026, 11, 2), hora, minuto }
    }

    #[test]
    fn horarios_por_dia() {
        assert_eq!(horas_del_dia(0).len(), 7); // lunes
        assert_eq!(horas_del_dia(5), &[9, 10, 11, 12]); // sábado
        assert!(horas_del_dia(6).is_empty()); // domingo cerrado
    }

    #[test]
    fn hoy_solo_quedan_las_horas_con_anticipacion() {
        // Lunes 10:30: la hora mínima es 11:30, así que quedan 12, 15, 16 y 17.
        assert_eq!(horas_reservables(lunes(10, 30), f(2026, 11, 2)), vec![12, 15, 16, 17]);
        // Lunes 10:00 justo: 11:00 ya cumple la hora de anticipación.
        assert_eq!(horas_reservables(lunes(10, 0), f(2026, 11, 2)), vec![11, 12, 15, 16, 17]);
        // Lunes 16:30: ya no queda ninguna hora hoy.
        assert!(horas_reservables(lunes(16, 30), f(2026, 11, 2)).is_empty());
    }

    #[test]
    fn el_futuro_tiene_todas_sus_horas() {
        assert_eq!(horas_reservables(lunes(20, 0), f(2026, 11, 3)).len(), 7); // martes
        assert!(horas_reservables(lunes(8, 0), f(2026, 11, 8)).is_empty()); // domingo
    }

    #[test]
    fn fuera_del_rango_no_se_reserva() {
        assert!(horas_reservables(lunes(8, 0), f(2026, 11, 1)).is_empty()); // ayer
        assert!(horas_reservables(lunes(8, 0), f(2026, 11, 17)).is_empty()); // 15 días adelante
        assert!(!horas_reservables(lunes(8, 0), f(2026, 11, 16)).is_empty()); // 14 días adelante (lunes)
    }

    #[test]
    fn dias_ofrecidos_saltan_el_domingo_y_hoy_si_ya_paso() {
        // Lunes por la noche: hoy ya no se puede; salen mar, mié, jue, vie, sáb y (salta dom) lun.
        let dias = dias_disponibles(lunes(20, 0));
        assert_eq!(dias.len(), MAX_DIAS_MOSTRADOS);
        assert_eq!(dias[0], f(2026, 11, 3));
        assert!(!dias.contains(&f(2026, 11, 8))); // domingo
        assert_eq!(dias[5], f(2026, 11, 9)); // lunes siguiente
        // Lunes por la mañana temprano: hoy sí aparece.
        assert_eq!(dias_disponibles(lunes(7, 0))[0], f(2026, 11, 2));
    }

    #[test]
    fn cupo_por_hora() {
        let ocupadas = [(9, 2), (10, 1)];
        // El martes 9:00 ya tiene 2 citas (lleno); 10:00 tiene 1 (aún cabe una).
        let libres = horas_libres(lunes(20, 0), f(2026, 11, 3), &ocupadas);
        assert!(!libres.contains(&9));
        assert!(libres.contains(&10));
        assert_eq!(libres.len(), 6);
    }

    #[test]
    fn horario_en_una_frase() {
        assert_eq!(rangos_de_atencion(&[9, 10, 11, 12, 15, 16, 17]), "9–13 h y 15–18 h");
        assert_eq!(rangos_de_atencion(&[9, 10, 11, 12]), "9–13 h");
        assert_eq!(rangos_de_atencion(&[]), "");
        assert_eq!(horario_texto(), "Atención de lunes a viernes 9–13 h y 15–18 h, sábados 9–13 h.");
    }

    #[test]
    fn textos_de_fecha_y_hora() {
        assert_eq!(fecha_hora_texto(f(2026, 11, 9), 9), "2026-11-09 09:00");
        assert_eq!(botones_de_horas(&[9, 15]), vec!["09:00", "15:00"]);
    }

    #[test]
    fn lee_horas_escritas() {
        assert_eq!(hora_desde_texto("09:00"), Some(9));
        assert_eq!(hora_desde_texto(" 9:00 "), Some(9));
        assert_eq!(hora_desde_texto("15"), Some(15));
        assert_eq!(hora_desde_texto("15h"), Some(15));
        assert_eq!(hora_desde_texto("9:30"), None);
        assert_eq!(hora_desde_texto("25"), None);
        assert_eq!(hora_desde_texto("mañana"), None);
        assert_eq!(hora_desde_texto(""), None);
    }

    #[test]
    fn lee_dias_escritos() {
        let ofrecidos = [f(2026, 11, 3), f(2026, 11, 4), f(2026, 11, 9)];
        assert_eq!(dia_desde_texto("mar 03/11", &ofrecidos), Some(0));
        assert_eq!(dia_desde_texto("4/11", &ofrecidos), Some(1));
        assert_eq!(dia_desde_texto("el 09-11 por favor", &ofrecidos), Some(2));
        assert_eq!(dia_desde_texto("10/11", &ofrecidos), None);
        assert_eq!(dia_desde_texto("mañana", &ofrecidos), None);
    }
}
