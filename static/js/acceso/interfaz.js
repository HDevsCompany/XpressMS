// interfaz.js: lo visual de la página de ingreso.
//   - Cambia entre el lado de "equipo" y el de "cliente" (panel deslizante en pantallas anchas, pestañas en celulares).
//   - El lado que no se ve queda "inert": no se puede enfocar con Tab ni leer con un lector de pantalla.
//   - El botón "ojo" muestra u oculta la contraseña.
// En /ingreso (la ventana de «Iniciar sesión» de la página principal) se ve primero el lado de cliente; en /ingresar, el de equipo.
// Las direcciones con #cliente o #equipo abren directamente ese lado.

const tarjeta = document.getElementById('tarjeta');
const panelCliente = document.getElementById('panel-cliente');
const panelEquipo = document.getElementById('panel-equipo');
const pestanas = document.querySelectorAll('.ing-pestanas [data-ir]');
const capaAzul = document.querySelector('.ing-cap.azul');
const capaVioleta = document.querySelector('.ing-cap.vio');

const reducirMovimiento = () => window.matchMedia('(prefers-reduced-motion: reduce)').matches;
const DURACION_MS = 700; // lo que dura el deslizamiento (igual que en acceso.css)

// El lado que se ve al abrir la página, según la dirección.
const LADO_PREDETERMINADO = location.pathname === '/ingresar' ? 'equipo' : 'cliente';
function ladoDeLaDireccion() {
  if (location.hash === '#cliente') return 'cliente';
  if (location.hash === '#equipo') return 'equipo';
  return LADO_PREDETERMINADO;
}

// Muestra un lado ("equipo" o "cliente"). Con enfocar = true, lleva el cursor al primer campo.
function ir(lado, { enfocar = true } = {}) {
  tarjeta.dataset.lado = lado;

  // Lo que no se ve no debe poder usarse con el teclado.
  panelCliente.inert = lado !== 'cliente';
  panelEquipo.inert = lado !== 'equipo';
  if (capaAzul) capaAzul.inert = lado !== 'equipo';
  if (capaVioleta) capaVioleta.inert = lado !== 'cliente';

  pestanas.forEach((pestana) => {
    const activa = pestana.dataset.ir === lado;
    pestana.setAttribute('aria-selected', String(activa));
    pestana.tabIndex = activa ? 0 : -1;
  });

  // La dirección recuerda el lado elegido (así se puede enviar o recargar). El lado predeterminado no lleva "#".
  const direccion = lado === LADO_PREDETERMINADO ? location.pathname + location.search : '#' + lado;
  history.replaceState(null, '', direccion);

  if (enfocar) {
    const primero = (lado === 'cliente' ? panelCliente : panelEquipo).querySelector('input, select');
    // Se espera a que termine el deslizamiento para no mover la pantalla a media animación.
    setTimeout(() => primero?.focus(), reducirMovimiento() ? 0 : DURACION_MS);
  }
}

document.querySelectorAll('[data-ir]').forEach((boton) => {
  boton.addEventListener('click', () => ir(boton.dataset.ir));
});

// Al abrir: se muestra el lado indicado en la dirección, sin animar el panel.
ir(ladoDeLaDireccion(), { enfocar: false });
requestAnimationFrame(() => requestAnimationFrame(() => tarjeta.classList.remove('ing-sin-animar')));

window.addEventListener('hashchange', () => ir(ladoDeLaDireccion(), { enfocar: false }));

// Ojo de la contraseña.
document.querySelectorAll('[data-mostrar]').forEach((boton) => {
  const campo = document.getElementById(boton.dataset.mostrar);
  boton.addEventListener('click', () => {
    const visible = campo.type === 'text';
    campo.type = visible ? 'password' : 'text';
    boton.setAttribute('aria-pressed', String(!visible));
    boton.setAttribute('aria-label', visible ? 'Mostrar contraseña' : 'Ocultar contraseña');
  });
});
