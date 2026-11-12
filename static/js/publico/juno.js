// juno.js: la burbuja flotante de Juno en la página principal.
//   - Botón redondo con un aviso («globo») que aparece una vez por visita.
//   - Al abrirlo, muestra el chat completo (/chat) dentro de un marco: es el mismo chat de siempre, con la misma conversación.
//   - Cuando el chat pide registrarse o ingresar, nos avisa con un mensaje (postMessage) y la página inicia la guía.

const CLAVE_GLOBO = 'xpressms_globo_visto';
const ESPERA_GLOBO_MS = 1500;

const boton = document.getElementById('juno-boton');
const globo = document.getElementById('juno-globo');
const panel = document.getElementById('juno-panel');
const marco = document.getElementById('juno-marco');

let cargado = false;

function globoYaVisto() {
  try {
    return sessionStorage.getItem(CLAVE_GLOBO) === '1';
  } catch {
    return false;
  }
}

function recordarGlobo() {
  try {
    sessionStorage.setItem(CLAVE_GLOBO, '1');
  } catch {
    /* sin almacenamiento: el aviso saldrá otra vez, no pasa nada */
  }
}

export function abrirJuno() {
  panel.hidden = false;
  boton.hidden = true;
  globo.hidden = true;
  boton.setAttribute('aria-expanded', 'true');
  recordarGlobo();
  // El chat se carga la primera vez que se abre (así la página principal carga más rápido).
  if (!cargado) {
    marco.src = '/chat';
    cargado = true;
  }
}

export function cerrarJuno() {
  panel.hidden = true;
  boton.hidden = false;
  boton.setAttribute('aria-expanded', 'false');
}

// "alPedirGuia(cual)" se llama cuando el chat pide ayuda para ingresar ("registro" o "cliente").
export function iniciarJuno({ alPedirGuia }) {
  boton.addEventListener('click', abrirJuno);
  document.getElementById('juno-cerrar').addEventListener('click', cerrarJuno);
  document.getElementById('juno-globo-cerrar').addEventListener('click', () => {
    globo.hidden = true;
    recordarGlobo();
  });
  document.querySelectorAll('[data-abrir-juno]').forEach((elemento) => elemento.addEventListener('click', abrirJuno));

  document.addEventListener('keydown', (evento) => {
    if (evento.key === 'Escape' && !panel.hidden && !document.querySelector('.pub-velo, .guia-bloqueo')) cerrarJuno();
  });

  // Mensajes del chat que va dentro del marco. Solo se aceptan los de esta misma página web (mismo origen).
  window.addEventListener('message', (evento) => {
    if (evento.origin !== window.location.origin) return;
    const datos = evento.data;
    if (!datos || datos.origen !== 'xpressms-chat' || datos.tipo !== 'guia') return;
    if (datos.cual === 'registro' || datos.cual === 'cliente') alPedirGuia(datos.cual);
  });

  // El aviso aparece una sola vez por visita, un momento después de cargar.
  if (!globoYaVisto()) {
    setTimeout(() => {
      if (panel.hidden && !globoYaVisto()) globo.hidden = false;
    }, ESPERA_GLOBO_MS);
  }
}
