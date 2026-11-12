// pagina.js (productos): la página /panel/productos. Carga productos y categorías, maneja las pestañas
// y las vuelve a cargar después de cada cambio.

import { listarCategorias, listarProductos, quienSoy, salir } from '../api.js';
import { dibujarCategorias } from './lista_categorias.js';
import { dibujarProductos } from './lista_productos.js';

const vistaProductos = document.getElementById('vista-productos');
const vistaCategorias = document.getElementById('vista-categorias');
const pestanaProductos = document.getElementById('pestana-productos');
const pestanaCategorias = document.getElementById('pestana-categorias');
const avisoError = document.getElementById('aviso-error');

let temporizadorError = null;

function mostrarError(texto) {
  avisoError.textContent = texto;
  avisoError.hidden = false;
  clearTimeout(temporizadorError);
  temporizadorError = setTimeout(() => (avisoError.hidden = true), 8000);
}

// Cambia de pestaña: muestra una vista y oculta la otra.
function elegirPestana(nombre) {
  const enProductos = nombre === 'productos';
  vistaProductos.hidden = !enProductos;
  vistaCategorias.hidden = enProductos;
  pestanaProductos.setAttribute('aria-selected', String(enProductos));
  pestanaCategorias.setAttribute('aria-selected', String(!enProductos));
}
pestanaProductos.addEventListener('click', () => elegirPestana('productos'));
pestanaCategorias.addEventListener('click', () => elegirPestana('categorias'));

// Trae las dos listas del servidor y las dibuja. Se llama al abrir y después de cada cambio.
async function recargar() {
  try {
    const [categorias, productos] = await Promise.all([listarCategorias(), listarProductos()]);
    if (!categorias.ok || !productos.ok) {
      mostrarError('No se pudieron cargar los datos. Recarga la página.');
      return;
    }
    // Los formularios de producto usan la lista de categorías ({ codigo, nombre }).
    dibujarProductos(vistaProductos, productos.datos, categorias.datos, mostrarError, recargar);
    dibujarCategorias(vistaCategorias, categorias.datos, mostrarError, recargar);
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
