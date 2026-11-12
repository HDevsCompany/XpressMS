// formato.js (apartados): cómo se escriben las fechas y las clases de color en la pantalla de apartados.

export { nombrePersona, textoCita } from '../ordenes/formato.js';

export const PENDIENTE = 'PEN';

// Clase de color de la etiqueta del estado.
export function claseEstado(apartado) {
  if (apartado.estado_codigo === 'ENT') return 'etiqueta precio';
  if (apartado.estado_codigo === 'CAN' || apartado.estado_codigo === 'VEN') return 'etiqueta peligro';
  return 'etiqueta equipo';
}
