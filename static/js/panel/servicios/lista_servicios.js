// lista_servicios.js: la pestaña «Servicios»: la lista con sus botones Editar (lápiz) y Eliminar (X).
// Ya no abre formularios: cada botón le pide a Juno que edite o elimine, y Juno confirma antes de hacer nada.

import { crear } from '../../dom.js';
import { icono } from './iconos.js';

const FOTO_JUNO = '/static/img/juno.png';
const CODIGO_DIAGNOSTICO = 'DIAGNOSTI'; // el servicio fijo que usa la página /servicios para las citas de diagnóstico: no se elimina

// Igual que el asistente: «Desde S/ 60.00» o «A cotizar».
function textoPrecio(servicio) {
  return servicio.precio_base ? 'Desde S/ ' + servicio.precio_base : 'A cotizar';
}

function filaServicio(servicio, acciones) {
  const fila = crear('article', 'fila');
  fila.dataset.codigo = servicio.codigo;

  const cuerpo = crear('div', 'fila-cuerpo');
  if (servicio.imagen) {
    const imagen = crear('img', 'fila-img');
    imagen.src = servicio.imagen;
    imagen.alt = '';
    imagen.loading = 'lazy';
    imagen.addEventListener('error', () => imagen.remove());
    cuerpo.append(imagen);
  }
  const texto = crear('div', 'fila-texto');
  texto.append(crear('h3', 'fila-titulo', servicio.nombre), crear('p', 'fila-descripcion', servicio.descripcion));

  const etiquetas = crear('div', 'etiquetas');
  const precio = crear('span', 'etiqueta precio', textoPrecio(servicio));
  precio.dataset.original = precio.textContent;
  etiquetas.append(precio);
  if (servicio.tiempo_estimado) etiquetas.append(crear('span', 'etiqueta', servicio.tiempo_estimado));
  for (const equipo of servicio.equipos) etiquetas.append(crear('span', 'etiqueta equipo', equipo.nombre));
  const fijo = servicio.codigo === CODIGO_DIAGNOSTICO;
  if (fijo) etiquetas.append(crear('span', 'pj-fijo', 'Servicio fijo · cita de diagnóstico'));
  texto.append(etiquetas);
  cuerpo.append(texto);

  const botones = crear('div', 'fila-acciones');
  const editar = crear('button', 'boton-secundario');
  editar.type = 'button';
  editar.append(icono('lapiz', 2.4), 'Editar');
  editar.addEventListener('click', () => acciones.editar(servicio));
  botones.append(editar);
  if (!fijo) {
    const eliminar = crear('button', 'boton-peligro');
    eliminar.type = 'button';
    eliminar.append(icono('equis', 2.6), 'Eliminar');
    eliminar.addEventListener('click', () => acciones.eliminar(servicio));
    botones.append(eliminar);
  }

  fila.append(cuerpo, botones);
  return fila;
}

// Dibuja la pestaña completa dentro de «contenedor». «acciones»: crear(), editar(servicio), eliminar(servicio) y zonaBorrador (el espacio de la ficha en construcción).
export function dibujarServicios(contenedor, servicios, acciones) {
  contenedor.replaceChildren();

  const barra = crear('div', 'pj-barra');
  const titulo = crear('h2', '', 'Servicios');
  titulo.append(crear('small', '', servicios.length === 1 ? '1 activo' : servicios.length + ' activos'));
  const nuevo = crear('button', 'pj-btn-juno');
  nuevo.type = 'button';
  const foto = crear('img');
  foto.src = FOTO_JUNO;
  foto.alt = '';
  nuevo.append(foto, 'Crear nuevo servicio');
  nuevo.addEventListener('click', acciones.crear);
  barra.append(titulo, nuevo);
  contenedor.append(barra);
  if (acciones.zonaBorrador) contenedor.append(acciones.zonaBorrador); // la ficha en construcción va justo debajo del título

  if (servicios.length === 0) {
    contenedor.append(crear('p', 'vacio', 'Todavía no hay servicios. Pídele a Juno que arme el primero con «Crear nuevo servicio».'));
    return;
  }
  for (const servicio of servicios) contenedor.append(filaServicio(servicio, acciones));
}

function pegarPastilla(fila, clase, texto) {
  fila.querySelector('.fila-titulo').append(crear('span', 'pj-pill ' + clase, texto));
}

// Marca en la lista el servicio que Juno está editando (amarillo) o por eliminar (rojo). Con borrador null se limpia todo.
export function marcarFilas(contenedor, borrador) {
  contenedor.querySelectorAll('.fila').forEach((fila) => {
    fila.classList.remove('edita', 'elimina');
    fila.querySelectorAll('.fila-titulo .pj-pill:not(.nuevo-pill)').forEach((p) => p.remove());
    const precio = fila.querySelector('.etiqueta.precio');
    if (precio && precio.dataset.original) precio.textContent = precio.dataset.original;
  });
  if (!borrador || !borrador.codigo) return;
  const fila = contenedor.querySelector('.fila[data-codigo="' + CSS.escape(borrador.codigo) + '"]');
  if (!fila) return;
  if (borrador.modo === 'editar') {
    fila.classList.add('edita');
    pegarPastilla(fila, 'am', 'EDITANDO');
    const dato = borrador.datos.find((d) => d.clave === 'precio');
    const precio = fila.querySelector('.etiqueta.precio');
    if (dato && dato.estado === 'cambio' && dato.antes && precio) {
      precio.replaceChildren(crear('s', '', precio.dataset.original), ' → ' + (dato.valor.startsWith('S/') ? 'Desde ' + dato.valor : dato.valor));
    }
  } else if (borrador.modo === 'eliminar') {
    fila.classList.add('elimina');
    pegarPastilla(fila, 'ro', 'SE ELIMINARÁ');
  }
}

// Resalta unos segundos el servicio que acaba de crearse o editarse.
export function resaltarFila(contenedor, codigo, texto) {
  const fila = contenedor.querySelector('.fila[data-codigo="' + CSS.escape(codigo) + '"]');
  if (!fila) return;
  fila.classList.add('nuevo');
  const pastilla = crear('span', 'pj-pill nuevo-pill', texto);
  fila.querySelector('.fila-titulo').append(pastilla);
  fila.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  setTimeout(() => {
    fila.classList.remove('nuevo');
    pastilla.remove();
  }, 6000);
}
