// guia.js: la guía animada de Juno. Oscurece la página y señala, paso a paso, dónde pulsar:
//   Paso 1: «Iniciar sesión» (en la barra superior).
//   Paso 2: dentro de la ventana de ingreso, según lo que la persona quería:
//           - "registro": «Regístrate con Juno» → "Ahora aquí, y yo me encargo del resto".
//           - "cliente":  el formulario de cliente → "Ingresa aquí tu documento...".
// Mientras dura, la página queda bloqueada: solo se puede pulsar lo señalado, seguir con «Omitir guía» o Escape.
// Al terminar llama a alTerminar("completada" o "cancelada").

const MARGEN = 12;
const ESPERA_TARJETA_MS = 650; // la tarjeta de ingreso tarda un momento en terminar su animación de entrada

let activa = null; // { elementos, paso, cual, objetivo, relleno, alTerminar }

function crear(clase, etiqueta = 'div') {
  const elemento = document.createElement(etiqueta);
  elemento.className = clase;
  return elemento;
}

// Un rectángulo sencillo con las medidas de la pantalla.
function rectangulo(r, desplazamientoX = 0, desplazamientoY = 0) {
  const left = r.left + desplazamientoX;
  const top = r.top + desplazamientoY;
  return { left, top, width: r.width, height: r.height, right: left + r.width, bottom: top + r.height };
}

// El elemento señalado en el paso 2 vive dentro del marco de la ventana de ingreso.
function elementoDelMarco(cual) {
  const marco = document.querySelector('.pub-ingreso iframe');
  const documento = marco?.contentDocument;
  const elemento = documento?.querySelector(cual === 'registro' ? '.ing-registro' : '#formulario-cliente');
  return { marco, elemento };
}

function colocar(rect, relleno, radio, lado) {
  const { foco, llamada, cursor } = activa.elementos;

  foco.style.left = rect.left - relleno + 'px';
  foco.style.top = rect.top - relleno + 'px';
  foco.style.width = rect.width + relleno * 2 + 'px';
  foco.style.height = rect.height + relleno * 2 + 'px';
  foco.style.borderRadius = radio;

  const ancho = llamada.offsetWidth;
  const alto = llamada.offsetHeight;
  let clase;
  let left;
  let top;
  if (lado === 'abajo') {
    clase = 'abajo';
    left = rect.right + relleno - ancho;
    top = rect.bottom + relleno + 22;
  } else if (rect.right + relleno + 26 + ancho <= window.innerWidth - MARGEN) {
    clase = 'derecha'; // a la derecha de lo señalado
    left = rect.right + relleno + 26;
    top = rect.top + rect.height / 2 - alto / 2;
  } else if (rect.top - relleno - alto - 18 > MARGEN) {
    clase = 'sin-flecha'; // no cabe al lado: arriba
    left = rect.left;
    top = rect.top - relleno - alto - 18;
  } else {
    clase = 'sin-flecha'; // o abajo
    left = rect.left;
    top = rect.bottom + relleno + 18;
  }
  llamada.classList.remove('abajo', 'derecha', 'sin-flecha');
  llamada.classList.add(clase);
  llamada.style.left = Math.min(Math.max(MARGEN, left), window.innerWidth - ancho - MARGEN) + 'px';
  llamada.style.top = Math.min(Math.max(MARGEN, top), window.innerHeight - alto - MARGEN) + 'px';

  cursor.style.left = rect.left + rect.width * 0.62 + 'px';
  cursor.style.top = rect.top + rect.height * 0.6 + 'px';
}

// Cambia el texto de la llamada (la ventanita blanca con Juno).
function escribirLlamada(titulo, texto, paso) {
  const { llamada } = activa.elementos;
  llamada.replaceChildren();

  const cuerpo = crear('guia-cuerpo');
  const imagen = document.createElement('img');
  imagen.src = '/static/img/juno.png';
  imagen.alt = '';
  const columna = document.createElement('div');
  const encabezado = document.createElement('b');
  encabezado.textContent = titulo;
  columna.append(encabezado);
  if (texto) {
    const parrafo = document.createElement('p');
    parrafo.textContent = texto;
    columna.append(parrafo);
  }
  cuerpo.append(imagen, columna);

  const pie = crear('guia-pie');
  const puntos = crear('guia-puntos');
  [1, 2].forEach((numero) => {
    const punto = document.createElement('i');
    if (numero === paso) punto.className = 'activo';
    puntos.append(punto);
  });
  const etiqueta = document.createElement('span');
  etiqueta.textContent = `Paso ${paso} de 2`;
  puntos.append(etiqueta);
  const omitir = crear('guia-omitir', 'button');
  omitir.type = 'button';
  omitir.textContent = 'Omitir guía';
  omitir.addEventListener('click', () => terminar('cancelada'));
  pie.append(puntos, omitir);

  llamada.append(cuerpo, pie);
}

// Paso 1: señala «Iniciar sesión».
function paso1() {
  const boton = document.getElementById('boton-ingresar');
  activa.paso = 1;
  activa.relleno = 7;
  activa.objetivo = () => rectangulo(boton.getBoundingClientRect());
  activa.activar = () => boton.click(); // abre la ventana de ingreso; el paso 2 empieza cuando está lista
  escribirLlamada('Presiona aquí', null, 1);
  colocar(activa.objetivo(), 7, '999px', 'abajo');
  activa.reubicar = () => colocar(activa.objetivo(), 7, '999px', 'abajo');
}

// Paso 2: señala lo que corresponde dentro de la ventana de ingreso.
function paso2() {
  if (!activa) return;
  const { cual } = activa;
  const { marco, elemento } = elementoDelMarco(cual);
  if (!marco || !elemento) {
    terminar('cancelada');
    return;
  }
  // En celulares la tarjeta puede estar más abajo: se trae a la vista antes de medir.
  elemento.scrollIntoView({ block: 'center' });

  const medir = () => {
    const marcoRect = marco.getBoundingClientRect();
    return rectangulo(elemento.getBoundingClientRect(), marcoRect.left, marcoRect.top);
  };
  const radio = cual === 'registro' ? '12px' : '16px';
  const relleno = cual === 'registro' ? 8 : 10;

  activa.paso = 2;
  activa.relleno = relleno;
  activa.objetivo = medir;
  activa.activar =
    cual === 'registro'
      ? () => {
          terminar('completada');
          elemento.click(); // «Regístrate con Juno →»: lleva al chat de registro
        }
      : () => {
          terminar('completada');
          marco.contentWindow.focus();
          elemento.querySelector('input, select')?.focus();
        };
  activa.reubicar = () => colocar(medir(), relleno, radio, 'derecha');

  if (cual === 'registro') {
    escribirLlamada('Ahora aquí', 'Y yo me encargo del resto: te voy preguntando todo por el chat.', 2);
  } else {
    escribirLlamada('Ingresa aquí', 'Tu documento y los últimos 3 dígitos de tu celular.', 2);
  }
  colocar(medir(), relleno, radio, 'derecha');
}

function alAbrirseLaVentana() {
  // Se espera a que la tarjeta termine de aparecer para medirla bien.
  setTimeout(paso2, ESPERA_TARJETA_MS);
}

function alCerrarseLaVentana() {
  if (activa && activa.paso === 2) terminar('cancelada');
}

function alTeclear(evento) {
  if (evento.key === 'Escape') {
    evento.stopImmediatePropagation(); // que Escape no cierre también la ventana de ingreso
    terminar('cancelada');
  } else if (evento.key === 'Enter' && activa) activa.activar();
}

function alHacerClic(evento) {
  if (!activa) return;
  const r = activa.objetivo();
  const dentro =
    evento.clientX >= r.left - activa.relleno &&
    evento.clientX <= r.right + activa.relleno &&
    evento.clientY >= r.top - activa.relleno &&
    evento.clientY <= r.bottom + activa.relleno;
  if (dentro) activa.activar();
}

function terminar(motivo) {
  if (!activa) return;
  const { elementos, alTerminar } = activa;
  Object.values(elementos).forEach((elemento) => elemento.remove());
  document.removeEventListener('ingreso:listo', alAbrirseLaVentana);
  document.removeEventListener('ingreso:cerrado', alCerrarseLaVentana);
  document.removeEventListener('keydown', alTeclear);
  window.removeEventListener('resize', alCambiarTamano);
  activa = null;
  alTerminar(motivo);
}

function alCambiarTamano() {
  if (activa) activa.reubicar();
}

// Inicia la guía. "cual" es "registro" o "cliente".
export function empezarGuia(cual, { alTerminar = () => {} } = {}) {
  if (activa) return;
  const boton = document.getElementById('boton-ingresar');
  if (!boton || boton.offsetParent === null) {
    alTerminar('cancelada'); // no hay botón que señalar (por ejemplo, ya hay sesión)
    return;
  }

  const elementos = {
    bloqueo: crear('guia-bloqueo'),
    foco: crear('guia-foco'),
    llamada: crear('guia-llamada'),
    cursor: crear('guia-cursor', 'div'),
  };
  elementos.llamada.setAttribute('role', 'dialog');
  elementos.llamada.setAttribute('aria-label', 'Guía de Juno');
  elementos.cursor.innerHTML =
    '<svg viewBox="0 0 24 24" style="width:34px;height:34px"><path d="M5 3l14 8-6 2 4 7-3 1.5-4-7-5 4z" fill="#fff" stroke="#0f172a" stroke-width="1.4" stroke-linejoin="round"/></svg>';
  elementos.bloqueo.addEventListener('click', alHacerClic);
  Object.values(elementos).forEach((elemento) => document.body.append(elemento));

  activa = { elementos, cual, alTerminar, paso: 1 };
  document.addEventListener('ingreso:listo', alAbrirseLaVentana);
  document.addEventListener('ingreso:cerrado', alCerrarseLaVentana);
  document.addEventListener('keydown', alTeclear);
  window.addEventListener('resize', alCambiarTamano);
  paso1();
}

