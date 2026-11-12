// catalogo_comun.js: lo que comparten la vitrina (catalogo.js), la ficha del producto (ficha.js) y el carrito (carrito.js):
// qué dibujo lleva cada producto, el color pastel de su categoría, cómo se parte el precio y el rótulo de existencias.

import { crear } from '../dom.js';

const NS_SVG = 'http://www.w3.org/2000/svg';

// Qué dibujo lleva un producto, según las palabras de su nombre (sin tildes).
const REGLAS_ICONO = [
  ['bateria', ['bateria', 'cargador', 'adaptador']], // antes que «laptop»: «Batería para laptop» es una batería
  ['pantalla', ['pantalla', 'monitor', 'display']],
  ['ssd', ['ssd', 'disco', 'nvme', 'memoria usb', 'pendrive']],
  ['ram', ['ram', 'memoria']],
  ['mouse', ['mouse', 'raton']],
  ['teclado', ['teclado']],
  ['audio', ['audifono', 'auricular', 'parlante', 'headset']],
  ['cable', ['cable', 'hdmi', 'usb']],
  ['laptop', ['laptop', 'notebook', 'portatil']],
];

// Colores pastel fijos por categoría (tres tonos del fondo de la foto, de más fuerte a más claro).
// La clave es el nombre de la categoría sin tildes y en minúsculas. Para dar color a una categoría nueva, basta una línea aquí.
const COLORES_CATEGORIA = {
  laptops: ['#c7d2fe', '#e0e7ff', '#f5f7ff'], // azul violáceo
  repuestos: ['#99f6e4', '#ccfbf1', '#f0fdfa'], // verde menta
  accesorios: ['#fbcfe8', '#fce7f3', '#fff5fa'], // rosa
};
const COLOR_NEUTRO = ['#cbd5e1', '#e2e8f0', '#f8fafc']; // gris azulado, para las categorías que aún no tienen color

export function sinTildes(texto) {
  return texto.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();
}

export function iconoDe(producto) {
  const nombre = sinTildes(`${producto.nombre} ${producto.categoria}`);
  for (const [icono, palabras] of REGLAS_ICONO) {
    if (palabras.some((palabra) => nombre.includes(palabra))) return icono;
  }
  return 'caja';
}

export function tonoDe(categoria) {
  return COLORES_CATEGORIA[sinTildes(categoria).trim()] || COLOR_NEUTRO;
}

// Las laptops alternan entre dos colores de pantalla; el producto siempre sale con el mismo.
export function varianteDe(nombre) {
  let marca = 0;
  for (const letra of nombre) marca = (marca * 31 + letra.charCodeAt(0)) % 65521;
  return marca % 2;
}

// "S/ 1899.00" -> entero "1,899" y, si los céntimos no son cero, ".50".
export function partirPrecio(precio) {
  const limpio = precio.replace('S/', '').trim();
  const [entero, decimales = '00'] = limpio.split('.');
  const miles = entero.replace(/\B(?=(\d{3})+(?!\d))/g, ','); // en el Perú los miles llevan coma: S/ 1,899
  return { entero: miles, decimales: decimales === '00' ? '' : `.${decimales}` };
}

// Los iconitos del rótulo de existencias (trazos de SVG fijos): una palomita, un triángulo de aviso y una X; los tres con el mismo formato: figura de trazo fino con su marca adentro.
const CIRCULO = 'M12 3.5a8.5 8.5 0 100 17 8.5 8.5 0 000-17z'; // mismo tamaño y trazo que el triángulo de aviso
const ICONOS_STOCK = {
  ok: [CIRCULO, 'M8 12.4l2.8 2.8 5.2-5.7'],
  poco: ['M12 4.5l8.5 15h-17z', 'M12 10v4', 'M12 16.8h.01'],
  no: [CIRCULO, 'M9.2 9.2l5.6 5.6', 'M14.8 9.2l-5.6 5.6'],
};

export function iconoStock(clase) {
  const envoltura = crear('span', 'lz-prod-aviso');
  const svg = document.createElementNS(NS_SVG, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('aria-hidden', 'true');
  for (const trazo of ICONOS_STOCK[clase]) {
    const camino = document.createElementNS(NS_SVG, 'path');
    camino.setAttribute('d', trazo);
    svg.appendChild(camino);
  }
  envoltura.appendChild(svg);
  return envoltura;
}

// El rótulo de existencias: disponible (palomita verde), pocas unidades (aviso que late suavemente) o agotado (X roja).
export function existencias(unidades) {
  if (unidades <= 0) return { clase: 'no', texto: 'Agotado' };
  if (unidades <= 3) return { clase: 'poco', texto: unidades === 1 ? 'Última unidad' : `Últimas ${unidades} unidades` };
  return { clase: 'ok', texto: 'Disponible' };
}

