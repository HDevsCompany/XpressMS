// servicios.js: la página /servicios (los servicios técnicos). Solo MUESTRA: pide los servicios al servidor y los dibuja.
// Quien solicita el servicio, anota lo que le pasa al equipo y agenda la cita es Juno, la burbuja flotante (tienda/juno.js). Los botones
// de la página («Solicitar con Juno», «Agendar con Juno», «Solicitar servicio»…) no registran nada por su cuenta: le piden a Juno que lo haga,
// y lo que Juno va anotando se ve en el cajón «Tu solicitud» (componentes/solicitud.js).
// La página es pública: se ve igual con o sin sesión (la sesión solo cambia el encabezado y el menú).

import { datosDeSesion, nuevaConversacion } from './api.js';
import {
  borrarTema,
  borrarToken,
  dejarDestino,
  dejarMensajePendiente,
  dejarNuevaConversacion,
  guardarToken,
  leerToken,
  leerTokenServicios,
} from './almacen.js';
import { marcarElegido, mostrarServicios } from './componentes/servicios.js';
import { iniciarSolicitud } from './componentes/solicitud.js';
import { iniciarMenu, mostrarCliente, mostrarVisitante } from './menu.js';
import { abrirIngreso } from './publico/ingreso.js';
import { iniciarJuno } from './tienda/juno.js';

const grilla = document.getElementById('servicios-grid');
const estado = document.getElementById('servicios-estado');
const aviso = document.getElementById('servicios-aviso');
const reintentar = document.getElementById('servicios-reintentar');
const resumen = document.getElementById('servicios-resumen');
const botonSolicitar = document.getElementById('sv-solicitar');
const botonDiagnostico = document.getElementById('sv-diagnostico');
const textoDiagnostico = document.getElementById('sv-diag-texto');
const pista = document.getElementById('sv-hint');

const ORDEN_SOLICITAR = 'Solicitar un servicio técnico';
const ORDEN_DIAGNOSTICO = 'Agendar cita de diagnóstico';

// ---------- Juno y «Tu solicitud» ----------

// Los botones de la página no registran nada: se lo piden a Juno, que abre su ventana y lo hace a la vista de la persona.
const acciones = {
  solicitar: (servicio) => juno.pedir(`Solicitar servicio: ${servicio.nombre}`),
  preguntar: (servicio) => juno.pedir(`Tengo una duda sobre el servicio: ${servicio.nombre}`),
  diagnostico: () => juno.pedir(ORDEN_DIAGNOSTICO),
};

const juno = iniciarJuno({
  motivo: 'servicios',
  alIdentificar: () => mostrarSesion(),
  alSolicitud: (solicitud) => {
    // La tarjeta del servicio que Juno está atendiendo se marca en la página.
    if (solicitud.fase === 'fin' || solicitud.fase === 'listo') marcarElegido(null);
    else if (solicitud.servicio && solicitud.modo === 'servicio') marcarElegido(solicitud.servicio);
    else marcarElegido(null);
    cajon.actualizar(solicitud);
  },
});

const cajon = iniciarSolicitud({
  ordenar: (texto) => juno.ordenar(texto),
  abrirJuno: () => juno.abrir(),
  minimizarJuno: () => juno.minimizar(),
  alTerminar: () => marcarElegido(null),
});

botonSolicitar.addEventListener('click', () => juno.pedir(ORDEN_SOLICITAR));
botonDiagnostico.addEventListener('click', () => juno.pedir(ORDEN_DIAGNOSTICO));
document.getElementById('sv-hint-boton').addEventListener('click', () => juno.pedir(ORDEN_DIAGNOSTICO));

// ---------- La lista de servicios ----------

function mostrarAviso(titulo, texto, conReintento) {
  grilla.replaceChildren();
  estado.hidden = true;
  pista.hidden = true;
  resumen.textContent = '';
  document.getElementById('servicios-aviso-titulo').textContent = titulo;
  document.getElementById('servicios-aviso-texto').textContent = texto;
  reintentar.hidden = !conReintento;
  aviso.hidden = false;
}

// La tarjeta «Agendar cita de diagnóstico» dice lo que el negocio configuró para ese servicio (su descripción y su precio base).
function prepararDiagnostico(diagnostico) {
  if (!diagnostico) {
    botonDiagnostico.disabled = true;
    textoDiagnostico.textContent = 'La cita de diagnóstico no está disponible por ahora. Escríbele a Juno y te ayudamos.';
    return;
  }
  botonDiagnostico.disabled = false;
  const precio = diagnostico.precio ? ` Precio base: S/ ${diagnostico.precio}.` : '';
  if (diagnostico.descripcion) textoDiagnostico.textContent = `${diagnostico.descripcion}${precio}`;
  else if (precio) textoDiagnostico.textContent = `${textoDiagnostico.textContent}${precio}`;
}

function resumenDe(cantidad) {
  return cantidad === 1 ? '1 servicio' : `${cantidad} servicios`;
}

async function cargar() {
  aviso.hidden = true;
  estado.hidden = false;
  document.getElementById('servicios-estado-texto').textContent = 'Cargando los servicios…';
  resumen.textContent = 'Cargando…';
  grilla.replaceChildren();
  try {
    const respuesta = await fetch('/publico/servicios');
    if (!respuesta.ok) throw new Error('Error ' + respuesta.status);
    const pagina = await respuesta.json();
    prepararDiagnostico(pagina.diagnostico);
    if (pagina.servicios.length === 0) {
      botonSolicitar.disabled = true;
      mostrarAviso(
        'Aún no hay servicios publicados',
        pagina.diagnostico
          ? 'Estamos preparando la lista. Mientras tanto puedes agendar una cita de diagnóstico y lo revisamos en el local.'
          : 'Estamos preparando la lista. Vuelve pronto o escríbele a Juno en el chat.',
        false,
      );
      return;
    }
    botonSolicitar.disabled = false;
    mostrarServicios(grilla, pagina.servicios, acciones);
    resumen.textContent = resumenDe(pagina.servicios.length);
    document.getElementById('servicios-estado-texto').textContent = `Hay ${resumenDe(pagina.servicios.length)} para tu equipo.`;
    pista.hidden = !pagina.diagnostico;
    juno.ponerSugerencias([ORDEN_SOLICITAR, ORDEN_DIAGNOSTICO, 'Ver mi orden']);
    juno.recuperar(); // si había una solicitud a medias, el cajón vuelve a dibujarse
  } catch {
    mostrarAviso('No pude cargar los servicios', 'Revisa tu conexión e inténtalo de nuevo.', true);
  }
}

reintentar.addEventListener('click', cargar);

// ---------- Sesión, encabezado y menú ----------

// El encabezado y el menú según haya o no un cliente reconocido. Si la persona se registró o se identificó DENTRO de la ventana
// de Juno, su chat principal todavía no la conoce: se le abre uno nuevo (vacío) ya vinculado a ella, sin tocar nada de lo conversado.
async function mostrarSesion() {
  const principal = leerToken();
  let sesion = principal ? await datosDeSesion(principal) : { reconocido: false, nombre: null };
  if (!sesion.reconocido) {
    const deServicios = leerTokenServicios();
    const otra = deServicios ? await datosDeSesion(deServicios) : { reconocido: false, nombre: null };
    if (otra.reconocido) {
      try {
        guardarToken(await nuevaConversacion(deServicios));
      } catch {
        /* sin conexión: el chat principal la reconocerá cuando vuelva a identificarse */
      }
      sesion = otra;
    }
  }
  if (sesion.reconocido) mostrarCliente(sesion.nombre);
  else mostrarVisitante();
}

// Pide algo a Juno desde el menú: deja el mensaje y abre el chat, donde Juno lo atiende.
function pedirAJunoEnElChat(mensaje) {
  dejarMensajePendiente(mensaje);
  location.assign('/chat');
}

iniciarMenu({
  alElegir: pedirAJunoEnElChat,
  alNueva: () => {
    dejarNuevaConversacion();
    location.assign('/chat');
  },
  alIngresar: () => {
    dejarDestino('/servicios'); // la ventana de ingreso termina en el lienzo; desde ahí se vuelve a los servicios
    abrirIngreso({ lado: 'cliente' });
  },
  alSalir: () => {
    borrarToken();
    borrarTema();
    location.assign('/');
  },
});

// En el menú, esta página queda marcada.
const enlace = document.getElementById('menu-servicios');
if (enlace) enlace.setAttribute('aria-current', 'page');

mostrarVisitante(); // por defecto; se corrige enseguida si hay un cliente reconocido
mostrarSesion();
cargar();
