// burbujas.js: dibuja los mensajes y los botones rápidos dentro de la zona de conversación.

import { crear } from './dom.js';

const zona = document.getElementById('mensajes');

// Baja hasta el último elemento de la conversación.
export function bajar() {
  zona.scrollTop = zona.scrollHeight;
}

// Agrega un elemento cualquiera (burbuja, tarjeta...) al final de la conversación.
export function agregar(elemento) {
  zona.appendChild(elemento);
  bajar();
  return elemento;
}

// Agrega una burbuja de mensaje. "quien" es 'agente' o 'persona'.
export function agregarBurbuja(quien, texto) {
  return agregar(crear('div', 'burbuja ' + quien, texto));
}

// Quita los botones rápidos anteriores (para que no se acumulen).
export function limpiarOpciones() {
  zona.querySelectorAll('.opciones').forEach((caja) => caja.remove());
}

// Muestra botones rápidos. "alElegir" recibe el texto del botón pulsado.
export function mostrarOpciones(opciones, alElegir) {
  if (!opciones || opciones.length === 0) return;
  const caja = crear('div', 'opciones');
  opciones.forEach((texto) => {
    const boton = crear('button', '', texto);
    boton.type = 'button';
    boton.addEventListener('click', () => alElegir(texto));
    caja.appendChild(boton);
  });
  agregar(caja);
}
