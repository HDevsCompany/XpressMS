// lista.js (órdenes): la lista de órdenes de la pestaña activa. Cada fila abre el detalle de la orden.

import { crear } from '../../dom.js';
import { abrirDetalleOrden } from './detalle.js';
import { estaTerminada, nombrePersona, textoCita } from './formato.js';

// Clase de color de la etiqueta del estado.
function claseEstado(orden) {
  if (orden.estado_codigo === 'CAN') return 'etiqueta peligro';
  if (orden.estado_codigo === 'ENT' || orden.estado_codigo === 'LIS') return 'etiqueta precio';
  return 'etiqueta equipo';
}

function filaOrden(orden, contexto) {
  const fila = crear('article', 'fila fila-orden');

  const texto = crear('div', 'fila-texto');
  const titulo = crear('h3', 'fila-titulo', orden.servicio);
  texto.append(titulo);
  texto.append(
    crear('p', 'fila-descripcion', nombrePersona(orden.cliente) + ' · ' + orden.telefono),
  );

  const etiquetas = crear('div', 'etiquetas');
  etiquetas.append(crear('span', claseEstado(orden), orden.estado_nombre));
  etiquetas.append(crear('span', 'etiqueta', textoCita(orden.fecha_cita)));
  if (orden.equipo) etiquetas.append(crear('span', 'etiqueta', orden.equipo));
  if (orden.tecnico_nombre) {
    etiquetas.append(crear('span', 'etiqueta', 'Técnico: ' + nombrePersona(orden.tecnico_nombre)));
  } else if (!estaTerminada(orden)) {
    etiquetas.append(crear('span', 'etiqueta alerta', 'Sin técnico'));
  }
  etiquetas.append(crear('span', 'etiqueta codigo', orden.codigo));
  texto.append(etiquetas);

  const acciones = crear('div', 'fila-acciones');
  const abrir = crear('button', 'boton-secundario', estaTerminada(orden) ? 'Ver' : 'Atender');
  abrir.type = 'button';
  abrir.addEventListener('click', () => abrirDetalleOrden(orden.codigo, contexto));
  acciones.append(abrir);

  fila.append(texto, acciones);
  return fila;
}

// Dibuja las órdenes ya filtradas dentro de "contenedor". "contexto" lleva { esAdmin, mostrarError, recargar }.
export function dibujarOrdenes(contenedor, ordenes, mensajeVacio, contexto) {
  contenedor.replaceChildren();

  const barra = crear('div', 'lista-barra');
  const titulo = crear('h2', '', 'Órdenes');
  titulo.append(crear('small', '', ordenes.length === 1 ? '1 orden' : ordenes.length + ' órdenes'));
  barra.append(titulo);
  contenedor.append(barra);

  if (ordenes.length === 0) {
    contenedor.append(crear('p', 'vacio', mensajeVacio));
    return;
  }
  for (const orden of ordenes) contenedor.append(filaOrden(orden, contexto));
}
