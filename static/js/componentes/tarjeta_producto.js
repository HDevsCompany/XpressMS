// tarjeta_producto.js: la tarjeta con el detalle de un producto (nombre, descripción, categoría, precio, unidades).
// Comparte el diseño de la tarjeta de servicio (static/css/servicio.css); solo cambia el icono y el contenido.

import { crear } from '../dom.js';
import { agregar } from '../burbujas.js';

const NS_SVG = 'http://www.w3.org/2000/svg';

// Icono de caja dibujado con SVG, para no depender de imágenes externas.
function crearIcono() {
  const dibujo = document.createElementNS(NS_SVG, 'svg');
  dibujo.setAttribute('viewBox', '0 0 24 24');
  dibujo.setAttribute('aria-hidden', 'true');
  const trazos = [
    'M21 8l-9-5-9 5 9 5 9-5z',
    'M3 8v8l9 5 9-5V8',
    'M12 13v8',
  ];
  for (const d of trazos) {
    const trazo = document.createElementNS(NS_SVG, 'path');
    trazo.setAttribute('d', d);
    dibujo.appendChild(trazo);
  }
  return dibujo;
}

// Una fila de la tarjeta: etiqueta pequeña arriba y el valor debajo.
function crearDato(etiqueta, valor, clase = '') {
  const dato = crear('div', 'servicio-dato ' + clase);
  dato.append(crear('dt', '', etiqueta), crear('dd', '', valor));
  return dato;
}

// "datos" es { nombre, descripcion, categoria, precio, disponibilidad } (lo envía el servidor).
export function mostrarTarjetaProducto(datos) {
  const tarjeta = crear('div', 'servicio');

  const cabecera = crear('div', 'servicio-cabecera');
  const icono = crear('span', 'servicio-icono');
  icono.appendChild(crearIcono());
  cabecera.append(icono, crear('h3', '', datos.nombre));

  const descripcion = crear('p', 'servicio-descripcion', datos.descripcion);

  const lista = crear('dl', 'servicio-datos');
  lista.appendChild(crearDato('Precio', datos.precio, 'precio'));
  lista.appendChild(crearDato('Categoría', datos.categoria));
  lista.appendChild(crearDato('Disponibilidad', datos.disponibilidad));

  tarjeta.append(cabecera, descripcion, lista);
  agregar(tarjeta);
}
