// formato.js (órdenes): cómo se escriben las fechas y los nombres en la pantalla de órdenes.

import { aTitulo } from '../personal/roles.js';

const DIAS = ['dom', 'lun', 'mar', 'mié', 'jue', 'vie', 'sáb'];

// "2026-11-09 15:00" -> "lun 09/11 · 15:00". Si el texto no tiene ese formato, se muestra tal cual.
export function textoCita(fecha) {
  const partes = /^(\d{4})-(\d{2})-(\d{2}) (\d{2}:\d{2})$/.exec(fecha || '');
  if (!partes) return fecha || '';
  const [, anio, mes, dia, hora] = partes;
  // Se arma con los números del texto (sin zonas horarias): el día de la semana sale del calendario.
  const diaSemana = DIAS[new Date(Number(anio), Number(mes) - 1, Number(dia)).getDay()];
  return diaSemana + ' ' + dia + '/' + mes + ' · ' + hora;
}

// Los nombres vienen en mayúsculas ("PEDRO JUAN GUEVARA"): se muestran como "Pedro Juan Guevara".
export function nombrePersona(texto) {
  return aTitulo(texto);
}

// ¿La orden ya terminó (entregada o cancelada)?
export function estaTerminada(orden) {
  return orden.estado_codigo === 'ENT' || orden.estado_codigo === 'CAN';
}
