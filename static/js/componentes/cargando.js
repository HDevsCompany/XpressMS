// cargando.js: indicador de que el agente está trabajando.
//  - Sin texto: tres puntos que rebotan ("el agente está escribiendo").
//  - Con texto: círculo giratorio y el texto ("Consultando tus datos…").

import { crear } from '../dom.js';
import { agregar } from '../burbujas.js';

// Devuelve una función "quitar" que borra el indicador cuando llega la respuesta.
export function mostrarCargando(texto) {
  const burbuja = crear('div', 'burbuja agente cargando');
  burbuja.setAttribute('role', 'status'); // los lectores de pantalla lo anuncian

  if (texto) {
    burbuja.appendChild(crear('span', 'spinner'));
    burbuja.appendChild(crear('span', '', texto));
  } else {
    const puntos = crear('span', 'puntos');
    puntos.append(crear('span'), crear('span'), crear('span'));
    burbuja.appendChild(puntos);
  }

  agregar(burbuja);
  return () => burbuja.remove();
}
