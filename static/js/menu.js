// menu.js: el encabezado y el menú lateral del lienzo.
//   - El botón ☰ y, si hay un cliente reconocido, su avatar abren y cierran el menú.
//   - El velo (lo oscuro de detrás), la tecla Escape y la X también lo cierran.
//   - Las acciones rápidas del menú NO hacen nada por sí solas: avisan a chat.js (alElegir) con el mensaje que deben
//     enviarle a Juno. Así la interfaz solo muestra y Juno ejecuta.
//   - Cambia lo que se ve según haya o no un cliente reconocido (visitante: «Iniciar sesión»; cliente: avatar y su nombre).
// No habla con el servidor.

const cuerpo = document.body;
const menu = document.getElementById('menu');
const botonMenu = document.getElementById('boton-menu');
const botonAvatar = document.getElementById('boton-avatar');
const botonCerrar = document.getElementById('cerrar-menu');
const velo = document.getElementById('velo-menu');

let abridor = null; // el botón que abrió el menú, para devolverle el foco al cerrar

export function menuAbierto() {
  return cuerpo.classList.contains('lz-menu-abierto');
}

export function abrirMenu(desde = botonMenu) {
  if (menuAbierto()) return;
  abridor = desde;
  cuerpo.classList.add('lz-menu-abierto');
  menu.inert = false;
  botonMenu.setAttribute('aria-expanded', 'true');
  botonAvatar.setAttribute('aria-expanded', 'true');
  botonCerrar.focus({ preventScroll: true });
}

export function cerrarMenu({ devolverFoco = true } = {}) {
  if (!menuAbierto()) return;
  cuerpo.classList.remove('lz-menu-abierto');
  menu.inert = true;
  botonMenu.setAttribute('aria-expanded', 'false');
  botonAvatar.setAttribute('aria-expanded', 'false');
  if (devolverFoco && abridor && typeof abridor.focus === 'function' && abridor.offsetParent !== null) {
    abridor.focus({ preventScroll: true });
  }
  abridor = null;
}

function alternar(desde) {
  if (menuAbierto()) cerrarMenu();
  else abrirMenu(desde);
}

// ---------- Quién está en la pantalla ----------

// Visitante: botón «Iniciar sesión» arriba y la invitación a registrarse al pie del menú.
export function mostrarVisitante() {
  cuerpo.dataset.sesion = 'visitante';
}

// Cliente reconocido: su avatar (con la inicial) arriba y su nombre al pie del menú.
export function mostrarCliente(nombre) {
  const limpio = (nombre || '').trim();
  const inicial = limpio ? limpio.charAt(0).toUpperCase() : 'C';
  document.getElementById('avatar-inicial').textContent = inicial;
  document.getElementById('menu-inicial').textContent = inicial;
  document.getElementById('menu-nombre').textContent = limpio || 'Cliente';
  cuerpo.dataset.sesion = 'cliente';
}

// ---------- Conexión con el resto de la pantalla ----------
// alElegir(mensaje): una acción rápida (o píldora) pidió enviarle ese mensaje a Juno.
// alNueva(): «Nueva conversación».   alIngresar(): «Iniciar sesión».   alSalir(): «Cerrar sesión».
export function iniciarMenu({ alElegir, alNueva, alIngresar, alSalir }) {
  botonMenu.addEventListener('click', () => alternar(botonMenu));
  botonAvatar.addEventListener('click', () => alternar(botonAvatar));
  botonCerrar.addEventListener('click', () => cerrarMenu());
  velo.addEventListener('click', () => cerrarMenu());

  document.addEventListener('keydown', (evento) => {
    // Con la ventana de ingreso abierta, Escape es para ella (la maneja static/js/publico/ingreso.js).
    if (evento.key === 'Escape' && menuAbierto() && !document.querySelector('.pub-velo')) cerrarMenu();
  });

  // Las acciones rápidas del menú: cierran el menú y piden enviar su mensaje.
  menu.querySelectorAll('[data-mensaje]').forEach((accion) => {
    accion.addEventListener('click', () => {
      cerrarMenu({ devolverFoco: false });
      alElegir(accion.dataset.mensaje);
    });
  });

  document.getElementById('menu-nueva').addEventListener('click', () => {
    cerrarMenu({ devolverFoco: false });
    alNueva();
  });

  const ingresar = () => {
    cerrarMenu({ devolverFoco: false });
    alIngresar();
  };
  document.getElementById('menu-ingresar').addEventListener('click', ingresar);
  document.getElementById('boton-ingresar').addEventListener('click', ingresar);

  document.getElementById('menu-salir').addEventListener('click', alSalir);
}
