// modal_terminos.js: la ventana animada con los Términos y Condiciones.
// El texto legal NO está en el código: vive en static/legal/terminos.html, para que quien lo
// revise (por ejemplo un asesor legal) lo edite sin tocar JavaScript.

import { crear } from '../dom.js';

const RUTA_TEXTO = '/static/legal/terminos.html';
let fondo = null; // la ventana se crea la primera vez que se abre
let cuerpo = null;
let textoCargado = false;
let elementoPrevio = null; // para devolver el foco a donde estaba al cerrar

// Construye la ventana (una sola vez) y la agrega a la página.
function construir() {
  fondo = crear('div', 'modal-fondo');
  const ventana = crear('div', 'modal');
  ventana.setAttribute('role', 'dialog');
  ventana.setAttribute('aria-modal', 'true');
  ventana.setAttribute('aria-labelledby', 'modal-titulo');

  const cabecera = crear('div', 'modal-cabecera');
  const titulo = crear('h2', '', 'Términos y Condiciones');
  titulo.id = 'modal-titulo';
  const cerrar = crear('button', 'modal-cerrar', '✕');
  cerrar.type = 'button';
  cerrar.setAttribute('aria-label', 'Cerrar');
  cerrar.addEventListener('click', cerrarTerminos);
  cabecera.append(titulo, cerrar);

  cuerpo = crear('div', 'modal-cuerpo', 'Cargando…');

  const pie = crear('div', 'modal-pie');
  const entendido = crear('button', 'boton-primario', 'Entendido');
  entendido.type = 'button';
  entendido.addEventListener('click', cerrarTerminos);
  pie.appendChild(entendido);

  ventana.append(cabecera, cuerpo, pie);
  fondo.appendChild(ventana);

  // Cerrar al hacer clic fuera de la ventana, o con la tecla Escape.
  fondo.addEventListener('click', (evento) => {
    if (evento.target === fondo) cerrarTerminos();
  });
  document.addEventListener('keydown', (evento) => {
    if (evento.key === 'Escape' && fondo.classList.contains('abierto')) cerrarTerminos();
  });

  document.body.appendChild(fondo);
}

// Trae el texto legal del servidor (solo la primera vez).
async function cargarTexto() {
  if (textoCargado) return;
  try {
    const respuesta = await fetch(RUTA_TEXTO);
    if (!respuesta.ok) throw new Error('Error ' + respuesta.status);
    // Es un archivo propio del proyecto (no texto de usuarios), por eso puede insertarse como HTML.
    cuerpo.innerHTML = await respuesta.text();
    textoCargado = true;
  } catch (error) {
    cuerpo.textContent = 'No se pudo cargar el texto. Intenta de nuevo en un momento.';
  }
}

export function abrirTerminos() {
  if (!fondo) construir();
  elementoPrevio = document.activeElement;
  // El pequeño retraso permite que el navegador anime la aparición.
  requestAnimationFrame(() => fondo.classList.add('abierto'));
  fondo.querySelector('.modal-cerrar').focus();
  cargarTexto();
}

export function cerrarTerminos() {
  if (!fondo) return;
  fondo.classList.remove('abierto');
  if (elementoPrevio) elementoPrevio.focus();
}
