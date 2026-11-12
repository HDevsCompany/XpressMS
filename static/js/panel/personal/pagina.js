// pagina.js: la página /panel/personal. Carga la lista y la vuelve a cargar después de cada cambio.

import { listarPersonal, quienSoy, salir } from '../api.js';
import { dibujarPersonal } from './lista.js';

const vista = document.getElementById('vista-personal');
const avisoError = document.getElementById('aviso-error');
let codigoPropio = null; // el código del usuario en sesión (para marcar "tú" y proteger su cuenta)

function mostrarError(texto) {
  avisoError.textContent = texto;
  avisoError.hidden = false;
}

async function recargar() {
  try {
    const resultado = await listarPersonal();
    if (!resultado.ok) {
      mostrarError('No se pudo cargar la lista. Recarga la página.');
      return;
    }
    dibujarPersonal(vista, resultado.datos, codigoPropio, mostrarError, recargar);
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
  codigoPropio = yo.datos.codigo;
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
