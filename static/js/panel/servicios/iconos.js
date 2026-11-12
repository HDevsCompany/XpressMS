// iconos.js (panel/servicios): los iconos de la pantalla, dibujados con SVG (no son emojis ni imágenes externas).
// Todos usan "currentColor": toman el color del texto del botón donde están.

const NS = 'http://www.w3.org/2000/svg';

const TRAZOS = {
  lapiz: ['M4 20h4L19.5 8.5a2.1 2.1 0 000-3l-1-1a2.1 2.1 0 00-3 0L4 16z', 'M13.5 6.5l4 4'],
  equis: ['M6 6l12 12', 'M18 6L6 18'],
  mas: ['M12 5v14', 'M5 12h14'],
  check: ['M5 12.5l4.5 4.5L19 7.5'],
  imagen: ['M5 4h14a1.5 1.5 0 011.5 1.5v13A1.5 1.5 0 0119 20H5a1.5 1.5 0 01-1.5-1.5v-13A1.5 1.5 0 015 4z', 'M3.5 16l5-5 4 4 3-3 5 5', 'M9 8.5h.01'],
  info: ['M12 3.5a8.5 8.5 0 100 17 8.5 8.5 0 000-17z', 'M12 11v5', 'M12 8h.01'],
  enviar: ['M5 12h14', 'M13 6l6 6-6 6'],
};

// Devuelve un <svg> listo para poner dentro de un botón. "grosor" es el ancho del trazo.
export function icono(nombre, grosor = 2.2) {
  const svg = document.createElementNS(NS, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('fill', 'none');
  svg.setAttribute('stroke', 'currentColor');
  svg.setAttribute('stroke-width', String(grosor));
  svg.setAttribute('stroke-linecap', 'round');
  svg.setAttribute('stroke-linejoin', 'round');
  svg.setAttribute('aria-hidden', 'true');
  for (const d of TRAZOS[nombre] || []) {
    const camino = document.createElementNS(NS, 'path');
    camino.setAttribute('d', d);
    svg.appendChild(camino);
  }
  return svg;
}
