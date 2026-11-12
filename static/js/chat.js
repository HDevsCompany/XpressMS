// chat.js: el director de orquesta del lienzo de Juno (/ y /chat). No dibuja casi nada por sí mismo:
// coordina a los demás archivos (api, burbujas, componentes, menú) y mantiene el estado de la pantalla.
//
// La pantalla tiene dos vistas (atributo data-vista de <body>):
//   - "bienvenida":   Juno al centro, la caja para escribir y las píldoras. No hay conversación todavía.
//   - "conversacion": en cuanto la persona escribe o elige algo, aparecen las burbujas y la caja baja al pie.
// La conversación se crea en el servidor recién cuando la persona escribe por primera vez (o si ya traía una guardada
// en este navegador): quien solo entra a mirar no deja nada en la base de datos.

import { ErrorSesion, datosDeSesion, enviarMensaje, iniciar, nuevaConversacion } from './api.js';
import {
  borrarTema,
  borrarToken,
  guardarToken,
  leerToken,
  tomarDestino,
  tomarMensajePendiente,
  tomarNuevaConversacion,
} from './almacen.js';
import { agregarBurbuja, agregarSaludo, fijarAgente, limpiarOpciones, mostrarOpciones } from './burbujas.js';
import { mostrarCargando } from './componentes/cargando.js';
import { mostrarConsentimiento } from './componentes/consentimiento.js';
import { mostrarTarjetaApartado } from './componentes/tarjeta_apartado.js';
import { mostrarTarjetaDatos } from './componentes/tarjeta_datos.js';
import { mostrarTarjetaOrden } from './componentes/tarjeta_orden.js';
import { mostrarTarjetaProducto } from './componentes/tarjeta_producto.js';
import { mostrarTarjetaServicio } from './componentes/tarjeta_servicio.js';
import { pausa } from './dom.js';
import { cerrarMenu, iniciarMenu, mostrarCliente, mostrarVisitante } from './menu.js';
import { abrirIngreso } from './publico/ingreso.js';

const cuerpo = document.body;
const zonaMensajes = document.getElementById('mensajes');
const formulario = document.getElementById('formulario');
const campoTexto = document.getElementById('texto');
const botonEnviar = formulario.querySelector('button');
const titulo = document.getElementById('titulo');

const TITULO_VISITANTE = 'Hola, soy Juno';

let token = leerToken();
let conversacionLista = false; // true cuando el servidor ya abrió (o recuperó) la conversación de este navegador
let sesion = { reconocido: false, nombre: null }; // ¿hay un cliente reconocido?
let esperaSiguiente = null; // texto de espera que el servidor pidió para el próximo mensaje
let ocupado = false; // evita enviar dos mensajes a la vez

// Tiempo mínimo que se ve el aviso de espera (si la consulta fue instantánea, igual se nota la animación).
const MINIMO_ESPERA_MS = 1100;
const MINIMO_ESCRIBIENDO_MS = 450;

// ---------- Vista y sesión ----------

function fijarVista(vista) {
  cuerpo.dataset.vista = vista;
}

// En computadoras se deja el cursor en la caja; en celulares no, para que no salte el teclado sin que lo pidan.
function enfocarCaja() {
  if (window.matchMedia('(pointer: fine)').matches) campoTexto.focus({ preventScroll: true });
}

// Pone el saludo según haya o no un cliente reconocido, y el avatar o el botón «Iniciar sesión» del encabezado.
function mostrarSesion() {
  if (sesion.reconocido) {
    titulo.textContent = sesion.nombre ? `Hola, ${sesion.nombre}` : 'Hola de nuevo';
    mostrarCliente(sesion.nombre);
  } else {
    titulo.textContent = TITULO_VISITANTE;
    mostrarVisitante();
  }
}

async function actualizarSesion() {
  if (!token) return;
  sesion = await datosDeSesion(token);
  mostrarSesion();
}

// Vuelve a la bienvenida: borra lo que se veía de la conversación.
function volverALaBienvenida() {
  zonaMensajes.replaceChildren();
  fijarVista('bienvenida');
}

// ---------- Conversación ----------

// Bloquea o libera la barra de escritura mientras se espera una respuesta.
function bloquear(estado) {
  ocupado = estado;
  campoTexto.disabled = estado;
  botonEnviar.disabled = estado;
  if (!estado) campoTexto.focus({ preventScroll: true });
}

// Dibuja el elemento visual especial que pidió el servidor (si hay uno).
function mostrarComponente(componente) {
  if (!componente) return;
  if (componente.tipo === 'consentimiento') {
    mostrarConsentimiento({
      alAceptar: () => enviar('Acepto los términos y condiciones', 'aceptar_terminos'),
      alRechazar: () => enviar('No, gracias', 'rechazar_terminos'),
    });
  } else if (componente.tipo === 'identidad') {
    mostrarTarjetaDatos(componente);
    // Si la persona acaba de registrarse o de identificarse en el chat, el encabezado y el menú pasan a ser los de un cliente.
    if (!sesion.reconocido) actualizarSesion();
  } else if (componente.tipo === 'servicio') {
    mostrarTarjetaServicio(componente);
  } else if (componente.tipo === 'orden') {
    mostrarTarjetaOrden(componente);
  } else if (componente.tipo === 'producto') {
    mostrarTarjetaProducto(componente);
  } else if (componente.tipo === 'apartado') {
    mostrarTarjetaApartado(componente);
  }
}

// Muestra todo lo que el agente responde: texto, elemento visual y botones rápidos.
function mostrarSalida(salida) {
  limpiarOpciones();
  if (salida.respuesta) agregarBurbuja('agente', salida.respuesta);
  mostrarComponente(salida.componente);
  mostrarOpciones(salida.opciones, enviar);
  esperaSiguiente = salida.espera || null;
}

// Abre (o recupera) la conversación en el servidor y dibuja lo que ya tuviera. "motivo" es 'registro' cuando la persona
// llegó con «Regístrate con Juno»: el servidor deja el saludo con la casilla de términos.
async function prepararConversacion(motivo = null) {
  const datos = await iniciar(token, motivo);
  token = datos.token;
  guardarToken(token);
  conversacionLista = true;
  fijarAgente(datos.agente); // desde aquí, todos los mensajes del agente llevan su icono y su nombre

  // Una conversación sin mensajes propios (vacía, o con el saludo de versiones anteriores) se ve como la bienvenida.
  const vacia = datos.mensajes.length <= 1 && datos.opciones.length > 0 && !datos.componente;
  if (vacia) {
    volverALaBienvenida();
    return;
  }

  zonaMensajes.replaceChildren();
  fijarVista('conversacion');
  datos.mensajes.forEach((m, posicion) => {
    // El primer mensaje de la conversación es el saludo de la presentación: lleva el icono más grande.
    if (posicion === 0 && m.emisor === 'A') agregarSaludo(datos.agente, m.texto);
    else agregarBurbuja(m.emisor === 'A' ? 'agente' : 'persona', m.texto);
  });
  mostrarComponente(datos.componente);
  mostrarOpciones(datos.opciones, enviar);
}

// Envía un mensaje al servidor y muestra la respuesta. Desde la bienvenida, abre la conversación primero.
async function enviar(texto, accion = null) {
  if (ocupado) return;
  bloquear(true);
  cerrarMenu({ devolverFoco: false });

  try {
    if (!conversacionLista) {
      try {
        await prepararConversacion();
      } catch {
        fijarVista('conversacion');
        agregarBurbuja('agente', 'Ups, no pude conectarme. Intenta de nuevo en un momento.');
        return;
      }
    }
    fijarVista('conversacion');
    limpiarOpciones();
    agregarBurbuja('persona', texto);

    // Mientras se espera, mostramos el aviso de espera (si el servidor lo pidió) o los puntos de "escribiendo".
    const quitarCargando = mostrarCargando(esperaSiguiente);
    const minimo = esperaSiguiente ? MINIMO_ESPERA_MS : MINIMO_ESCRIBIENDO_MS;
    esperaSiguiente = null;
    const inicio = Date.now();

    try {
      const salida = await enviarMensaje(token, texto, accion);
      await pausa(Math.max(0, minimo - (Date.now() - inicio)));
      quitarCargando();
      mostrarSalida(salida);
    } catch (error) {
      quitarCargando();
      if (error instanceof ErrorSesion) {
        // El servidor ya no conoce esta conversación: se olvida, y el próximo mensaje abre una nueva.
        borrarToken();
        token = null;
        conversacionLista = false;
        agregarBurbuja('agente', 'Se perdió esta conversación. Escríbeme de nuevo y empezamos otra.');
      } else {
        agregarBurbuja('agente', 'Ups, no pude conectarme. Intenta de nuevo en un momento.');
      }
    }
  } finally {
    bloquear(false);
  }
}

// ---------- Cosas que pide el menú y las píldoras ----------

// «Nueva conversación». Un cliente reconocido empieza una limpia sin perder su identificación;
// un visitante simplemente olvida la suya (sus datos de registro, si los hubiera, no se borran de la base de datos).
async function nuevaConversacionPedida() {
  if (ocupado) return;
  if (!conversacionLista && cuerpo.dataset.vista === 'bienvenida') {
    enfocarCaja(); // ya está en una pantalla limpia
    return;
  }
  if (sesion.reconocido) {
    try {
      token = await nuevaConversacion(token);
      guardarToken(token);
    } catch {
      return; // sin conexión: se deja todo como estaba
    }
    conversacionLista = false;
    esperaSiguiente = null;
    volverALaBienvenida();
    try {
      await prepararConversacion(); // deja listo el agente y confirma que la conversación nueva existe
    } catch {
      /* se abrirá al escribir el primer mensaje */
    }
  } else {
    borrarToken();
    token = null;
    conversacionLista = false;
    esperaSiguiente = null;
    volverALaBienvenida();
  }
  enfocarCaja();
}

// «Cerrar sesión»: este navegador olvida al cliente (sus datos no se borran de la base de datos).
function cerrarSesion() {
  borrarToken();
  borrarTema();
  location.assign('/');
}

iniciarMenu({
  alElegir: (mensaje) => enviar(mensaje),
  alNueva: nuevaConversacionPedida,
  alIngresar: () => abrirIngreso({ lado: 'cliente' }),
  alSalir: cerrarSesion,
});

// Las píldoras de la bienvenida envían su mensaje, igual que las acciones del menú.
document.querySelectorAll('#pildoras [data-mensaje]').forEach((pildora) => {
  pildora.addEventListener('click', () => enviar(pildora.dataset.mensaje));
});

// Cuando la persona pulsa la flecha (o Enter).
formulario.addEventListener('submit', (evento) => {
  evento.preventDefault(); // evita que la página se recargue
  const texto = campoTexto.value.trim();
  if (texto === '') return;
  campoTexto.value = '';
  enviar(texto);
});

// ---------- Al abrir la pantalla ----------
// La tarjeta de ingreso manda por dónde llegó la persona (/chat?motivo=registro o =mirar): con «registro», Juno abre
// la conversación con la casilla de términos; con «mirar» (o sin nada) se ve la bienvenida.
async function arrancar() {
  const motivo = new URLSearchParams(location.search).get('motivo');
  if (motivo) history.replaceState(null, '', location.pathname); // la dirección vuelve a la normal (recargar no repite el saludo)

  mostrarSesion(); // visitante por defecto; se corrige enseguida si el navegador trae un cliente reconocido

  // Se abre la conversación ahora solo si ya había una guardada o si viene a registrarse.
  if (token || motivo === 'registro') {
    try {
      await prepararConversacion(motivo);
    } catch {
      conversacionLista = false; // el servidor no respondió: se ve la bienvenida y se reintenta al escribir
    }
  }
  await actualizarSesion();

  // Venía de otra pantalla (la de «Mi perfil y datos»): el destino, una conversación nueva o una petición para Juno.
  const destino = tomarDestino();
  if ((destino === '/perfil' || destino === '/catalogo' || destino === '/servicios') && sesion.reconocido) {
    location.replace(destino); // acababa de iniciar sesión desde el perfil, la tienda o los servicios: vuelve a esa página
    return;
  }
  if (tomarNuevaConversacion()) await nuevaConversacionPedida();
  const pendiente = tomarMensajePendiente();
  if (pendiente) {
    await enviar(pendiente);
    return;
  }
  if (cuerpo.dataset.vista === 'bienvenida') enfocarCaja();
}

arrancar();
