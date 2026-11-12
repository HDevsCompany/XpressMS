// cliente.js: el formulario «Soy cliente» de la página de ingreso.
// Envía documento + últimos 3 dígitos del celular al servidor; si coinciden, el servidor deja la conversación
// vinculada al cliente y devuelve su token. Se guarda y se va al chat, donde Juno ya lo reconoce.

import { identificar } from '../api.js';
import { borrarTema, guardarToken, leerToken } from '../almacen.js';

const formulario = document.getElementById('formulario-cliente');
const campoTipo = document.getElementById('tipo-documento');
const campoNumero = document.getElementById('numero-documento');
const campoUltimos = document.getElementById('ultimos-digitos');
const mensajeError = document.getElementById('error-cliente');
const boton = document.getElementById('boton-cliente');

// Cada tipo de documento tiene su propio largo y ayuda de escritura.
const AYUDAS = {
  1: { texto: '8 dígitos', largo: 8, numerico: true }, // DNI
  6: { texto: '11 dígitos', largo: 11, numerico: true }, // RUC
  4: { texto: 'Número de tu carnet', largo: 12, numerico: false }, // Carnet de extranjería
  7: { texto: 'Número de tu pasaporte', largo: 12, numerico: false }, // Pasaporte
};

function ajustarNumero() {
  const ayuda = AYUDAS[campoTipo.value];
  campoNumero.placeholder = ayuda.texto;
  campoNumero.maxLength = ayuda.largo + 3; // un poco de margen para espacios o guiones
  campoNumero.inputMode = ayuda.numerico ? 'numeric' : 'text';
}
campoTipo.addEventListener('change', () => {
  ajustarNumero();
  campoNumero.focus();
});
ajustarNumero();

// Los 3 dígitos aceptan solo números.
campoUltimos.addEventListener('input', () => {
  campoUltimos.value = campoUltimos.value.replace(/\D/g, '').slice(0, 3);
});

function mostrarError(texto) {
  mensajeError.textContent = texto;
  mensajeError.hidden = false;
}

formulario.addEventListener('submit', async (evento) => {
  evento.preventDefault();
  mensajeError.hidden = true;

  if (campoNumero.value.trim() === '') {
    mostrarError('Escribe tu número de documento.');
    campoNumero.focus();
    return;
  }
  if (campoUltimos.value.length !== 3) {
    mostrarError('Escribe los últimos 3 dígitos de tu celular.');
    campoUltimos.focus();
    return;
  }

  boton.disabled = true;
  boton.textContent = 'Verificando…';
  try {
    const resultado = await identificar(leerToken(), campoTipo.value, campoNumero.value.trim(), campoUltimos.value);
    if (resultado.ok) {
      guardarToken(resultado.datos.token);
      borrarTema(); // el cliente reconocido ve el chat en el azul de siempre
      (window.top || window).location.href = '/chat'; // si va en la ventana de la página principal, cambia la página completa
      return;
    }
    // El servidor manda el texto del error ("No pudimos verificar tus datos…", "Demasiados intentos…").
    mostrarError(resultado.datos?.mensaje || 'No se pudo verificar. Inténtalo de nuevo.');
    campoUltimos.value = '';
    campoUltimos.focus();
  } catch {
    mostrarError('No hay conexión con el servidor. Revisa tu internet e inténtalo de nuevo.');
  }
  boton.disabled = false;
  boton.textContent = 'Continuar';
});
