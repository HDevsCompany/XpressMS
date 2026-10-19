// chat.js: el director de orquesta del chat. No dibuja nada por sí mismo:
// coordina a los demás archivos (api, burbujas, componentes) y mantiene el estado de la conversación.

import { ErrorSesion, enviarMensaje, iniciar } from './api.js';
import { borrarToken, guardarToken, leerToken } from './almacen.js';
import { agregarBurbuja, limpiarOpciones, mostrarOpciones } from './burbujas.js';
import { mostrarCargando } from './componentes/cargando.js';
import { mostrarConsentimiento } from './componentes/consentimiento.js';
import { mostrarTarjetaDatos } from './componentes/tarjeta_datos.js';
import { mostrarTarjetaServicio } from './componentes/tarjeta_servicio.js';
import { pausa } from './dom.js';

const formulario = document.getElementById('formulario');
const campoTexto = document.getElementById('texto');
const botonEnviar = formulario.querySelector('button');

let token = leerToken();
let esperaSiguiente = null; // texto de espera que el servidor pidió para el próximo mensaje
let ocupado = false; // evita enviar dos mensajes a la vez

// Tiempo mínimo que se ve el aviso de espera (si la consulta fue instantánea, igual se nota la animación).
const MINIMO_ESPERA_MS = 1100;
const MINIMO_ESCRIBIENDO_MS = 450;

// Bloquea o libera la barra de escritura mientras se espera una respuesta.
function bloquear(estado) {
  ocupado = estado;
  campoTexto.disabled = estado;
  botonEnviar.disabled = estado;
  if (!estado) campoTexto.focus();
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
  } else if (componente.tipo === 'servicio') {
    mostrarTarjetaServicio(componente);
  }
}

// Muestra todo lo que el agente responde: texto, elemento visual y botones rápidos.
function mostrarSalida(salida) {
  limpiarOpciones();
  if (salida.respuesta) agregarBurbuja('agente', salida.respuesta);
  mostrarComponente(salida.componente);
  mostrarOpciones(salida.opciones, (texto) => enviar(texto));
  esperaSiguiente = salida.espera || null;
}

// Envía un mensaje al servidor y muestra la respuesta.
async function enviar(texto, accion = null) {
  if (ocupado) return;
  bloquear(true);
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
      borrarToken();
      agregarBurbuja('agente', 'Se perdió tu sesión. Recarga la página para empezar de nuevo.');
    } else {
      agregarBurbuja('agente', 'Ups, no pude conectarme. Intenta de nuevo en un momento.');
    }
  } finally {
    bloquear(false);
  }
}

// Cuando la persona pulsa "Enviar" (o Enter).
formulario.addEventListener('submit', (evento) => {
  evento.preventDefault(); // evita que la página se recargue
  const texto = campoTexto.value.trim();
  if (texto === '') return;
  campoTexto.value = '';
  enviar(texto);
});

// Botón "Nueva conversación": olvida el token guardado y recarga la página.
// El servidor crea entonces una conversación nueva. Los datos ya registrados NO se borran de la base de datos:
// si la persona vuelve a escribir su documento, tendrá que verificar su identidad como cualquier visita.
document.getElementById('nueva-conversacion').addEventListener('click', () => {
  borrarToken();
  location.reload();
});

// Al abrir la página: iniciamos (o recuperamos) la conversación y dibujamos el historial.
async function arrancar() {
  try {
    const datos = await iniciar(token);
    token = datos.token;
    guardarToken(token);
    datos.mensajes.forEach((m) => agregarBurbuja(m.emisor === 'A' ? 'agente' : 'persona', m.texto));
    mostrarComponente(datos.componente);
    mostrarOpciones(datos.opciones, (texto) => enviar(texto));
  } catch (error) {
    agregarBurbuja('agente', 'Ups, no pude conectarme. Recarga la página.');
  }
}

arrancar();
