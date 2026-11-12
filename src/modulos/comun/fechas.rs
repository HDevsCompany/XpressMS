// fechas.rs: cálculos de calendario para las citas (día de la semana, sumar días, hora de Lima).
// Son funciones "puras": no tocan base de datos ni red, así se prueban solas con "cargo test".
// No usamos librerías de fechas: solo necesitamos días, meses y horas, y Lima no tiene horario de
// verano (siempre es UTC-5), así que el cálculo es exacto y corto.

use std::time::{SystemTime, UNIX_EPOCH};

// Lima está 5 horas detrás de UTC todo el año.
const SEGUNDOS_DESFASE_LIMA: i64 = 5 * 3600;

const DIAS_CORTOS: [&str; 7] = ["lun", "mar", "mié", "jue", "vie", "sáb", "dom"];
const DIAS_LARGOS: [&str; 7] = [
    "lunes", "martes", "miércoles", "jueves", "viernes", "sábado", "domingo",
];
const MESES: [&str; 12] = [
    "enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "setiembre",
    "octubre", "noviembre", "diciembre",
];

// Un día del calendario (sin hora).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fecha {
    pub anio: i32,
    pub mes: u32, // 1 a 12
    pub dia: u32, // 1 a 31
}

// Un día con su hora, en hora de Lima.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Momento {
    pub fecha: Fecha,
    pub hora: u32,
    pub minuto: u32,
}

// Convierte "días desde el 1 de enero de 1970" en una fecha (algoritmo de Howard Hinnant).
pub fn desde_dias(dias: i64) -> Fecha {
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097; // día dentro de la era (0 a 146096)
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dia = doy - (153 * mp + 2) / 5 + 1;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 };
    let anio = if mes <= 2 { y + 1 } else { y };
    Fecha { anio: anio as i32, mes: mes as u32, dia: dia as u32 }
}

// Lo contrario: de una fecha a "días desde el 1 de enero de 1970".
pub fn a_dias(fecha: Fecha) -> i64 {
    let mes = fecha.mes as i64;
    let y = fecha.anio as i64 - if mes <= 2 { 1 } else { 0 };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if mes > 2 { mes - 3 } else { mes + 9 }) + 2) / 5 + fecha.dia as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

// Día de la semana: 0 = lunes ... 6 = domingo. El 1/1/1970 fue jueves (3).
pub fn dia_semana(fecha: Fecha) -> u32 {
    (a_dias(fecha) + 3).rem_euclid(7) as u32
}

// La fecha que queda sumando (o restando, si es negativo) esa cantidad de días.
pub fn sumar_dias(fecha: Fecha, dias: i64) -> Fecha {
    desde_dias(a_dias(fecha) + dias)
}

// "2026-11-09": el formato que viaja entre el servidor y la base de datos.
pub fn iso(fecha: Fecha) -> String {
    format!("{:04}-{:02}-{:02}", fecha.anio, fecha.mes, fecha.dia)
}

// Lee "2026-11-09". Devuelve None si el texto no tiene ese formato o la fecha no existe (30 de febrero).
pub fn desde_iso(texto: &str) -> Option<Fecha> {
    let partes: Vec<&str> = texto.split('-').collect();
    if partes.len() != 3 || partes[0].len() != 4 || partes[1].len() != 2 || partes[2].len() != 2 {
        return None;
    }
    let fecha = Fecha {
        anio: partes[0].parse().ok()?,
        mes: partes[1].parse().ok()?,
        dia: partes[2].parse().ok()?,
    };
    // Si la fecha no existe, ir a días y volver la cambia: así se detecta.
    if fecha.mes == 0 || fecha.mes > 12 || fecha.dia == 0 || desde_dias(a_dias(fecha)) != fecha {
        return None;
    }
    Some(fecha)
}

// "lun 09/11": corta, para botones.
pub fn corta(fecha: Fecha) -> String {
    format!(
        "{} {:02}/{:02}",
        DIAS_CORTOS[dia_semana(fecha) as usize],
        fecha.dia,
        fecha.mes
    )
}

// "lun": el día de la semana abreviado.
pub fn semana_corta(fecha: Fecha) -> &'static str {
    DIAS_CORTOS[dia_semana(fecha) as usize]
}

// "nov": el mes abreviado.
pub fn mes_corto(fecha: Fecha) -> String {
    MESES[(fecha.mes - 1) as usize].chars().take(3).collect()
}

// "lunes 9 de noviembre": para frases.
pub fn larga(fecha: Fecha) -> String {
    format!(
        "{} {} de {}",
        DIAS_LARGOS[dia_semana(fecha) as usize],
        fecha.dia,
        MESES[(fecha.mes - 1) as usize]
    )
}

// "09:00" o "15:00".
pub fn hora_texto(hora: u32, minuto: u32) -> String {
    format!("{:02}:{:02}", hora, minuto)
}

// Convierte segundos desde 1970 (UTC) en el día y la hora de Lima. Separada de ahora_lima() para probarla.
pub fn momento_desde_segundos(segundos_utc: i64) -> Momento {
    let local = segundos_utc - SEGUNDOS_DESFASE_LIMA;
    let dias = local.div_euclid(86_400);
    let resto = local.rem_euclid(86_400);
    Momento {
        fecha: desde_dias(dias),
        hora: (resto / 3600) as u32,
        minuto: ((resto % 3600) / 60) as u32,
    }
}

// El día y la hora de Lima en este instante.
pub fn ahora_lima() -> Momento {
    let segundos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    momento_desde_segundos(segundos)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn f(anio: i32, mes: u32, dia: u32) -> Fecha {
        Fecha { anio, mes, dia }
    }

    #[test]
    fn dias_y_fechas_se_convierten_en_ambos_sentidos() {
        assert_eq!(a_dias(f(1970, 1, 1)), 0);
        assert_eq!(desde_dias(0), f(1970, 1, 1));
        assert_eq!(desde_dias(a_dias(f(2026, 11, 2))), f(2026, 11, 2));
        assert_eq!(desde_dias(a_dias(f(2024, 2, 29))), f(2024, 2, 29)); // año bisiesto
        for d in (-1000..30000).step_by(7) {
            assert_eq!(a_dias(desde_dias(d)), d);
        }
    }

    #[test]
    fn dia_de_la_semana() {
        assert_eq!(dia_semana(f(1970, 1, 1)), 3); // jueves
        assert_eq!(dia_semana(f(2026, 11, 2)), 0); // lunes
        assert_eq!(dia_semana(f(2026, 11, 7)), 5); // sábado
        assert_eq!(dia_semana(f(2026, 11, 8)), 6); // domingo
        assert_eq!(dia_semana(f(2000, 1, 1)), 5); // sábado
    }

    #[test]
    fn sumar_dias_cruza_meses_y_anios() {
        assert_eq!(sumar_dias(f(2026, 11, 28), 5), f(2026, 12, 3));
        assert_eq!(sumar_dias(f(2026, 12, 30), 3), f(2027, 1, 2));
        assert_eq!(sumar_dias(f(2028, 2, 28), 1), f(2028, 2, 29));
        assert_eq!(sumar_dias(f(2027, 2, 28), 1), f(2027, 3, 1));
        assert_eq!(sumar_dias(f(2026, 11, 2), -2), f(2026, 10, 31));
    }

    #[test]
    fn texto_iso_ida_y_vuelta() {
        assert_eq!(iso(f(2026, 11, 9)), "2026-11-09");
        assert_eq!(desde_iso("2026-11-09"), Some(f(2026, 11, 9)));
        assert_eq!(desde_iso("2026-02-30"), None);
        assert_eq!(desde_iso("2026-13-01"), None);
        assert_eq!(desde_iso("2026-00-10"), None);
        assert_eq!(desde_iso("2026-11-9"), None);
        assert_eq!(desde_iso("hola"), None);
        assert_eq!(desde_iso("2026-11-09-1"), None);
    }

    #[test]
    fn textos_para_la_persona() {
        assert_eq!(corta(f(2026, 11, 9)), "lun 09/11");
        assert_eq!(corta(f(2026, 11, 11)), "mié 11/11");
        assert_eq!(larga(f(2026, 11, 9)), "lunes 9 de noviembre");
        assert_eq!(larga(f(2026, 9, 12)), "sábado 12 de setiembre");
        assert_eq!(hora_texto(9, 0), "09:00");
        assert_eq!(hora_texto(15, 30), "15:30");
    }

    #[test]
    fn hora_de_lima_resta_cinco_horas() {
        // 2026-11-02 15:30 UTC = 10:30 en Lima, el mismo día.
        let utc = a_dias(f(2026, 11, 2)) * 86_400 + 15 * 3600 + 30 * 60;
        let m = momento_desde_segundos(utc);
        assert_eq!((m.fecha, m.hora, m.minuto), (f(2026, 11, 2), 10, 30));
        // 2026-11-03 02:00 UTC = 21:00 del día anterior en Lima.
        let utc = a_dias(f(2026, 11, 3)) * 86_400 + 2 * 3600;
        let m = momento_desde_segundos(utc);
        assert_eq!((m.fecha, m.hora, m.minuto), (f(2026, 11, 2), 21, 0));
    }
}
