// burbujas.js: dibuja los mensajes y los botones rápidos dentro de la zona de conversación.
// Cada mensaje del agente lleva su icono y su nombre (como el saludo), para que la conversación tenga personalidad.

import { crear } from './dom.js';

const zona = document.getElementById('mensajes');

let agente = null; // { nombre, icono }: lo entrega el servidor (así se puede cambiar sin tocar esta página)
let turnoDelAgente = false; // true mientras lo último que se dibujó fue del agente (sus tarjetas y botones van alineados con su burbuja)

// Guarda los datos del agente. Hasta que se llame, los mensajes del agente salen como burbujas sin icono.
export function fijarAgente(datos) {
  agente = datos;
}

// Baja hasta el último elemento de la conversación.
export function bajar() {
  zona.scrollTop = zona.scrollHeight;
}

// Agrega un elemento cualquiera (burbuja, tarjeta...) al final de la conversación.
export function agregar(elemento) {
  if (elemento.classList.contains('saludo')) {
    turnoDelAgente = true;
    // El saludo de presentación lleva el icono más grande, así que su sangría también es mayor.
    zona.style.setProperty('--sangria-actual', elemento.classList.contains('grande') ? '68px' : '56px');
  } else if (elemento.classList.contains('burbuja') && elemento.classList.contains('persona')) {
    turnoDelAgente = false;
  } else if (turnoDelAgente) {
    elemento.classList.add('con-sangria'); // tarjetas y botones de esta respuesta: alineados con la burbuja, no con el icono
  }
  zona.appendChild(elemento);
  bajar();
  return elemento;
}

// Envuelve un contenido (burbuja, indicador de escritura...) en una fila con el icono del agente a la izquierda.
// Si todavía no se conocen los datos del agente, devuelve el contenido tal cual.
export function filaAgente(contenido, grande = false) {
  if (!agente) return contenido;
  const fila = crear('div', grande ? 'saludo grande' : 'saludo');
  const avatar = crear('div', 'avatar');
  const imagen = crear('img');
  imagen.src = agente.icono;
  imagen.alt = agente.nombre;
  avatar.appendChild(imagen);
  fila.append(avatar, contenido);
  return fila;
}

// Lo que el servidor escribe entre « » se muestra resaltado (en negrita sobre un fondo suave) y sin las comillas.
// Se arma con nodos de texto, nunca con innerHTML.
function conResaltado(texto) {
  const partes = document.createDocumentFragment();
  let desde = 0;
  for (const coincidencia of String(texto).matchAll(/«([^»]+)»/g)) {
    if (coincidencia.index > desde) partes.append(document.createTextNode(texto.slice(desde, coincidencia.index)));
    partes.append(crear('b', 'rs', coincidencia[1]));
    desde = coincidencia.index + coincidencia[0].length;
  }
  if (desde < texto.length) partes.append(document.createTextNode(texto.slice(desde)));
  return partes;
}

// Una burbuja del agente con su nombre arriba.
function burbujaDelAgente(texto) {
  const burbuja = crear('div', 'burbuja agente');
  if (agente) burbuja.append(crear('span', 'nombre-agente', agente.nombre));
  burbuja.append(conResaltado(texto));
  return burbuja;
}

// Agrega una burbuja de mensaje. "quien" es 'agente' o 'persona'.
export function agregarBurbuja(quien, texto) {
  if (quien === 'agente') return agregar(filaAgente(burbujaDelAgente(texto)));
  return agregar(crear('div', 'burbuja ' + quien, texto));
}

// Agrega el primer mensaje del agente (su presentación): igual que los demás, pero con el icono más grande.
export function agregarSaludo(datosAgente, texto) {
  fijarAgente(datosAgente);
  return agregar(filaAgente(burbujaDelAgente(texto), true));
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
