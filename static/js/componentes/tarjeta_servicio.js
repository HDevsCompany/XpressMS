// tarjeta_servicio.js: la tarjeta con el detalle de un servicio técnico (nombre, qué incluye, precio, tiempo, equipo).
// Estilos en static/css/servicio.css.

import { crear } from '../dom.js';
import { agregar } from '../burbujas.js';

const NS_SVG = 'http://www.w3.org/2000/svg';

// Icono de herramienta (llave) dibujado con SVG, para no depender de imágenes externas.
function crearIcono() {
  const dibujo = document.createElementNS(NS_SVG, 'svg');
  dibujo.setAttribute('viewBox', '0 0 24 24');
  dibujo.setAttribute('aria-hidden', 'true');
  const trazo = document.createElementNS(NS_SVG, 'path');
  trazo.setAttribute(
    'd',
    'M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z',
  );
  dibujo.appendChild(trazo);
  return dibujo;
}

// Una fila de la tarjeta: etiqueta pequeña arriba y el valor debajo.
function crearDato(etiqueta, valor, clase = '') {
  const dato = crear('div', 'servicio-dato ' + clase);
  dato.append(crear('dt', '', etiqueta), crear('dd', '', valor));
  return dato;
}

// "datos" es { nombre, descripcion, precio, nota, tiempo?, equipo? } (lo envía el servidor).
export function mostrarTarjetaServicio(datos) {
  const tarjeta = crear('div', 'servicio');

  // --- Cabecera: icono + nombre ---
  const cabecera = crear('div', 'servicio-cabecera');
  const icono = crear('span', 'servicio-icono');
  icono.appendChild(crearIcono());
  cabecera.append(icono, crear('h3', '', datos.nombre));

  // --- Qué incluye ---
  const descripcion = crear('p', 'servicio-descripcion', datos.descripcion);

  // --- Datos: precio (destacado), tiempo y equipo (solo si existen) ---
  const lista = crear('dl', 'servicio-datos');
  lista.appendChild(crearDato('Precio', datos.precio, 'precio'));
  if (datos.tiempo) lista.appendChild(crearDato('Tiempo estimado', datos.tiempo));
  if (datos.equipo) lista.appendChild(crearDato('Equipo', datos.equipo));

  // --- Aviso sobre el precio ---
  const nota = crear('p', 'servicio-nota', datos.nota);

  tarjeta.append(cabecera, descripcion, lista, nota);
  agregar(tarjeta);
}
