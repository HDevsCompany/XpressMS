// tarjeta_apartado.js: la tarjeta de un producto apartado: qué es, su estado, el código y los datos para recogerlo.
// También sirve para la confirmación (todavía sin código). Comparte el diseño de la tarjeta de orden
// (static/css/orden.css); solo cambia el icono y las filas de datos.

import { crear } from '../dom.js';
import { agregar } from '../burbujas.js';

const NS_SVG = 'http://www.w3.org/2000/svg';

// Icono de bolsa de compra dibujado con SVG, para no depender de imágenes externas.
function crearIcono() {
  const dibujo = document.createElementNS(NS_SVG, 'svg');
  dibujo.setAttribute('viewBox', '0 0 24 24');
  dibujo.setAttribute('aria-hidden', 'true');
  const trazos = [
    'M6 2L3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z',
    'M3 6h18',
    'M16 10a4 4 0 0 1-8 0',
  ];
  for (const d of trazos) {
    const trazo = document.createElementNS(NS_SVG, 'path');
    trazo.setAttribute('d', d);
    dibujo.appendChild(trazo);
  }
  return dibujo;
}

// "datos" es { codigo?, estado, producto, campos: [{ etiqueta, valor }] } (lo envía el servidor).
export function mostrarTarjetaApartado(datos) {
  const tarjeta = crear('div', 'orden');

  const cabecera = crear('div', 'orden-cabecera');
  const icono = crear('span', 'orden-icono');
  icono.appendChild(crearIcono());
  const titulo = crear('div', 'orden-titulo');
  titulo.append(crear('h3', '', datos.producto), crear('span', 'orden-estado', datos.estado));
  cabecera.append(icono, titulo);
  tarjeta.append(cabecera);

  // El código solo existe cuando el apartado ya está guardado.
  if (datos.codigo) {
    const codigo = crear('div', 'orden-codigo');
    codigo.append(crear('span', '', 'Código de apartado'), crear('strong', '', datos.codigo));
    tarjeta.append(codigo);
  }

  const lista = crear('dl', 'orden-datos');
  for (const campo of datos.campos || []) {
    const dato = crear('div', 'orden-dato completo');
    dato.append(crear('dt', '', campo.etiqueta), crear('dd', '', campo.valor));
    lista.appendChild(dato);
  }
  tarjeta.appendChild(lista);

  agregar(tarjeta);
}
