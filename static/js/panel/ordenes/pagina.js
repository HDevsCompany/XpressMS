// pagina.js (órdenes): la página /panel/ordenes. Carga las órdenes, maneja las pestañas y las vuelve a cargar
// después de cada cambio. El servidor ya entrega solo las órdenes que esta persona puede ver.

import { listarOrdenes, quienSoy, salir } from '../api.js';
import { crear } from '../../dom.js';
import { dibujarOrdenes } from './lista.js';
import { estaTerminada } from './formato.js';

const contenedorPestanas = document.getElementById('pestanas');
const vistaOrdenes = document.getElementById('vista-ordenes');
const avisoError = document.getElementById('aviso-error');

// Las pestañas: cada una filtra la misma lista. "soloAdmin" = la ve solo el Administrador.
const PESTANAS = [
  {
    id: 'curso',
    titulo: 'En curso',
    filtro: (orden) => !estaTerminada(orden),
    vacio: 'No hay órdenes en curso. Cuando un cliente agende una cita desde el asistente, aparecerá aquí.',
  },
  {
    id: 'sin-tecnico',
    titulo: 'Sin técnico',
    soloAdmin: true,
    filtro: (orden) => !estaTerminada(orden) && !orden.tecnico_codigo,
    vacio: 'Todas las órdenes en curso ya tienen técnico.',
  },
  {
    id: 'terminadas',
    titulo: 'Terminadas',
    filtro: (orden) => estaTerminada(orden),
    vacio: 'Todavía no hay órdenes entregadas ni canceladas.',
  },
];

let esAdmin = false;
let pestanaActiva = 'curso';
let ordenes = [];
let temporizadorError = null;

function mostrarError(texto) {
  avisoError.textContent = texto;
  avisoError.hidden = false;
  clearTimeout(temporizadorError);
  temporizadorError = setTimeout(() => (avisoError.hidden = true), 8000);
}

const contexto = { esAdmin: false, mostrarError, recargar };

function pestanasVisibles() {
  return PESTANAS.filter((pestana) => esAdmin || !pestana.soloAdmin);
}

// Dibuja las pestañas (con cuántas órdenes tiene cada una) y la lista de la activa.
function dibujar() {
  contenedorPestanas.replaceChildren();
  for (const pestana of pestanasVisibles()) {
    const cantidad = ordenes.filter(pestana.filtro).length;
    const boton = crear('button', 'pestana', pestana.titulo + ' (' + cantidad + ')');
    boton.type = 'button';
    boton.setAttribute('role', 'tab');
    boton.setAttribute('aria-selected', String(pestana.id === pestanaActiva));
    boton.addEventListener('click', () => {
      pestanaActiva = pestana.id;
      avisoError.hidden = true;
      dibujar();
    });
    contenedorPestanas.append(boton);
  }
  const activa = PESTANAS.find((pestana) => pestana.id === pestanaActiva) || PESTANAS[0];
  dibujarOrdenes(vistaOrdenes, ordenes.filter(activa.filtro), activa.vacio, contexto);
}

// Trae las órdenes del servidor y las dibuja. Se llama al abrir y después de cada cambio.
async function recargar() {
  try {
    const resultado = await listarOrdenes();
    if (!resultado.ok) {
      mostrarError('No se pudieron cargar las órdenes. Recarga la página.');
      return;
    }
    ordenes = resultado.datos;
    dibujar();
  } catch {
    mostrarError('No hay conexión con el servidor. Revisa tu internet y recarga la página.');
  }
}

async function iniciar() {
  const yo = await quienSoy();
  if (!yo.ok) {
    window.location.href = '/ingresar';
    return;
  }
  esAdmin = yo.datos.roles.includes('ADM');
  contexto.esAdmin = esAdmin;
  document.getElementById('cabecera-usuario').textContent = yo.datos.nombre + ' · ' + yo.datos.usuario;
  await recargar();
}

document.getElementById('boton-salir').addEventListener('click', async () => {
  try {
    await salir();
  } finally {
    window.location.href = '/ingresar';
  }
});

iniciar();
