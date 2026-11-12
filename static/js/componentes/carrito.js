// carrito.js: lo que se VE del carrito de la tienda: el botón «Carrito» del encabezado (con su contador), el cajón «Mi carrito»,
// la pantalla «¿Cómo quieres completar tu pedido?» y la confirmación «¡Pedido apartado!».
// Esta pantalla SOLO MUESTRA. El carrito vive en Juno (la conversación lo guarda y lo calcula): cada vez que Juno cambia algo,
// manda el carrito ya armado y aquí se dibuja. Los botones (+, −, Quitar, Continuar, Apartar) no cambian nada por su cuenta:
// le piden a Juno que lo haga con "ordenar(texto)", igual que si la persona lo hubiera escrito.

import { crear } from '../dom.js';
import { dibujoDeProducto } from './catalogo_arte.js';
import { iconoDe, tonoDe, varianteDe } from './catalogo_comun.js';

const NS_SVG = 'http://www.w3.org/2000/svg';
const FOTO_JUNO = '/static/img/juno.png';

const ICONOS = {
  carrito: ['M3 4h2l2.4 10.2a1.6 1.6 0 001.6 1.3h8.1a1.6 1.6 0 001.6-1.2L20.5 8H6.2', 'M9.5 18.2a1.3 1.3 0 100 2.6 1.3 1.3 0 000-2.6z', 'M17 18.2a1.3 1.3 0 100 2.6 1.3 1.3 0 000-2.6z'],
  cerrar: ['M6 6l12 12M18 6L6 18'],
  tienda: ['M4 9l1.5-5h13L20 9M4 9v10h16V9M4 9h16M9 19v-5h6v5'],
  tarjeta: ['M5.5 6h13A2.5 2.5 0 0121 8.5v7a2.5 2.5 0 01-2.5 2.5h-13A2.5 2.5 0 013 15.5v-7A2.5 2.5 0 015.5 6z', 'M3 10h18M7 15h3'],
  ok: ['M5 12.5l4.5 4.5L19 7.5'],
  copiar: ['M9 9h9a1 1 0 011 1v9a1 1 0 01-1 1H9a1 1 0 01-1-1v-9a1 1 0 011-1z', 'M5 15V6a1 1 0 011-1h9'],
};

function icono(nombre) {
  const svg = document.createElementNS(NS_SVG, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('aria-hidden', 'true');
  for (const trazo of ICONOS[nombre]) {
    const camino = document.createElementNS(NS_SVG, 'path');
    camino.setAttribute('d', trazo);
    svg.appendChild(camino);
  }
  return svg;
}

// "producto" es lo que tiene la vitrina (puede ser undefined si ya no está): se dibuja su foto con el color de su categoría.
function miniatura(producto, clase) {
  const caja = crear('div', clase);
  const [fuerte, medio, claro] = tonoDe(producto ? producto.categoria : '');
  caja.style.background = `linear-gradient(160deg, ${fuerte}, ${medio} 60%, ${claro})`;
  caja.append(producto ? dibujoDeProducto(iconoDe(producto), varianteDe(producto.nombre)) : dibujoDeProducto('caja'));
  return caja;
}

function botonIcono(nombre, clase, etiqueta) {
  const boton = crear('button', clase);
  boton.type = 'button';
  boton.setAttribute('aria-label', etiqueta);
  boton.append(icono(nombre));
  return boton;
}

function ficha(...hijos) {
  const caja = crear('div');
  caja.append(...hijos);
  return caja;
}

// "ordenar(texto)": le pide algo a Juno y devuelve una promesa que termina cuando ella respondió.
// "productoDe(id)" y "productoPorNombre(nombre)": buscan el producto en la vitrina (para dibujar su foto).
// "alSeguirViendo()": la persona quiere volver a la vitrina. "alHablarConJuno()": abre la ventana de Juno.
export function iniciarCarrito({ ordenar, productoDe, productoPorNombre, alHablarConJuno, alCambiar }) {
  let estado = { fase: 'carrito', productos: [], unidades: 0, total: 'S/ 0.00' };
  let avisoJuno = ''; // lo último que dijo Juno sobre el carrito (se muestra arriba del cajón)
  let cajon = null; // { velo, panel } cuando el cajón está abierto
  let cierre = null; // { velo, panel } cuando está la pantalla «¿Cómo quieres completar tu pedido?»
  let confirmacion = null;
  let ocupado = false;
  let opcion = 'tienda'; // la opción elegida en el cierre: «ahora» o «tienda»
  let ultimoFoco = null;

  // ---------- El botón «Carrito» del encabezado ----------
  const boton = crear('button', 'cr-boton');
  boton.type = 'button';
  boton.setAttribute('aria-haspopup', 'dialog');
  const contador = crear('b', '', '0');
  boton.append(icono('carrito'), crear('span', 'cr-boton-texto', 'Carrito'), contador);
  const destino = document.querySelector('.lz-cab-der');
  if (destino) destino.prepend(boton);
  else document.body.append(boton);
  boton.addEventListener('click', () => abrirCajon());

  function pintarBoton() {
    contador.textContent = String(estado.unidades);
    boton.classList.toggle('lleno', estado.unidades > 0);
    boton.setAttribute('aria-label', estado.unidades === 1 ? 'Carrito, 1 producto' : `Carrito, ${estado.unidades} productos`);
  }
  pintarBoton();

  // ---------- Piezas comunes ----------
  function velo(alCerrar) {
    const capa = crear('div', 'cr-velo');
    capa.addEventListener('click', alCerrar);
    return capa;
  }

  function cerrarEnEscape(evento) {
    if (evento.key !== 'Escape') return;
    if (confirmacion) cerrarConfirmacion();
    else if (cierre) cerrarCierre();
    else if (cajon) cerrarCajon();
  }
  document.addEventListener('keydown', cerrarEnEscape);

  function quitar(...partes) {
    for (const parte of partes) {
      if (!parte) continue;
      parte.classList.add('saliendo');
      setTimeout(() => parte.remove(), 160);
    }
  }

  function devolverFoco() {
    if (!cajon && !cierre && !confirmacion && ultimoFoco && ultimoFoco.isConnected) ultimoFoco.focus({ preventScroll: true });
  }

  function bloqueoDePagina() {
    document.body.classList.toggle('cr-abierto', Boolean(cajon || cierre || confirmacion));
  }

  // Pide algo a Juno desde el cajón o el cierre: mientras responde, los botones quedan en pausa.
  async function pedir(texto) {
    if (ocupado) return;
    ocupado = true;
    marcarOcupado(true);
    try {
      await ordenar(texto);
    } finally {
      ocupado = false;
      marcarOcupado(false);
    }
  }

  function marcarOcupado(si) {
    for (const parte of [cajon, cierre]) if (parte) parte.panel.classList.toggle('ocupado', si);
  }

  function resumenDeLineas(lineas, rotulo) {
    const columna = crear('div', 'cr-resumen');
    columna.append(crear('b', 'cr-resumen-titulo', rotulo));
    for (const linea of lineas) {
      const fila = crear('div', 'cr-rs');
      fila.append(miniatura(linea.producto, 'cr-rs-im'), crear('span', 'cr-rs-n', linea.cantidad > 1 ? `${linea.nombre} × ${linea.cantidad}` : linea.nombre), crear('span', '', linea.subtotal));
      columna.append(fila);
    }
    return columna;
  }

  // ---------- El cajón «Mi carrito» ----------
  function lineaDelCajon(linea) {
    const producto = productoDe(linea.id);
    const fila = crear('div', 'cr-it');
    const datos = crear('div', 'cr-it-d');
    datos.append(crear('b', 'cr-it-n', linea.nombre));
    if (linea.detalle) datos.append(crear('span', 'cr-it-sp', linea.detalle));

    const cantidad = crear('div', 'cr-qty');
    const menos = crear('button', '', '−');
    menos.type = 'button';
    menos.setAttribute('aria-label', 'Quitar una unidad');
    const numero = crear('i', '', String(linea.cantidad));
    const mas = crear('button', '', '+');
    mas.type = 'button';
    mas.setAttribute('aria-label', 'Añadir una unidad');
    mas.disabled = linea.cantidad >= linea.maximo;
    menos.addEventListener('click', () =>
      pedir(linea.cantidad <= 1 ? `Quitar del carrito: ${linea.nombre}` : `Cambiar cantidad: ${linea.nombre} = ${linea.cantidad - 1}`),
    );
    mas.addEventListener('click', () => pedir(`Cambiar cantidad: ${linea.nombre} = ${linea.cantidad + 1}`));
    cantidad.append(menos, numero, mas);

    const quitarLinea = crear('button', 'cr-rm', 'Quitar');
    quitarLinea.type = 'button';
    quitarLinea.addEventListener('click', () => pedir(`Quitar del carrito: ${linea.nombre}`));
    const controles = crear('div', 'cr-it-c');
    controles.append(cantidad, quitarLinea);
    datos.append(controles);
    if (linea.cantidad >= linea.maximo) datos.append(crear('span', 'cr-tope', linea.cantidad === 1 && linea.maximo === 1 ? 'Última unidad disponible' : 'Máximo que puedo guardarte'));

    const precio = crear('div', 'cr-it-pz');
    precio.append(crear('b', '', linea.subtotal));
    if (linea.cantidad > 1) precio.append(crear('small', '', `${linea.precio_unitario} c/u`));
    fila.append(miniatura(producto, 'cr-it-im'), datos, precio);
    return fila;
  }

  function dibujarCajon() {
    if (!cajon) return;
    const { panel } = cajon;
    const desplazado = panel.querySelector('.cr-lst')?.scrollTop || 0; // para que al cambiar una cantidad la lista no salte
    const cabecera = crear('div', 'cr-ch');
    const titulo = crear('h2', '', 'Mi carrito');
    if (estado.unidades > 0) titulo.append(crear('small', '', estado.unidades === 1 ? '1 producto' : `${estado.unidades} productos`));
    const cerrar = botonIcono('cerrar', 'cr-x', 'Cerrar el carrito');
    cerrar.addEventListener('click', cerrarCajon);
    cabecera.append(titulo, cerrar);

    const juno = crear('div', 'cr-jn');
    const foto = crear('img');
    foto.src = FOTO_JUNO;
    foto.alt = '';
    juno.append(foto, crear('span', '', avisoJuno || '¿Quieres quitar o cambiar algo? Dímelo en el chat y yo me encargo.'));

    const lista = crear('div', 'cr-lst');
    const pie = crear('div', 'cr-cf');
    if (estado.productos.length === 0) {
      const vacio = crear('div', 'cr-vacio');
      vacio.append(icono('carrito'), crear('b', '', 'Tu carrito está vacío'), crear('span', '', 'Añade lo que te guste desde el catálogo y Juno lo guarda por ti.'));
      lista.append(vacio);
      const seguir = crear('button', 'cr-bn p', 'Ver el catálogo');
      seguir.type = 'button';
      seguir.addEventListener('click', cerrarCajon);
      pie.append(seguir);
    } else {
      for (const linea of estado.productos) lista.append(lineaDelCajon(linea));
      const total = crear('div', 'cr-tot');
      total.append(crear('span', '', 'Subtotal'), crear('b', '', estado.total));
      const continuar = crear('button', 'cr-bn p', 'Continuar con mi pedido');
      continuar.type = 'button';
      continuar.addEventListener('click', async () => {
        await pedir('Ir a pagar');
      });
      const seguir = crear('button', 'cr-seg', 'Seguir viendo el catálogo');
      seguir.type = 'button';
      seguir.addEventListener('click', cerrarCajon);
      pie.append(total, continuar, seguir);
    }
    panel.replaceChildren(cabecera, juno, lista, pie);
    lista.scrollTop = desplazado;
  }

  function abrirCajon() {
    if (cajon) return;
    cerrarCierre(true);
    ultimoFoco = document.activeElement;
    const capa = velo(cerrarCajon);
    const panel = crear('aside', 'cr');
    panel.setAttribute('role', 'dialog');
    panel.setAttribute('aria-modal', 'true');
    panel.setAttribute('aria-label', 'Mi carrito');
    cajon = { velo: capa, panel };
    dibujarCajon();
    document.body.append(capa, panel);
    bloqueoDePagina();
    panel.querySelector('.cr-x')?.focus({ preventScroll: true });
  }

  function cerrarCajon() {
    if (!cajon) return;
    quitar(cajon.velo, cajon.panel);
    cajon = null;
    avisoJuno = '';
    bloqueoDePagina();
    devolverFoco();
  }

  // ---------- «¿Cómo quieres completar tu pedido?» ----------
  function opcionDeCierre(clave, nombreIcono, titulo, texto, etiqueta) {
    const caja = crear('button', 'cr-op');
    caja.type = 'button';
    caja.setAttribute('role', 'radio');
    caja.dataset.clave = clave;
    const ic = crear('span', 'cr-op-ic');
    ic.append(icono(nombreIcono));
    const cuerpo = crear('div');
    const cabeza = crear('b', '', titulo);
    if (etiqueta) cabeza.append(crear('em', '', etiqueta));
    cuerpo.append(cabeza, crear('span', 'cr-op-t', texto));
    caja.append(ic, cuerpo, crear('span', 'cr-rd'));
    return caja;
  }

  function dibujarCierre() {
    if (!cierre) return;
    const { panel } = cierre;
    const izquierda = crear('div', 'cr-l');
    const pasos = crear('div', 'cr-pas');
    const paso = (numero, nombre, clase) => {
      const p = crear('span', clase);
      p.append(crear('i', '', clase === 'ok' ? '✓' : String(numero)), nombre);
      return p;
    };
    pasos.append(paso(1, 'Carrito', 'ok'), crear('hr'), paso(2, 'Entrega y pago', 'on'), crear('hr'), paso(3, 'Listo', ''));

    const opciones = crear('div', 'cr-ops');
    opciones.setAttribute('role', 'radiogroup');
    opciones.setAttribute('aria-label', 'Cómo completar el pedido');
    const ahora = opcionDeCierre('ahora', 'tarjeta', 'Pagar ahora', 'Paga desde la plataforma y tu pedido queda confirmado.', 'Próximamente');
    const tienda = opcionDeCierre('tienda', 'tienda', 'Apartar y pagar en tienda', 'Te guardamos tus productos y pagas cuando pases a recogerlos.');
    opciones.append(ahora, tienda);

    const volver = crear('button', 'cr-bn s', 'Volver al carrito');
    volver.type = 'button';
    volver.addEventListener('click', () => abrirCajon());
    const confirmar = crear('button', 'cr-bn p');
    confirmar.type = 'button';
    const nota = crear('p', 'cr-nota');

    function pintarOpcion() {
      for (const caja of [ahora, tienda]) {
        const activa = caja.dataset.clave === opcion;
        caja.classList.toggle('on', activa);
        caja.setAttribute('aria-checked', String(activa));
      }
      confirmar.replaceChildren();
      if (opcion === 'tienda') {
        const juno = crear('img');
        juno.src = FOTO_JUNO;
        juno.alt = '';
        confirmar.append(juno, 'Apartar mi pedido con Juno');
        confirmar.disabled = false;
        nota.textContent = '';
      } else {
        confirmar.textContent = 'Pagar ahora';
        confirmar.disabled = true;
        nota.textContent = 'El pago en línea todavía no está disponible. Mientras tanto, aparta tu pedido y págalo en la tienda al recogerlo.';
      }
    }
    ahora.addEventListener('click', () => {
      opcion = 'ahora';
      pintarOpcion();
    });
    tienda.addEventListener('click', () => {
      opcion = 'tienda';
      pintarOpcion();
    });
    confirmar.addEventListener('click', () => {
      if (opcion === 'tienda') pedir('Apartar mi pedido');
    });
    pintarOpcion();

    const botones = crear('div', 'cr-bs');
    botones.append(volver, confirmar);
    const cerrar = botonIcono('cerrar', 'cr-x cr-x-cierre', 'Cerrar');
    cerrar.addEventListener('click', () => cerrarCierre());
    izquierda.append(pasos, crear('h2', '', '¿Cómo quieres completar tu pedido?'), crear('p', 'cr-sub', 'Elige una opción. Juno se encarga del resto.'), opciones, nota, botones, cerrar);

    const derecha = crear('div', 'cr-r');
    const lineas = estado.productos.map((l) => ({ ...l, producto: productoDe(l.id) }));
    derecha.append(resumenDeLineas(lineas, `TU PEDIDO · ${estado.unidades === 1 ? '1 PRODUCTO' : `${estado.unidades} PRODUCTOS`}`));
    const total = crear('div', 'cr-tot cr-tot-r');
    total.append(crear('span', '', 'Total'), crear('b', '', estado.total));
    derecha.append(total);
    panel.replaceChildren(izquierda, derecha);
  }

  function abrirCierre() {
    if (cierre) {
      dibujarCierre();
      return;
    }
    cerrarCajon();
    ultimoFoco = ultimoFoco || document.activeElement;
    const capa = velo(() => cerrarCierre());
    const panel = crear('section', 'cr-ck');
    panel.setAttribute('role', 'dialog');
    panel.setAttribute('aria-modal', 'true');
    panel.setAttribute('aria-label', '¿Cómo quieres completar tu pedido?');
    cierre = { velo: capa, panel };
    opcion = 'tienda';
    dibujarCierre();
    document.body.append(capa, panel);
    bloqueoDePagina();
    panel.querySelector('.cr-op.on')?.focus({ preventScroll: true });
  }

  function cerrarCierre(sinFoco = false) {
    if (!cierre) return;
    quitar(cierre.velo, cierre.panel);
    cierre = null;
    bloqueoDePagina();
    if (!sinFoco) devolverFoco();
  }

  // ---------- «¡Pedido apartado!» ----------
  function cerrarConfirmacion() {
    if (!confirmacion) return;
    quitar(confirmacion.velo, confirmacion.panel);
    confirmacion = null;
    bloqueoDePagina();
    devolverFoco();
  }

  function mostrarPedido(pedido) {
    cerrarCajon();
    cerrarCierre(true);
    cerrarConfirmacion();
    const capa = velo(cerrarConfirmacion);
    const panel = crear('section', 'cr-ck cr-listo');
    panel.setAttribute('role', 'dialog');
    panel.setAttribute('aria-modal', 'true');
    panel.setAttribute('aria-label', 'Pedido apartado');
    confirmacion = { velo: capa, panel };

    const izquierda = crear('div', 'cr-l');
    const pasos = crear('div', 'cr-pas');
    const paso = (numero, nombre, clase) => {
      const p = crear('span', clase);
      p.append(crear('i', '', clase === 'ok' ? '✓' : String(numero)), nombre);
      return p;
    };
    pasos.append(paso(1, 'Carrito', 'ok'), crear('hr'), paso(2, 'Entrega y pago', 'ok'), crear('hr'), paso(3, 'Listo', 'on'));

    const centro = crear('div', 'cr-centro');
    const marca = crear('div', 'cr-okc');
    marca.append(icono('ok'));
    centro.append(marca, crear('h2', '', '¡Pedido apartado!'), crear('p', 'cr-sub', 'Pasa por la tienda y paga al recoger tus productos.'));

    const codigo = crear('div', 'cr-cod');
    const numero = crear('b', '', pedido.codigo);
    const copiar = crear('button', 'cr-copiar', 'Copiar');
    copiar.type = 'button';
    copiar.addEventListener('click', async () => {
      try {
        await navigator.clipboard.writeText(pedido.codigo);
        copiar.textContent = 'Copiado';
      } catch {
        copiar.textContent = 'Anótalo';
      }
      setTimeout(() => (copiar.textContent = 'Copiar'), 1800);
    });
    codigo.append(crear('small', '', 'CÓDIGO DE RETIRO'), numero, copiar);
    if (pedido.vence) codigo.append(crear('span', 'cr-vence', `Vigente hasta el ${pedido.vence}`));

    const juno = crear('div', 'cr-jb');
    const foto = crear('img');
    foto.src = FOTO_JUNO;
    foto.alt = '';
    const cantidad = pedido.productos.reduce((suma, l) => suma + l.cantidad, 0);
    juno.append(foto, crear('div', '', `Listo, guardé tus ${cantidad === 1 ? 'producto' : `${cantidad} productos`}. Muestra este código cuando vengas a recogerlos.`));

    const seguir = crear('button', 'cr-bn s', 'Seguir viendo el catálogo');
    seguir.type = 'button';
    seguir.addEventListener('click', cerrarConfirmacion);
    const hablar = crear('button', 'cr-bn p', 'Hablar con Juno');
    hablar.type = 'button';
    hablar.addEventListener('click', () => {
      cerrarConfirmacion();
      alHablarConJuno();
    });
    const botones = crear('div', 'cr-bs');
    botones.append(seguir, hablar);
    const cerrar = botonIcono('cerrar', 'cr-x cr-x-cierre', 'Cerrar');
    cerrar.addEventListener('click', cerrarConfirmacion);
    izquierda.append(pasos, centro, codigo, juno, botones, cerrar);

    const derecha = crear('div', 'cr-r');
    const lineas = pedido.productos.map((l) => ({ ...l, producto: productoPorNombre(l.nombre) }));
    derecha.append(resumenDeLineas(lineas, 'APARTADO'));
    const total = crear('div', 'cr-sm');
    total.append(crear('span', '', 'A pagar en tienda'), crear('b', '', pedido.total));
    derecha.append(total);
    panel.replaceChildren(izquierda, derecha);

    document.body.append(capa, panel);
    bloqueoDePagina();
    numero.parentElement.querySelector('.cr-copiar')?.focus({ preventScroll: true });
  }

  // ---------- Lo que manda Juno ----------
  return {
    // Juno mandó el carrito al día. "texto" es lo que dijo (si el cajón está abierto, lo muestra arriba).
    actualizar(carrito, texto = '') {
      const antes = estado.unidades;
      estado = carrito;
      pintarBoton();
      if (carrito.unidades !== antes) {
        boton.classList.remove('salto');
        void boton.offsetWidth;
        boton.classList.add('salto');
      }
      avisoJuno = (cajon || cierre) && texto ? texto : ''; // solo se muestra lo que Juno dijo mientras la persona miraba el carrito
      if (carrito.fase === 'cierre' && carrito.productos.length > 0) {
        abrirCierre();
      } else if (cierre) {
        // Estaba cerrando el pedido y Juno devolvió el carrito (algo cambió o falta registrarse): se vuelve a revisar.
        abrirCajon();
      } else if (cajon) {
        dibujarCajon();
      }
      alCambiar(this.cantidades());
    },
    // Juno apartó el pedido.
    pedidoApartado(pedido) {
      estado = { fase: 'carrito', productos: [], unidades: 0, total: 'S/ 0.00' };
      pintarBoton();
      mostrarPedido(pedido);
      alCambiar(this.cantidades());
    },
    abrir: abrirCajon,
    hayAlgoAbierto: () => Boolean(cajon || cierre || confirmacion),
    unidades: () => estado.unidades,
    total: () => estado.total,
    cantidades() {
      return new Map(estado.productos.map((l) => [l.id, l.cantidad]));
    },
  };
}
