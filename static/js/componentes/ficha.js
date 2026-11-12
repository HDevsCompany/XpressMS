// ficha.js: la ficha de un producto («Más info»). Es una ventana sobre la vitrina (en celulares, una hoja que sube desde abajo)
// con la foto grande a la izquierda y, a la derecha: nombre, pestañas «Resumen» / «Especificaciones», las vistas del producto
// (miniaturas que cambian la imagen principal), las existencias, el precio y los botones.
// Solo MUESTRA: sus dos botones le piden algo a Juno («Añadir al carrito con Juno», «Preguntarle a Juno») y se cierra.
// Los datos (cajas del resumen, tabla de especificaciones y vistas) los manda el servidor dentro de cada producto.

import { crear } from '../dom.js';
import { dibujoDeProducto, vistaDeProducto } from './catalogo_arte.js';
import { existencias, iconoDe, iconoStock, partirPrecio, tonoDe, varianteDe } from './catalogo_comun.js';

const NS_SVG = 'http://www.w3.org/2000/svg';
const FOTO_JUNO = '/static/img/juno.png';

// Los iconos de las cajas del resumen (trazos de SVG fijos). La base de datos solo dice la clave («cpu», «ram»...).
const ICONOS_CAJA = {
  cpu: ['M9 7h6a2 2 0 012 2v6a2 2 0 01-2 2H9a2 2 0 01-2-2V9a2 2 0 012-2z', 'M9 3v2M15 3v2M9 19v2M15 19v2M3 9h2M3 15h2M19 9h2M19 15h2'],
  ram: ['M4.5 7h15a1.5 1.5 0 011.5 1.5v6a1.5 1.5 0 01-1.5 1.5h-15A1.5 1.5 0 013 14.5v-6A1.5 1.5 0 014.5 7z', 'M7 10.5v2M11 10.5v2M15 10.5v2', 'M6 16v2M10 16v2M14 16v2M18 16v2'],
  disco: ['M6.5 5h11A2.5 2.5 0 0120 7.5v9a2.5 2.5 0 01-2.5 2.5h-11A2.5 2.5 0 014 16.5v-9A2.5 2.5 0 016.5 5z', 'M4 14h16M8 17h.01'],
  pantalla: ['M5 4h14a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2V6a2 2 0 012-2z', 'M8 20h8M12 16v4'],
  bateria: ['M4 8h13a2 2 0 012 2v4a2 2 0 01-2 2H4a2 2 0 01-2-2v-4a2 2 0 012-2z', 'M22 11v2M6 12h6'],
  pila: ['M4 8h13a2 2 0 012 2v4a2 2 0 01-2 2H4a2 2 0 01-2-2v-4a2 2 0 012-2z', 'M22 11v2M8 12h4M10 10v4'],
  mouse: ['M12 3a5 5 0 015 5v8a5 5 0 01-10 0V8a5 5 0 015-5z', 'M12 3v6'],
  wifi: ['M4.5 10a11 11 0 0115 0M8 13.5a6 6 0 018 0M12 17.5h.01'],
  sensor: ['M12 4a8 8 0 100 16 8 8 0 000-16z', 'M12 9.5a2.5 2.5 0 100 5 2.5 2.5 0 000-5z', 'M12 4v3M12 17v3M4 12h3M17 12h3'],
  marca: ['M20.5 12.5l-8 8-9-9V3.5h8z', 'M7.5 6.5a1 1 0 100 2 1 1 0 000-2z'],
  usb: ['M12 3v14M12 8l4 2v3M12 12l-4-2V8', 'M12 17a2 2 0 100 4 2 2 0 000-4z'],
  teclado: ['M4 6h16a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V8a2 2 0 012-2z', 'M6 10h.01M10 10h.01M14 10h.01M18 10h.01M7.5 14h9'],
  idioma: ['M12 3a9 9 0 100 18 9 9 0 000-18z', 'M3 12h18M12 3c3.2 3 3.2 15 0 18M12 3c-3.2 3-3.2 15 0 18'],
  escudo: ['M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z'],
  calidad: ['M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z', 'M9 12l2 2 4-4'],
  enchufe: ['M9 3v5M15 3v5M6 8h12v3a6 6 0 01-12 0zM12 17v4'],
  rayo: ['M13 2L4 14h7l-1 8 9-12h-7z'],
  capacidad: ['M12 3c3.9 0 7 1.3 7 3s-3.1 3-7 3-7-1.3-7-3 3.1-3 7-3z', 'M5 6v12c0 1.7 3.1 3 7 3s7-1.3 7-3V6M5 12c0 1.7 3.1 3 7 3s7-1.3 7-3'],
  resol: ['M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z', 'M3 9h18M9 3v18'],
  panel: ['M12 3l9 5-9 5-9-5zM3 13l9 5 9-5'],
  reloj: ['M12 3a9 9 0 100 18 9 9 0 000-18z', 'M12 7v5l3 2'],
  uso: ['M7 3h10a2 2 0 012 2v14a2 2 0 01-2 2H7a2 2 0 01-2-2V5a2 2 0 012-2z', 'M12 18h.01'],
  windows: ['M4 5.5l7-1v7H4zM13 4.3L20 3v8.5h-7zM4 13h7v7l-7-1zM13 13h7v8l-7-1z'],
};

function svgDe(trazos, clase = '') {
  const svg = document.createElementNS(NS_SVG, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('aria-hidden', 'true');
  if (clase) svg.setAttribute('class', clase);
  for (const trazo of trazos) {
    const camino = document.createElementNS(NS_SVG, 'path');
    camino.setAttribute('d', trazo);
    svg.appendChild(camino);
  }
  return svg;
}

// Solo se aceptan fotos que vengan de este mismo sitio o de una dirección segura; cualquier otra cosa se ignora.
function direccionSegura(imagen) {
  return /^(https:\/\/|\/static\/|\/)/.test(imagen) && !imagen.startsWith('//');
}

let abierta = null; // la ficha que está en pantalla (solo hay una)

export function cerrarFicha() {
  if (abierta) abierta.cerrar();
}

// Abre la ficha. "acciones": anadir(producto), preguntar(producto). "tarjeta": la tarjeta de la vitrina de donde se abrió
// (para devolverle el foco al cerrar).
export function abrirFicha(producto, acciones, tarjeta) {
  cerrarFicha();
  const [fuerte, medio, claro] = tonoDe(producto.categoria);
  const tinte = `linear-gradient(160deg, ${fuerte}, ${medio} 60%, ${claro})`;
  const tipo = iconoDe(producto);
  const variante = varianteDe(producto.nombre);
  const vistas = Array.isArray(producto.vistas) ? producto.vistas : [];
  const chips = Array.isArray(producto.chips) ? producto.chips : [];
  const grupos = Array.isArray(producto.grupos) ? producto.grupos : [];
  const agotado = producto.unidades <= 0;

  const velo = crear('div', 'fh-velo');
  const ficha = crear('section', 'fh');
  ficha.setAttribute('role', 'dialog');
  ficha.setAttribute('aria-modal', 'true');
  ficha.setAttribute('aria-label', producto.nombre);

  // ----- Lado de la foto -----
  const foto = crear('div', 'fh-foto');
  foto.style.background = tinte;
  const etiquetaCat = crear('span', 'fh-cat', producto.categoria);
  const asa = crear('span', 'fh-asa');
  asa.setAttribute('aria-hidden', 'true');
  const botonCerrarMovil = crear('button', 'fh-x fh-x-movil');
  botonCerrarMovil.type = 'button';
  botonCerrarMovil.setAttribute('aria-label', 'Cerrar');
  botonCerrarMovil.append(svgDe(['M6 6l12 12M18 6L6 18']));
  const rotuloVista = crear('span', 'fh-vista');
  rotuloVista.hidden = vistas.length === 0;
  foto.append(etiquetaCat, asa, rotuloVista, botonCerrarMovil);

  // Lo que se ve grande para una vista: un dibujo de estudio o una foto.
  function imagenDe(vista) {
    if (!vista) return dibujoDeProducto(tipo, variante);
    if (vista.imagen.startsWith('dibujo:')) return vistaDeProducto(tipo, variante, vista.imagen.slice(7));
    if (direccionSegura(vista.imagen)) {
      const imagen = crear('img', 'fh-foto-real');
      imagen.src = vista.imagen;
      imagen.alt = `${producto.nombre}: ${vista.etiqueta}`;
      return imagen;
    }
    return dibujoDeProducto(tipo, variante);
  }

  let principal = null;
  let elegida = 0;
  const miniaturas = [];

  function mostrarVista(indice) {
    elegida = indice;
    const vista = vistas[indice];
    const nueva = imagenDe(vista);
    nueva.classList.add('fh-principal');
    if (vista && vista.imagen === 'dibujo:empaque') nueva.classList.add('fh-chica');
    if (principal) principal.remove();
    foto.prepend(nueva);
    principal = nueva;
    if (vista) {
      rotuloVista.replaceChildren(vista.etiqueta);
      const puntos = crear('i');
      vistas.forEach((_, i) => puntos.append(crear('b', i === indice ? 'on' : '')));
      rotuloVista.append(puntos);
    }
    miniaturas.forEach((miniatura, i) => {
      miniatura.classList.toggle('on', i === indice);
      miniatura.setAttribute('aria-pressed', String(i === indice));
    });
  }

  function galeria() {
    const caja = crear('div', 'fh-gal');
    caja.append(crear('small', '', 'VISTAS DEL PRODUCTO'));
    const fila = crear('div', 'fh-g4');
    vistas.forEach((vista, i) => {
      const miniatura = crear('button', 'fh-th');
      miniatura.type = 'button';
      miniatura.style.background = tinte;
      miniatura.setAttribute('aria-label', vista.etiqueta);
      miniatura.title = vista.etiqueta;
      const dibujo = imagenDe(vista);
      miniatura.append(dibujo);
      miniatura.addEventListener('click', () => mostrarVista(i));
      miniatura.addEventListener('mouseenter', () => {
        if (window.matchMedia('(hover: hover)').matches && elegida !== i) mostrarVista(i);
      });
      miniaturas.push(miniatura);
      fila.append(miniatura);
    });
    caja.append(fila);
    return caja;
  }

  // ----- Lado derecho -----
  const derecha = crear('div', 'fh-rt');
  const botonCerrar = crear('button', 'fh-x');
  botonCerrar.type = 'button';
  botonCerrar.setAttribute('aria-label', 'Cerrar');
  botonCerrar.append(svgDe(['M6 6l12 12M18 6L6 18']));

  const cabecera = crear('div', 'fh-hd');
  cabecera.append(crear('p', 'fh-c0', producto.categoria), crear('h2', '', producto.nombre));

  const cuerpo = crear('div', 'fh-bd');
  const panelResumen = crear('div', 'fh-panel');
  const panelEspecificaciones = crear('div', 'fh-panel');

  // Resumen: las cajas con ícono (o, si el producto aún no las tiene, su descripción).
  if (chips.length > 0) {
    const rejilla = crear('div', chips.length === 4 ? 'fh-cg dos' : 'fh-cg');
    for (const chip of chips) {
      const icono = crear('span', 'fh-i');
      icono.append(svgDe(ICONOS_CAJA[chip.icono] || ICONOS_CAJA.marca));
      const texto = crear('span');
      texto.append(crear('small', '', chip.titulo), crear('b', '', chip.valor));
      const caja = crear('div');
      caja.append(icono, texto);
      rejilla.append(caja);
    }
    panelResumen.append(rejilla);
  } else {
    panelResumen.append(crear('p', 'fh-desc', producto.descripcion));
  }
  if (vistas.length > 0) panelResumen.append(galeria());

  // Especificaciones: la tabla por grupos.
  if (grupos.length > 0) {
    const filas = grupos.reduce((suma, g) => suma + g.filas.length, 0);
    panelEspecificaciones.append(crear('p', 'fh-cnt', `${filas} especificaciones en ${grupos.length} grupos`));
    for (const grupo of grupos) {
      const tabla = crear('div', 'fh-gr');
      tabla.append(crear('h4', '', grupo.nombre));
      for (const fila of grupo.filas) {
        const linea = crear('div', 'fh-f');
        linea.append(crear('i', '', fila.titulo), crear('b', '', fila.valor));
        tabla.append(linea);
      }
      panelEspecificaciones.append(tabla);
    }
  }
  cuerpo.append(panelResumen);

  // Pestañas (solo si hay tabla de especificaciones).
  let pestanas = null;
  if (grupos.length > 0) {
    ficha.classList.add('con-tabs');
    pestanas = crear('div', 'fh-tabs');
    pestanas.setAttribute('role', 'tablist');
    const opciones = [
      ['Resumen', panelResumen],
      ['Especificaciones', panelEspecificaciones],
    ];
    const botones = opciones.map(([nombre, panel], i) => {
      const boton = crear('button', i === 0 ? 'on' : '', nombre);
      boton.type = 'button';
      boton.setAttribute('role', 'tab');
      boton.setAttribute('aria-selected', String(i === 0));
      boton.addEventListener('click', () => {
        botones.forEach((otro, j) => {
          otro.classList.toggle('on', j === i);
          otro.setAttribute('aria-selected', String(j === i));
        });
        cuerpo.replaceChildren(panel);
        cuerpo.classList.toggle('sc', i === 1);
        cuerpo.scrollTop = 0;
      });
      pestanas.append(boton);
      return boton;
    });
  }

  // Pie: existencias, precio y botones.
  const pie = crear('div', 'fh-ft');
  const stock = existencias(producto.unidades);
  const estado = crear('span', `fh-est ${stock.clase}`);
  const circulo = crear('span', 'fh-o');
  circulo.append(iconoStock(stock.clase).firstChild);
  const textoEstado =
    stock.clase === 'no' ? 'Agotado por ahora' : stock.clase === 'poco' ? (producto.unidades === 1 ? 'Queda 1 unidad' : `Quedan ${producto.unidades} unidades`) : 'Disponible en tienda';
  estado.append(circulo, textoEstado);
  const precio = partirPrecio(producto.precio);
  const lineaPrecio = crear('div', 'fh-pr');
  lineaPrecio.append(crear('small', '', 'S/'), precio.entero);
  if (precio.decimales) lineaPrecio.append(crear('small', 'dec', precio.decimales));
  const fila = crear('div', 'fh-fr');
  fila.append(estado, lineaPrecio);

  const nota = crear('p', 'fh-nota', 'Añádelo al carrito y Juno te ayuda a completar tu pedido.');

  const botonesPie = crear('div', 'fh-bs');
  const principalBoton = crear('button', 'fh-p');
  principalBoton.type = 'button';
  if (agotado) {
    principalBoton.disabled = true;
    principalBoton.textContent = 'Agotado por ahora';
  } else {
    const juno = crear('img');
    juno.src = FOTO_JUNO;
    juno.alt = '';
    principalBoton.append(juno, 'Añadir al carrito con Juno');
    principalBoton.addEventListener('click', () => {
      cerrar();
      acciones.anadir(producto);
    });
  }
  const preguntar = crear('button', 'fh-s', 'Preguntarle a Juno');
  preguntar.type = 'button';
  preguntar.addEventListener('click', () => {
    cerrar();
    acciones.preguntar(producto);
  });
  botonesPie.append(principalBoton, preguntar);
  pie.append(fila, nota, botonesPie);

  derecha.append(botonCerrar, cabecera);
  if (pestanas) derecha.append(pestanas);
  derecha.append(cuerpo, pie);
  ficha.append(foto, derecha);

  mostrarVista(0);

  // ----- Abrir y cerrar -----
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
    if (tarjeta && tarjeta.isConnected) tarjeta.querySelector('.lz-prod-mas')?.focus({ preventScroll: true });
  }
  const manejador = { cerrar };
  abierta = manejador;

  velo.addEventListener('click', cerrar);
  botonCerrar.addEventListener('click', cerrar);
  botonCerrarMovil.addEventListener('click', cerrar);
  document.addEventListener('keydown', alTeclado);
  document.body.append(velo, ficha);
  botonCerrar.focus({ preventScroll: true });
}
