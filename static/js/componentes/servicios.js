// servicios.js (componentes): las tarjetas de servicios de /servicios y la ficha de un servicio («Más info»).
// Solo MUESTRA: «Solicitar servicio», «Preguntarle a Juno» y el enlace a la cita de diagnóstico no ejecutan nada: le piden a Juno que lo haga
// (la página los conecta en static/js/servicios.js). Los datos (nombre, descripción, precio base, tiempo y equipos) los manda el servidor.
// Los dibujos y colores son los mismos de la tienda (catalogo_arte.js y catalogo_comun.js).

import { crear } from '../dom.js';
import { dibujoDeProducto } from './catalogo_arte.js';
import { partirPrecio, sinTildes, tonoDe, varianteDe } from './catalogo_comun.js';

const NS_SVG = 'http://www.w3.org/2000/svg';
const FOTO_JUNO = '/static/img/juno.png';

// Trazos de SVG fijos de los iconitos.
const CIRCULO = 'M12 3.5a8.5 8.5 0 100 17 8.5 8.5 0 000-17z';
const TRAZOS = {
  reloj: [CIRCULO, 'M12 7.5V12l3 2'],
  info: [CIRCULO, 'M12 11v5', 'M12 8h.01'],
  precio: ['M20.5 12.5l-8 8-9-9V3.5h8z', 'M7.5 6.5a1 1 0 100 2 1 1 0 000-2z'],
  laptop: ['M6.5 5h11A1.5 1.5 0 0119 6.5V15a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 015 15V6.5A1.5 1.5 0 016.5 5z', 'M2.5 19.5h19'],
  alerta: [CIRCULO, 'M12 8v5', 'M12 16h.01'],
  cerrar: ['M6 6l12 12M18 6L6 18'],
};

export function iconoSvg(nombre) {
  const svg = document.createElementNS(NS_SVG, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('aria-hidden', 'true');
  for (const trazo of TRAZOS[nombre]) {
    const camino = document.createElementNS(NS_SVG, 'path');
    camino.setAttribute('d', trazo);
    svg.appendChild(camino);
  }
  return svg;
}

// Qué dibujo lleva un servicio, según las palabras de su nombre (sin tildes). Lo que no se reconoce sale como laptop.
const REGLAS_ARTE = [
  ['bateria', ['bateria', 'cargador']],
  ['pantalla', ['pantalla', 'display']],
  ['teclado', ['teclado']],
  ['ssd', ['ssd', 'disco', 'almacenamiento']],
  ['ram', ['ram', 'memoria']],
  ['mouse', ['mouse']],
  ['caja', ['placa', 'motherboard', 'tarjeta madre', 'mainboard']],
];

export function arteDeServicio(nombre) {
  const texto = sinTildes(nombre);
  for (const [dibujo, palabras] of REGLAS_ARTE) {
    if (palabras.some((palabra) => texto.includes(palabra))) return dibujo;
  }
  return 'laptop';
}

// Los tres tonos pastel se reparten por turnos entre las tarjetas (los mismos de la tienda: azul violáceo, menta y rosa).
const TONOS = ['laptops', 'repuestos', 'accesorios'];

export function fondoDe(indice) {
  const [fuerte, medio, claro] = tonoDe(TONOS[indice % TONOS.length]);
  return `linear-gradient(160deg, ${fuerte}, ${medio} 60%, ${claro})`;
}

// «A cotizar» o «desde S/ 150» (con céntimos solo si no son cero).
function precioDe(servicio) {
  const linea = crear('p', servicio.precio ? 'sv-p' : 'sv-p q');
  if (!servicio.precio) {
    linea.textContent = 'A cotizar';
    return linea;
  }
  const { entero, decimales } = partirPrecio(`S/ ${servicio.precio}`);
  linea.append(crear('small', '', 'desde S/'), entero);
  if (decimales) linea.append(crear('small', 'dec', decimales));
  return linea;
}

// El texto que dice el costo (el mismo que usa Juno en el chat). Con precio, solo se muestra si el Administrador activó el aviso
// «El precio puede cambiar»; sin precio, siempre se explica cómo se cotiza. Devuelve '' cuando no hay nada que decir.
function notaDeCosto(servicio) {
  if (!servicio.precio) return 'Te damos el precio después de revisar tu equipo.';
  return servicio.aviso_precio === false ? '' : 'El costo final se confirma después de revisar tu equipo, según su estado.';
}

// La imagen que el Administrador le puso al servicio o, si no tiene (o no carga), la ilustración de siempre.
function ponerImagenODibujo(foto, servicio) {
  const dibujo = () => dibujoDeProducto(arteDeServicio(servicio.nombre), varianteDe(servicio.nombre));
  if (!servicio.imagen) {
    foto.append(dibujo());
    return;
  }
  const imagen = crear('img', 'sv-img');
  imagen.src = servicio.imagen;
  imagen.alt = '';
  imagen.decoding = 'async';
  imagen.addEventListener('error', () => imagen.replaceWith(dibujo()));
  foto.prepend(imagen); // debajo de las etiquetas (equipo, tiempo y precio)
}

// Dibuja todas las tarjetas. "acciones": solicitar(servicio), preguntar(servicio), diagnostico().
export function mostrarServicios(contenedor, servicios, acciones) {
  contenedor.replaceChildren(...servicios.map((servicio, indice) => tarjeta(servicio, indice, acciones)));
}

// La tarjeta elegida (la que Juno está atendiendo) se marca; con null se desmarcan todas.
export function marcarElegido(nombre) {
  document.querySelectorAll('.sv-c').forEach((tarjetaServicio) => {
    tarjetaServicio.classList.toggle('elegido', nombre !== null && tarjetaServicio.dataset.nombre === nombre);
  });
}

function tarjeta(servicio, indice, acciones) {
  const articulo = crear('article', 'sv-c');
  articulo.dataset.nombre = servicio.nombre;

  const foto = crear('div', 'sv-foto');
  foto.style.background = fondoDe(indice);
  if (servicio.equipos.length > 0) foto.append(crear('span', 'sv-eq', servicio.equipos.join(' · ')));
  if (servicio.tiempo) {
    const tiempo = crear('span', 'sv-t');
    tiempo.append(iconoSvg('reloj'), servicio.tiempo);
    foto.append(tiempo);
  }
  ponerImagenODibujo(foto, servicio);
  foto.append(precioDe(servicio));
  // Tocar la foto abre la ficha (igual que «Más info»).
  foto.addEventListener('click', () => abrirFichaServicio(servicio, indice, acciones, articulo));

  const cuerpo = crear('div', 'sv-bd');
  cuerpo.append(crear('h3', '', servicio.nombre), crear('p', 'sv-d', servicio.descripcion));

  const botones = crear('div', 'sv-m');
  const principal = crear('button', 'sv-pri');
  principal.type = 'button';
  const juno = crear('img');
  juno.src = FOTO_JUNO;
  juno.alt = '';
  principal.append(juno, 'Solicitar servicio');
  // No solicita nada: le pide a Juno que lo haga.
  principal.addEventListener('click', () => acciones.solicitar(servicio));

  const mas = crear('button', 'sv-sec2');
  mas.type = 'button';
  mas.append(iconoSvg('info'), 'Más info');
  mas.addEventListener('click', () => abrirFichaServicio(servicio, indice, acciones, articulo));

  botones.append(principal, mas);
  cuerpo.append(botones);
  articulo.append(foto, cuerpo);
  return articulo;
}

// ---------- La ficha de un servicio ----------

let abierta = null; // la ficha que está en pantalla (solo hay una)

export function cerrarFichaServicio() {
  if (abierta) abierta.cerrar();
}

function caja(icono, titulo, valor) {
  const celda = crear('div');
  const circulo = crear('span', 'ic');
  circulo.append(iconoSvg(icono));
  const texto = crear('span');
  texto.append(crear('small', '', titulo), crear('b', '', valor));
  celda.append(circulo, texto);
  return celda;
}

function abrirFichaServicio(servicio, indice, acciones, tarjetaOrigen, destino = null) {
  if (!destino) cerrarFichaServicio();
  const velo = crear('div', 'sv-velo');
  const ficha = crear('section', 'sv-fh');
  ficha.setAttribute('role', 'dialog');
  ficha.setAttribute('aria-modal', 'true');
  ficha.setAttribute('aria-label', servicio.nombre);

  // Lado de la foto.
  const foto = crear('div', 'sv-fh-foto');
  foto.style.background = fondoDe(indice);
  if (servicio.equipos.length > 0) foto.append(crear('span', 'sv-fh-eq', servicio.equipos.join(' · ')));
  ponerImagenODibujo(foto, servicio);

  // Lado de los datos.
  const derecha = crear('div', 'sv-fh-der');
  const cerrarBoton = crear('button', 'sv-x');
  cerrarBoton.type = 'button';
  cerrarBoton.setAttribute('aria-label', 'Cerrar');
  cerrarBoton.append(iconoSvg('cerrar'));

  const cabecera = crear('div', 'sv-fh-cab');
  cabecera.append(crear('p', '', 'Servicio técnico'), crear('h2', '', servicio.nombre));

  const cuerpo = crear('div', 'sv-fh-cuerpo');
  const cajas = crear('div', 'sv-cajas');
  cajas.append(caja('precio', 'PRECIO BASE', servicio.precio ? `Desde S/ ${servicio.precio}` : 'A cotizar'));
  if (servicio.tiempo) cajas.append(caja('reloj', 'TIEMPO ESTIMADO', servicio.tiempo));
  if (servicio.equipos.length > 0) cajas.append(caja('laptop', 'EQUIPOS', servicio.equipos.join(' · ')));
  const textoNota = notaDeCosto(servicio);
  const nota = crear('div', 'sv-nota');
  nota.append(iconoSvg('alerta'), crear('span', '', textoNota));

  // «¿No estás seguro de que sea esto?»: el camino del diagnóstico.
  const alterno = crear('p', 'sv-alt', '¿No estás seguro de que sea esto? ');
  const enlace = crear('button', '', 'Agenda una cita de diagnóstico');
  enlace.type = 'button';
  enlace.addEventListener('click', () => {
    cerrar();
    acciones.diagnostico();
  });
  alterno.append(enlace);
  cuerpo.append(crear('p', 'sv-fh-desc', servicio.descripcion), cajas);
  if (textoNota) cuerpo.append(nota);
  cuerpo.append(alterno);

  const pie = crear('div', 'sv-fh-pie');
  const solicitar = crear('button', 'sv-bn p');
  solicitar.type = 'button';
  const juno = crear('img');
  juno.src = FOTO_JUNO;
  juno.alt = '';
  solicitar.append(juno, 'Solicitar este servicio con Juno');
  solicitar.addEventListener('click', () => {
    cerrar();
    acciones.solicitar(servicio);
  });
  const preguntar = crear('button', 'sv-bn s', 'Preguntarle a Juno');
  preguntar.type = 'button';
  preguntar.addEventListener('click', () => {
    cerrar();
    acciones.preguntar(servicio);
  });
  pie.append(solicitar, preguntar);

  derecha.append(cerrarBoton, cabecera, cuerpo, pie);
  ficha.append(foto, derecha);

  // En el panel, la ficha se dibuja dentro de un bloque de la página (vista previa) en vez de abrirse como ventana.
  if (destino) {
    ficha.classList.add('en-linea');
    destino.replaceChildren(ficha);
    return;
  }

  function alTeclado(evento) {
    if (evento.key === 'Escape') {
      evento.preventDefault();
      cerrar();
    }
  }
  function cerrar() {
    if (abierta !== manejador) return;
    abierta = null;
    document.removeEventListener('keydown', alTeclado);
    ficha.classList.add('saliendo');
    velo.classList.add('saliendo');
    const quitar = () => {
      ficha.remove();
      velo.remove();
    };
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) quitar();
    else setTimeout(quitar, 160);
    if (tarjetaOrigen && tarjetaOrigen.isConnected) tarjetaOrigen.querySelector('.sv-sec2')?.focus({ preventScroll: true });
  }
  const manejador = { cerrar };
  abierta = manejador;

  velo.addEventListener('click', cerrar);
  cerrarBoton.addEventListener('click', cerrar);
  document.addEventListener('keydown', alTeclado);
  document.body.append(velo, ficha);
  cerrarBoton.focus({ preventScroll: true });
}

// Dibuja la ficha completa de un servicio dentro de "destino" (sin ventana flotante). La usa la vista previa del panel.
export function fichaEnLinea(destino, servicio) {
  abrirFichaServicio(servicio, 0, { solicitar() {}, preguntar() {}, diagnostico() {} }, null, destino);
}
