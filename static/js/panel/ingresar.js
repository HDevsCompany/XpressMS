// ingresar.js: el lado «Portal de equipo» de la página /ingresar. Envía usuario y contraseña; si son correctos, va al panel.

import { entrar } from './api.js';

const formulario = document.getElementById('formulario-ingreso');
const campoUsuario = document.getElementById('usuario');
const campoClave = document.getElementById('clave');
const mensajeError = document.getElementById('error');
const boton = document.getElementById('boton-ingresar');

function mostrarError(texto) {
  mensajeError.textContent = texto;
  mensajeError.hidden = false;
}

formulario.addEventListener('submit', async (evento) => {
  evento.preventDefault(); // evita que el navegador recargue la página
  mensajeError.hidden = true;
  boton.disabled = true;
  boton.textContent = 'Ingresando…';

  try {
    const resultado = await entrar(campoUsuario.value.trim(), campoClave.value);
    if (resultado.ok) {
      (window.top || window).location.href = '/panel'; // si va en la ventana de la página principal, cambia la página completa
      return;
    }
    // El servidor manda el texto del error ("Usuario o contraseña incorrectos.", "Demasiados intentos...").
    mostrarError(resultado.datos?.mensaje || 'No se pudo ingresar. Inténtalo de nuevo.');
    campoClave.value = '';
    campoClave.focus();
  } catch {
    mostrarError('No hay conexión con el servidor. Revisa tu internet e inténtalo de nuevo.');
  }

  boton.disabled = false;
  boton.textContent = 'Ingresar';
});
