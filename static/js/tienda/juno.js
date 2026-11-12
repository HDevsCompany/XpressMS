// juno.js: Juno en la tienda (/catalogo), como una burbuja flotante abajo a la derecha que abre una ventana de chat.
//
// Es una conversación APARTE del chat principal: no lleva saludo guardado, no aparece en el historial del lienzo y empieza
// vacía (la ventana ya trae la presentación de Juno). Se abre solo cuando la persona la toca; nunca manda mensajes por su cuenta.
// Juno es quien filtra y aparta: lo que la persona le pide aquí se envía al mismo agente de siempre (con el motivo «catalogo»,
// que hace que «laptops» o «menos de 500» filtren la vitrina) y cuando el agente devuelve una vitrina nueva, "alCatalogo" la dibuja.
// El carrito también es de Juno: cuando ella lo cambia, manda el carrito ya calculado y "alCarrito" lo dibuja (botón del encabezado,
// cajón «Mi carrito»); cuando aparta el pedido, "alPedido" muestra la confirmación con el código de retiro.
// La misma ventana la usa la página de servicios (/servicios) con el motivo «servicios»: su conversación también es aparte, y cada respuesta
// puede traer el estado de «Tu solicitud» ("alSolicitud") que la página dibuja en su cajón.
// Reutiliza las burbujas, tarjetas y botones del chat principal (burbujas.js y componentes/).

import { ErrorSesion, datosDeSesion, enviarMensaje, iniciar, nuevaConversacion } from '../api.js';
import { guardarTokenServicios, guardarTokenTienda, leerToken, leerTokenServicios, leerTokenTienda } from '../almacen.js';
import { agregarBurbuja, fijarAgente, limpiarOpciones, mostrarOpciones } from '../burbujas.js';
import { mostrarCargando } from '../componentes/cargando.js';
import { mostrarConsentimiento } from '../componentes/consentimiento.js';
import { mostrarTarjetaApartado } from '../componentes/tarjeta_apartado.js';
import { mostrarTarjetaDatos } from '../componentes/tarjeta_datos.js';
import { mostrarTarjetaOrden } from '../componentes/tarjeta_orden.js';
import { mostrarTarjetaProducto } from '../componentes/tarjeta_producto.js';
import { mostrarTarjetaServicio } from '../componentes/tarjeta_servicio.js';
import { pausa } from '../dom.js';

const MINIMO_ESPERA_MS = 1100;
const MINIMO_ESCRIBIENDO_MS = 450;

// "alCatalogo(vitrina)": dibuja la vitrina que Juno dejó. "alIdentificar()": la persona se registró o se identificó en la ventana.
// "alCarrito(carrito, texto)": Juno dejó el carrito al día (y dijo "texto"). "alPedido(pedido)": Juno apartó el pedido.
// "alVerCarrito()": la persona pulsó «Ver carrito» en el chat.
// "motivo": «catalogo» (la tienda, por omisión) o «servicios». "alSolicitud(solicitud, componente)": Juno dejó al día «Tu solicitud».
export function iniciarJuno({ motivo = 'catalogo', alCatalogo, alIdentificar, alCarrito, alPedido, alVerCarrito, alSolicitud }) {
  const leerTokenPropio = motivo === 'servicios' ? leerTokenServicios : leerTokenTienda;
  const guardarTokenPropio = motivo === 'servicios' ? guardarTokenServicios : guardarTokenTienda;
  const juno = document.getElementById('juno');
  const botonAbrir = document.getElementById('juno-abrir');
  const botonCerrar = document.getElementById('juno-cerrar');
  const ventana = document.getElementById('juno-ventana');
  const sugerencias = document.getElementById('juno-sugerencias');
  const formulario = document.getElementById('juno-form');
  const campo = document.getElementById('juno-texto');
  const botonEnviar = formulario.querySelector('button');
  const zona = document.getElementById('mensajes');

  let token = null; // la conversación de esta ventana (se abre al escribir por primera vez, o se recupera al abrir)
  let conversacionLista = false;
  let ocupado = false;
  let esperaSiguiente = null;
  ventana.dataset.conversacion = 'vacia';

  function enfocar() {
    if (window.matchMedia('(pointer: fine)').matches) campo.focus({ preventScroll: true });
  }

  // ---------- Abrir y cerrar ----------

  async function abrir() {
    ventana.hidden = false;
    juno.dataset.estado = 'abierta';
    botonAbrir.setAttribute('aria-expanded', 'true');
    // Si ya habían hablado antes, se recupera lo conversado (sin crear nada nuevo si no hubo conversación).
    if (!conversacionLista && leerTokenPropio()) {
      try {
        await prepararConversacion();
      } catch {
        /* sin conexión: se ve la ventana vacía y se reintenta al escribir */
      }
    }
    enfocar();
  }

  function cerrar() {
    ventana.hidden = true;
    juno.dataset.estado = 'cerrada';
    botonAbrir.setAttribute('aria-expanded', 'false');
    botonAbrir.focus({ preventScroll: true });
  }

  botonAbrir.addEventListener('click', abrir);
  botonCerrar.addEventListener('click', cerrar);
  document.addEventListener('keydown', (evento) => {
    if (evento.key === 'Escape' && juno.dataset.estado === 'abierta' && !document.querySelector('.pub-velo, .fh-velo, .cr-velo, .sv-velo')) cerrar();
  });

  // ---------- La conversación ----------

  // Abre (o recupera) la conversación de la tienda. Si hay un cliente reconocido en el chat principal, la de la tienda se
  // abre ya vinculada a él (no tiene que identificarse otra vez para apartar); la del chat principal no se toca.
  async function prepararConversacion() {
    let actual = leerTokenPropio();
    const principal = leerToken();
    if (principal && (await datosDeSesion(principal)).reconocido) {
      const reconocida = actual ? (await datosDeSesion(actual)).reconocido : false;
      if (!reconocida) actual = await nuevaConversacion(principal);
    }
    const datos = await iniciar(actual, motivo);
    token = datos.token;
    guardarTokenPropio(token);
    fijarAgente(datos.agente);
    conversacionLista = true;
    if (datos.mensajes.length > 0) {
      ventana.dataset.conversacion = 'activa';
      zona.replaceChildren();
      datos.mensajes.forEach((m) => agregarBurbuja(m.emisor === 'A' ? 'agente' : 'persona', m.texto));
      mostrarComponente(datos.componente);
    }
    // El carrito que Juno tenía guardado (de otra visita) vuelve a dibujarse, haya o no mensajes.
    if (datos.carrito) alCarrito?.(datos.carrito, '');
    // La solicitud que había quedado a medias en /servicios vuelve a dibujarse (con lo que ya se había anotado).
    if (datos.solicitud) alSolicitud?.(datos.solicitud, null);
  }

  // Una tira corta bajo lo que dijo Juno: «2 productos · S/ 168.00 · Ver carrito».
  function tiraDeCarrito(carrito) {
    const tira = document.createElement('div');
    tira.className = 'cr-mini';
    const texto = document.createElement('span');
    texto.textContent =
      carrito.unidades === 0 ? 'Carrito vacío' : `${carrito.unidades === 1 ? '1 producto' : `${carrito.unidades} productos`} · ${carrito.total}`;
    const ver = document.createElement('button');
    ver.type = 'button';
    ver.textContent = 'Ver carrito';
    ver.addEventListener('click', () => alVerCarrito?.());
    tira.append(texto, ver);
    zona.appendChild(tira);
    zona.scrollTop = zona.scrollHeight;
  }

  function mostrarComponente(componente, textoDeJuno = '') {
    if (!componente) return;
    if (componente.tipo === 'consentimiento') {
      mostrarConsentimiento({
        alAceptar: () => enviar('Acepto los términos y condiciones', 'aceptar_terminos'),
        alRechazar: () => enviar('No, gracias', 'rechazar_terminos'),
      });
    } else if (componente.tipo === 'identidad') {
      mostrarTarjetaDatos(componente);
      alIdentificar?.(); // el encabezado y el menú pasan a ser los de un cliente
    } else if (componente.tipo === 'servicio') {
      mostrarTarjetaServicio(componente);
    } else if (componente.tipo === 'orden') {
      mostrarTarjetaOrden(componente);
    } else if (componente.tipo === 'producto') {
      mostrarTarjetaProducto(componente);
    } else if (componente.tipo === 'apartado') {
      mostrarTarjetaApartado(componente);
    } else if (componente.tipo === 'catalogo') {
      alCatalogo?.(componente); // la vitrina de la página cambia; en la ventana solo queda lo que Juno dijo
    } else if (componente.tipo === 'carrito') {
      if (componente.productos.length > 0 || componente.fase === 'carrito') tiraDeCarrito(componente);
      alCarrito?.(componente, textoDeJuno); // el botón del encabezado y el cajón se ponen al día
    } else if (componente.tipo === 'pedido') {
      alPedido?.(componente);
    }
  }

  function mostrarSalida(salida) {
    limpiarOpciones();
    if (salida.respuesta) agregarBurbuja('agente', salida.respuesta);
    mostrarComponente(salida.componente, salida.respuesta);
    if (salida.solicitud) alSolicitud?.(salida.solicitud, salida.componente);
    mostrarOpciones(salida.opciones, enviar);
    avisarSiTapaLaVitrina(salida.componente);
    esperaSiguiente = salida.espera || null;
  }

  // En celulares la ventana ocupa buena parte de la pantalla y tapa la vitrina que Juno acaba de filtrar: se ofrece un botón
  // para minimizarla y ver los resultados.
  function avisarSiTapaLaVitrina(componente) {
    if (!componente || componente.tipo !== 'catalogo' || !window.matchMedia('(max-width: 820px)').matches) return;
    const cantidad = componente.productos.length;
    const boton = document.createElement('button');
    boton.type = 'button';
    boton.className = 'tn-ver-resultados';
    boton.textContent = cantidad === 1 ? 'Ver el producto' : `Ver los ${cantidad} productos`;
    boton.addEventListener('click', cerrar);
    let caja = zona.querySelector('.opciones');
    if (!caja) {
      caja = document.createElement('div');
      caja.className = 'opciones';
      zona.appendChild(caja);
    }
    caja.prepend(boton);
    zona.scrollTop = zona.scrollHeight;
  }

  function bloquear(estado) {
    ocupado = estado;
    campo.disabled = estado;
    botonEnviar.disabled = estado;
    if (!estado && !ventana.hidden && !document.querySelector('.cr, .cr-ck')) enfocar();
  }

  // Envía lo que la persona escribió (o pidió con un botón) y muestra la respuesta de Juno.
  // "rapido": una orden de la página (el cajón del carrito): sin la pausa de «escribiendo…».
  async function enviar(texto, accion = null, { rapido = false } = {}) {
    if (ocupado) return;
    bloquear(true);
    try {
      if (!conversacionLista) {
        try {
          await prepararConversacion();
        } catch {
          ventana.dataset.conversacion = 'activa';
          agregarBurbuja('agente', 'Ups, no pude conectarme. Intenta de nuevo en un momento.');
          return;
        }
      }
      ventana.dataset.conversacion = 'activa';
      limpiarOpciones();
      agregarBurbuja('persona', texto);

      const quitarCargando = mostrarCargando(esperaSiguiente);
      const minimo = rapido ? 0 : esperaSiguiente ? MINIMO_ESPERA_MS : MINIMO_ESCRIBIENDO_MS;
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

  // Si Juno está respondiendo otra cosa, la orden espera su turno.
  async function esperarLibre() {
    for (let i = 0; i < 100 && ocupado; i++) await pausa(80);
  }

  formulario.addEventListener('submit', (evento) => {
    evento.preventDefault();
    const texto = campo.value.trim();
    if (texto === '') return;
    campo.value = '';
    enviar(texto);
  });

  return {
    // Una petición hecha desde la página («Añadir al carrito con Juno»): abre la ventana y se la pide a Juno.
    async pedir(texto) {
      if (juno.dataset.estado !== 'abierta') await abrir();
      await esperarLibre();
      return enviar(texto);
    },
    // Una orden de la página que no necesita abrir la ventana (los botones del cajón del carrito): Juno la ejecuta igual.
    async ordenar(texto) {
      await esperarLibre();
      return enviar(texto, null, { rapido: true });
    },
    // Al cargar la página: si Juno tenía un carrito guardado, se recupera para dibujarlo (sin abrir la ventana).
    async recuperar() {
      if (conversacionLista || !leerTokenPropio()) return;
      try {
        await prepararConversacion();
      } catch {
        /* sin conexión: se recuperará al abrir la ventana */
      }
    },
    estaAbierta: () => juno.dataset.estado === 'abierta',
    abrir,
    minimizar() {
      if (juno.dataset.estado === 'abierta') cerrar();
    },
    // Las sugerencias de la ventana vacía (botones que escriben por la persona).
    ponerSugerencias(lista) {
      sugerencias.replaceChildren(
        ...lista.map((texto) => {
          const boton = document.createElement('button');
          boton.type = 'button';
          boton.textContent = texto;
          boton.addEventListener('click', () => enviar(texto));
          return boton;
        }),
      );
    },
  };
}
