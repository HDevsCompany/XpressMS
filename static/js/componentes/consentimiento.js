// consentimiento.js: la tarjeta con la casilla "Acepto los Términos y Condiciones".
// Reglas:
//  - "Continuar" solo se activa cuando la casilla está marcada.
//  - El enlace "Términos y Condiciones" abre la ventana con el texto legal (sin marcar la casilla).
//  - Al responder, la tarjeta se bloquea y muestra el resultado (así no se puede aceptar dos veces).

import { crear } from '../dom.js';
import { agregar } from '../burbujas.js';
import { abrirTerminos } from './modal_terminos.js';

// "alAceptar" y "alRechazar" son funciones que decide quien usa la tarjeta (chat.js).
export function mostrarConsentimiento({ alAceptar, alRechazar }) {
  const tarjeta = crear('div', 'tarjeta');

  // --- Casilla + texto con enlace ---
  const etiqueta = crear('label', 'casilla');
  const casilla = crear('input');
  casilla.type = 'checkbox';
  const texto = crear('span');
  const enlace = crear('button', 'enlace', 'Términos y Condiciones');
  enlace.type = 'button';
  enlace.addEventListener('click', (evento) => {
    evento.preventDefault(); // dentro de un <label>, evita que el clic marque la casilla
    evento.stopPropagation();
    abrirTerminos();
  });
  texto.append('Acepto los ', enlace, ' y el tratamiento de mis datos personales.');
  etiqueta.append(casilla, texto);

  // --- Botones ---
  const acciones = crear('div', 'acciones');
  const continuar = crear('button', 'boton-primario', 'Continuar');
  continuar.type = 'button';
  continuar.disabled = true; // se activa al marcar la casilla
  const rechazar = crear('button', 'boton-secundario', 'No, gracias');
  rechazar.type = 'button';
  acciones.append(continuar, rechazar);

  casilla.addEventListener('change', () => {
    continuar.disabled = !casilla.checked;
  });

  // Reemplaza los botones por el resultado y bloquea la tarjeta.
  function terminar(aceptado) {
    casilla.disabled = true;
    acciones.remove();
    const estado = crear('div', 'estado-final ' + (aceptado ? 'aceptado' : 'rechazado'));
    estado.append(crear('span', 'marca', aceptado ? '✓' : '–'), aceptado ? 'Aceptaste los términos' : 'No aceptaste los términos');
    tarjeta.appendChild(estado);
  }

  continuar.addEventListener('click', () => {
    terminar(true);
    alAceptar();
  });
  rechazar.addEventListener('click', () => {
    terminar(false);
    alRechazar();
  });

  tarjeta.append(etiqueta, acciones);
  agregar(tarjeta);
}
