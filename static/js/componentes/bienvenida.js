// bienvenida.js: las tarjetas con las que empieza la conversación.
//   Ya soy cliente · Soy nuevo · Solo estoy de paso  -> envían un mensaje al agente (alElegir).
//   Portal de equipo                                -> es un enlace a /ingresar (el personal NO inicia sesión por el chat:
//                                                      los mensajes se guardan y una clave quedaría escrita en la conversación).
// Cada tarjeta tiene su color y su propia animación (menos de 1 segundo). Con "reducir movimiento"
// activado en el sistema, las animaciones se quitan (ver bienvenida.css) y el mensaje se envía sin espera.

import { crear } from '../dom.js';
import { agregar } from '../burbujas.js';

const NS = 'http://www.w3.org/2000/svg';

// Dibujos de los iconos (el mismo estilo de línea que en el resto de la plataforma).
const ICONOS = {
  llave: ['M10.8 12.2 20 3M16 7l3 3M14 9l2 2', { circulo: [8, 15, 4] }],
  documento: [
    'M6.5 16c.6-1.4 1.6-2 2.5-2s1.9.6 2.5 2M14 10h4M14 13h3',
    { rect: [3, 5, 18, 14, 3] },
    { circulo: [9, 11, 2] },
  ],
  ojo: ['M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7S2 12 2 12z', { circulo: [12, 12, 3] }],
  candado: ['M8 11V8a4 4 0 0 1 8 0v3', { rect: [5, 11, 14, 9, 2] }],
};

// Arma un <svg> con las piezas de ICONOS[nombre].
function icono(nombre) {
  const svg = document.createElementNS(NS, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('aria-hidden', 'true');
  ICONOS[nombre].forEach((pieza) => {
    if (typeof pieza === 'string') {
      const trazo = document.createElementNS(NS, 'path');
      trazo.setAttribute('d', pieza);
      svg.appendChild(trazo);
    } else if (pieza.circulo) {
      const [cx, cy, r] = pieza.circulo;
      const c = document.createElementNS(NS, 'circle');
      c.setAttribute('cx', cx);
      c.setAttribute('cy', cy);
      c.setAttribute('r', r);
      svg.appendChild(c);
    } else if (pieza.rect) {
      const [x, y, ancho, alto, radio] = pieza.rect;
      const rect = document.createElementNS(NS, 'rect');
      rect.setAttribute('x', x);
      rect.setAttribute('y', y);
      rect.setAttribute('width', ancho);
      rect.setAttribute('height', alto);
      rect.setAttribute('rx', radio);
      svg.appendChild(rect);
    }
  });
  return svg;
}

// Las tarjetas que envían un mensaje. "mensaje" es lo que escribe la persona al pulsarlas.
const OPCIONES = [
  { color: 'azul', icono: 'llave', titulo: 'Ya soy cliente', accion: 'Continuar →', mensaje: 'Ya soy cliente' },
  { color: 'verde', icono: 'documento', titulo: 'Soy nuevo', accion: 'Registrarme →', mensaje: 'Registrarme' },
  { color: 'ambar', icono: 'ojo', titulo: 'Solo estoy de paso', accion: '', mensaje: 'Solo estoy de paso' },
];

const reducirMovimiento = () => window.matchMedia('(prefers-reduced-motion: reduce)').matches;

// Duración de la animación al pulsar: se espera a que se vea antes de enviar el mensaje.
const ESPERA_ANIMACION_MS = 450;

// Dibuja las tarjetas. "alElegir" recibe el mensaje de la tarjeta pulsada.
export function mostrarBienvenida({ alElegir }) {
  const caja = crear('div', 'bienvenida');
  const rejilla = crear('div', 'bienvenida-tarjetas');
  const tarjetas = [];

  OPCIONES.forEach((opcion, posicion) => {
    const boton = crear('button', 't ' + opcion.color);
    boton.type = 'button';
    boton.style.setProperty('--i', posicion); // escalona la entrada de las tarjetas

    const cuadro = crear('span', 'ic');
    cuadro.appendChild(icono(opcion.icono));
    const textos = crear('span', 'textos');
    textos.appendChild(crear('span', 'titulo', opcion.titulo));
    if (opcion.accion) textos.appendChild(crear('span', 'ir', opcion.accion));
    boton.append(cuadro, textos);

    boton.addEventListener('click', () => {
      caja.classList.add('bloqueada'); // una sola elección: las demás se apagan
      boton.classList.add('elegida', 'anima');
      tarjetas.forEach((t) => (t.disabled = true));
      setTimeout(() => alElegir(opcion.mensaje), reducirMovimiento() ? 0 : ESPERA_ANIMACION_MS);
    });

    tarjetas.push(boton);
    rejilla.appendChild(boton);
  });

  // Portal de equipo: enlace a la pantalla de ingreso del personal.
  const portal = crear('a', 't violeta portal');
  portal.href = '/ingresar';
  const cuadro = crear('span', 'ic');
  cuadro.appendChild(icono('candado'));
  portal.append(cuadro, crear('span', 'titulo', 'Portal de equipo'));
  portal.addEventListener('click', () => portal.classList.add('anima'));

  caja.append(rejilla, portal);
  agregar(caja);
}
