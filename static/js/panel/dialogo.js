// dialogo.js: la ventana emergente (<dialog>) que usan los formularios y las confirmaciones.
// El elemento <dialog> ya trae lo difícil: bloquea la página de fondo, se cierra con Esc y atrapa el foco.

import { crear } from '../dom.js';

const dialogo = document.getElementById('dialogo');

// Muestra un contenido (un elemento ya armado) dentro de la ventana.
export function abrirDialogo(contenido) {
  dialogo.replaceChildren(contenido);
  if (!dialogo.open) dialogo.showModal();
}

export function cerrarDialogo() {
  if (dialogo.open) dialogo.close();
}

// Un botón "Cancelar" y otro de acción, para el pie de los formularios.
export function crearBotones(textoAccion, estiloAccion = 'boton-principal') {
  const botones = crear('div', 'dialogo-botones');
  const cancelar = crear('button', 'boton-secundario', 'Cancelar');
  cancelar.type = 'button';
  cancelar.addEventListener('click', cerrarDialogo);
  const accion = crear('button', estiloAccion, textoAccion);
  accion.type = 'submit';
  botones.append(cancelar, accion);
  return { botones, accion };
}

// Pregunta "¿seguro?" y devuelve una promesa: true si confirmó, false si canceló o cerró con Esc.
export function confirmar(titulo, mensaje, textoBoton) {
  return new Promise((resolver) => {
    const cuerpo = crear('div', 'dialogo-cuerpo');
    cuerpo.append(crear('h2', '', titulo), crear('p', '', mensaje));

    const botones = crear('div', 'dialogo-botones');
    const cancelar = crear('button', 'boton-secundario', 'Cancelar');
    const aceptar = crear('button', 'boton-peligro lleno', textoBoton);
    cancelar.type = aceptar.type = 'button';
    botones.append(cancelar, aceptar);
    cuerpo.append(botones);

    let decidido = false;
    const decidir = (valor) => {
      if (decidido) return;
      decidido = true;
      cerrarDialogo();
      resolver(valor);
    };
    aceptar.addEventListener('click', () => decidir(true));
    cancelar.addEventListener('click', () => decidir(false));
    dialogo.addEventListener('close', () => decidir(false), { once: true }); // Esc
    abrirDialogo(cuerpo);
    cancelar.focus(); // el foco empieza en lo seguro: Cancelar
  });
}
