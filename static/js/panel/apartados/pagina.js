// pagina.js (apartados): la página /panel/apartados. Dos pestañas: los apartados pendientes (por atender)
// y el historial (entregados, cancelados y vencidos). El servidor libera solo los que vencieron.

import { listarApartados, quienSoy, salir } from '../api.js';
import { crear } from '../../dom.js';
import { dibujarApartados } from './lista.js';
import { PENDIENTE } from './formato.js';

const contenedorPestanas = document.getElementById('pestanas');
const vistaApartados = document.getElementById('vista-apartados');
const avisoError = document.getElementById('aviso-error');

const PESTANAS = [
  {
    id: 'pendientes',
    titulo: 'Pendientes',
    filtro: (apartado) => apartado.estado_codigo === PENDIENTE,
    vacio: 'No hay apartados pendientes. Cuando un cliente aparte un producto desde el asistente, aparecerá aquí.',
  },
  {
    id: 'historial',
    titulo: 'Historial',
    filtro: (apartado) => apartado.estado_codigo !== PENDIENTE,
    vacio: 'Todavía no hay apartados entregados, cancelados ni vencidos.',
  },
];

let pestanaActiva = 'pendientes';
let apartados = [];
let temporizadorError = null;

function mostrarError(texto) {
  avisoError.textContent = texto;
  avisoError.hidden = false;
  clearTimeout(temporizadorError);
  temporizadorError = setTimeout(() => (avisoError.hidden = true), 8000);
}

const contexto = { mostrarError, recargar };

// Dibuja las pestañas (con cuántos apartados tiene cada una) y la lista de la activa.
function dibujar() {
  contenedorPestanas.replaceChildren();
  for (const pestana of PESTANAS) {
    const cantidad = apartados.filter(pestana.filtro).length;
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
  dibujarApartados(vistaApartados, apartados.filter(activa.filtro), activa.vacio, contexto);
}

// Trae los apartados del servidor y los dibuja. Se llama al abrir y después de cada cambio.
async function recargar() {
  try {
    const resultado = await listarApartados();
    if (!resultado.ok) {
      mostrarError('No se pudieron cargar los apartados. Recarga la página.');
      return;
    }
    apartados = resultado.datos;
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
