// tarjeta_orden.js: la tarjeta de una cita u orden de servicio: qué servicio es, cuándo, qué le pasa al equipo
// y, si ya está guardada, el seguimiento paso a paso. Estilos en static/css/orden.css.

import { crear } from '../dom.js';
import { agregar } from '../burbujas.js';

const NS_SVG = 'http://www.w3.org/2000/svg';

// Icono de calendario dibujado con SVG, para no depender de imágenes externas.
function crearIcono() {
  const dibujo = document.createElementNS(NS_SVG, 'svg');
  dibujo.setAttribute('viewBox', '0 0 24 24');
  dibujo.setAttribute('aria-hidden', 'true');
  const trazos = [
    'M8 2v4M16 2v4',
    'M3 10h18',
    'M5 4h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z',
  ];
  for (const d of trazos) {
    const trazo = document.createElementNS(NS_SVG, 'path');
    trazo.setAttribute('d', d);
    dibujo.appendChild(trazo);
  }
  return dibujo;
}

// Una fila de la tarjeta: etiqueta pequeña arriba y el valor debajo.
function crearDato(etiqueta, valor, clase = '') {
  const dato = crear('div', 'orden-dato ' + clase);
  dato.append(crear('dt', '', etiqueta), crear('dd', '', valor));
  return dato;
}

// El seguimiento: una lista vertical. Cada paso es "hecho", "actual" o "pendiente".
function crearSeguimiento(pasos) {
  const lista = crear('ol', 'orden-pasos');
  for (const paso of pasos) {
    const item = crear('li', 'orden-paso ' + paso.estado);
    const punto = crear('span', 'orden-punto');
    punto.setAttribute('aria-hidden', 'true');
    const texto = crear('span', 'orden-paso-texto');
    texto.appendChild(crear('span', 'orden-paso-nombre', paso.nombre));
    if (paso.fecha) texto.appendChild(crear('span', 'orden-paso-fecha', paso.fecha));
    item.append(punto, texto);
    lista.appendChild(item);
  }
  return lista;
}

// "datos" es { codigo?, estado, servicio, equipo?, cita, problema, seguimiento } (lo envía el servidor).
export function mostrarTarjetaOrden(datos) {
  const tarjeta = crear('div', 'orden');

  // --- Cabecera: icono + servicio + estado ---
  const cabecera = crear('div', 'orden-cabecera');
  const icono = crear('span', 'orden-icono');
  icono.appendChild(crearIcono());
  const titulo = crear('div', 'orden-titulo');
  titulo.append(crear('h3', '', datos.servicio), crear('span', 'orden-estado', datos.estado));
  cabecera.append(icono, titulo);

  // --- Código de la orden (solo cuando ya está guardada) ---
  if (datos.codigo) {
    const codigo = crear('div', 'orden-codigo');
    codigo.append(crear('span', '', 'Código de orden'), crear('strong', '', datos.codigo));
    tarjeta.append(cabecera, codigo);
  } else {
    tarjeta.append(cabecera);
  }

  // --- Datos: cita, equipo y problema ---
  const lista = crear('dl', 'orden-datos');
  lista.appendChild(crearDato('Cita', datos.cita, 'completo'));
  if (datos.equipo) lista.appendChild(crearDato('Equipo', datos.equipo, 'completo'));
  lista.appendChild(crearDato('Problema', datos.problema, 'completo'));
  tarjeta.appendChild(lista);

  // --- Seguimiento ---
  if (Array.isArray(datos.seguimiento) && datos.seguimiento.length > 0) {
    tarjeta.append(crear('p', 'orden-subtitulo', 'Seguimiento'), crearSeguimiento(datos.seguimiento));
  }

  agregar(tarjeta);
}
