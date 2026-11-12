// ingreso.js: la ventana de «Iniciar sesión» de la página principal.
// Muestra la tarjeta de ingreso de siempre (/ingreso: cliente, registro con Juno y portal de equipo) dentro de un marco,
// sobre la página oscurecida. No dibuja el formulario: así sigue siendo exactamente la misma tarjeta.
// Avisa con eventos para que otras piezas (la guía de Juno) sepan cuándo está lista o se cerró:
//   - "ingreso:listo"   (detail.marco: el iframe ya cargó)
//   - "ingreso:cerrado"

let velo = null;
let anterior = null; // lo que tenía el foco antes de abrir, para devolvérselo al cerrar

function alTeclear(evento) {
  // Mientras la guía de Juno está activa, Escape solo la cancela a ella (lo ve guia.js); la ventana sigue abierta.
  if (evento.key === 'Escape' && !document.querySelector('.guia-bloqueo')) cerrarIngreso();
}

// Abre la ventana. "lado" es "cliente" (lo normal) o "equipo".
export function abrirIngreso({ lado = 'cliente' } = {}) {
  if (velo) return;
  anterior = document.activeElement;

  velo = document.createElement('div');
  velo.className = 'pub-velo';
  velo.setAttribute('role', 'dialog');
  velo.setAttribute('aria-modal', 'true');
  velo.setAttribute('aria-label', 'Iniciar sesión');

  const caja = document.createElement('div');
  caja.className = 'pub-ingreso';

  const cerrar = document.createElement('button');
  cerrar.type = 'button';
  cerrar.className = 'pub-ingreso-cerrar';
  cerrar.setAttribute('aria-label', 'Cerrar');
  cerrar.textContent = '×';
  cerrar.addEventListener('click', cerrarIngreso);

  const marco = document.createElement('iframe');
  marco.title = 'Iniciar sesión o registrarse';
  marco.addEventListener('load', () => {
    // Si el foco está dentro del marco, Escape no llega a esta página: se escucha también ahí.
    try {
      marco.contentDocument.addEventListener('keydown', alTeclear);
    } catch {
      /* mismo origen: no debería fallar */
    }
    document.dispatchEvent(new CustomEvent('ingreso:listo', { detail: { marco } }));
  });
  marco.src = '/ingreso#' + lado;

  caja.append(cerrar, marco);
  velo.append(caja);
  // Un clic fuera de la tarjeta (en lo oscuro) cierra la ventana.
  velo.addEventListener('mousedown', (evento) => {
    if (evento.target === velo) cerrarIngreso();
  });

  document.body.append(velo);
  document.body.classList.add('sin-desplazar');
  document.addEventListener('keydown', alTeclear);
}

export function cerrarIngreso() {
  if (!velo) return;
  velo.remove();
  velo = null;
  document.body.classList.remove('sin-desplazar');
  document.removeEventListener('keydown', alTeclear);
  if (anterior && typeof anterior.focus === 'function') anterior.focus();
  document.dispatchEvent(new CustomEvent('ingreso:cerrado'));
}

export function ingresoAbierto() {
  return velo !== null;
}
