// juno_panel.js (panel/servicios): la conversación con Juno. Solo dibuja el chat y se lo pasa al servidor; Juno (el servidor) es quien
// entiende, arma la ficha y ejecuta. La interfaz muestra, Juno ejecuta.
//
// El «estado» de la conversación (qué ficha se arma, qué se pregunta) lo manda el servidor y este archivo se lo devuelve en cada mensaje.
// Se guarda en sessionStorage para que recargar la página no pierda la ficha a medias.

import { crear } from '../../dom.js';
import { conversarJunoServicios } from '../api.js';
import { icono } from './iconos.js';

const FOTO_JUNO = '/static/img/juno.png';
const CLAVE = 'juno_servicios_v1';

// Convierte "texto **resaltado** texto" en nodos: lo resaltado va en <b class="rs">. Nunca se usa innerHTML.
export function conResaltado(texto, destino) {
  texto.split('**').forEach((trozo, i) => {
    if (!trozo) return;
    if (i % 2 === 1) destino.append(crear('b', 'rs', trozo));
    else destino.append(document.createTextNode(trozo));
  });
  return destino;
}

function leerGuardado() {
  try {
    const crudo = sessionStorage.getItem(CLAVE);
    return crudo ? JSON.parse(crudo) : null;
  } catch {
    return null;
  }
}

function guardar(datos) {
  try {
    sessionStorage.setItem(CLAVE, JSON.stringify(datos));
  } catch {
    /* sin almacenamiento: la conversación sigue, solo no sobrevive a recargar */
  }
}

// "raiz" es el <aside> de Juno. "alturno(salida)" se llama con cada respuesta (ficha, avisos de lo hecho…).
export function crearJuno(raiz, { alTurno, alAbrir }) {
  const zona = raiz.querySelector('.pj-msgs');
  const formulario = raiz.querySelector('.pj-caja');
  const campo = formulario.querySelector('input');
  const enviarBoton = formulario.querySelector('button');

  let estado = {};
  let mensajes = []; // lo que se ve: { quien: 'p'|'a', texto } y, al final, las opciones { ops: [...] }
  let ocupado = false;
  let ultimoBorrador = null;

  function persistir() {
    guardar({ estado, mensajes, borrador: ultimoBorrador });
  }

  function bajar() {
    zona.scrollTop = zona.scrollHeight;
  }

  function burbuja(quien, texto) {
    if (quien === 'p') {
      zona.append(crear('div', 'pj-b p', texto));
    } else {
      const fila = crear('div', 'pj-fila-a');
      const foto = crear('img');
      foto.src = FOTO_JUNO;
      foto.alt = '';
      const cuerpo = crear('div', 'pj-b a');
      conResaltado(texto, cuerpo);
      fila.append(foto, cuerpo);
      zona.append(fila);
    }
  }

  function dibujarOpciones(opciones) {
    quitarOpciones();
    if (!opciones || opciones.length === 0) return;
    const fila = crear('div', 'pj-ops');
    for (const opcion of opciones) {
      const boton = crear('button', opcion.tono || '', opcion.texto);
      boton.type = 'button';
      boton.addEventListener('click', () => enviar({ texto: opcion.texto, accion: opcion.accion || '' }));
      fila.append(boton);
    }
    zona.append(fila);
  }

  function quitarOpciones() {
    zona.querySelectorAll('.pj-ops').forEach((fila) => fila.remove());
  }

  function escribiendo(visible) {
    zona.querySelector('.pj-escribiendo-fila')?.remove();
    if (!visible) return;
    const fila = crear('div', 'pj-fila-a pj-escribiendo-fila');
    const foto = crear('img');
    foto.src = FOTO_JUNO;
    foto.alt = '';
    const puntos = crear('div', 'pj-b a pj-escribiendo');
    puntos.append(crear('i'), crear('i'), crear('i'));
    fila.append(foto, puntos);
    zona.append(fila);
  }

  function redibujar() {
    zona.replaceChildren();
    for (const m of mensajes) {
      if (m.ops) dibujarOpciones(m.ops);
      else burbuja(m.quien, m.texto);
    }
    bajar();
  }

  function activar(libre) {
    ocupado = !libre;
    campo.disabled = !libre;
    enviarBoton.disabled = !libre;
    zona.querySelectorAll('.pj-ops button').forEach((b) => (b.disabled = !libre));
  }

  // Envía algo a Juno. "texto" es lo que se ve como si lo hubiera escrito la persona (vacío = no se muestra);
  // "accion" es lo que se manda a ejecutar cuando viene de un botón.
  async function enviar({ texto = '', accion = '', mostrar = true }) {
    if (ocupado) return;
    if (mostrar) alAbrir?.(); // al saludar o hacer ajustes en silencio, el chat no se abre solo en el teléfono
    quitarOpciones();
    mensajes = mensajes.filter((m) => !m.ops);
    if (texto && mostrar) {
      mensajes.push({ quien: 'p', texto });
      burbuja('p', texto);
    }
    activar(false);
    escribiendo(true);
    bajar();

    let salida = null;
    try {
      const respuesta = await conversarJunoServicios({ texto, accion: accion || null, estado });
      if (respuesta.ok && respuesta.datos) salida = respuesta.datos;
    } catch {
      salida = null;
    }
    escribiendo(false);

    if (!salida) {
      const aviso = 'No pude responderte ahora. Revisa tu conexión e inténtalo de nuevo.';
      mensajes.push({ quien: 'a', texto: aviso });
      burbuja('a', aviso);
      activar(true);
      persistir();
      bajar();
      return;
    }

    estado = salida.estado || {};
    ultimoBorrador = salida.borrador || null;
    for (const parrafo of String(salida.respuesta || '').split('\n\n')) {
      if (!parrafo.trim()) continue;
      mensajes.push({ quien: 'a', texto: parrafo });
      burbuja('a', parrafo);
    }
    if (salida.opciones && salida.opciones.length > 0) {
      mensajes.push({ ops: salida.opciones });
      dibujarOpciones(salida.opciones);
    }
    activar(true);
    persistir();
    bajar();
    alTurno(salida);
    if (!matchMedia('(max-width: 900px)').matches) campo.focus({ preventScroll: true });
  }

  formulario.addEventListener('submit', (evento) => {
    evento.preventDefault();
    const texto = campo.value.trim();
    if (!texto) return;
    campo.value = '';
    enviar({ texto });
  });

  enviarBoton.replaceChildren(icono('enviar', 2.4));

  return {
    enviar,
    estado: () => estado,
    // Cambia el estado guardado sin pasar por Juno (por ejemplo, si se perdió la imagen elegida al recargar).
    ajustarEstado(cambio) {
      estado = cambio(estado);
      persistir();
    },
    // Empieza: recupera la conversación guardada o saluda.
    iniciar() {
      const guardado = leerGuardado();
      if (guardado && Array.isArray(guardado.mensajes) && guardado.mensajes.length > 0) {
        estado = guardado.estado || {};
        mensajes = guardado.mensajes;
        ultimoBorrador = guardado.borrador || null;
        redibujar();
        return { borrador: ultimoBorrador };
      }
      enviar({ accion: 'inicio', mostrar: false });
      return { borrador: null };
    },
  };
}
