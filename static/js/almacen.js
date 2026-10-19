// almacen.js: guarda el token de la conversación en el navegador (localStorage), así,
// si recargas la página, el agente te reconoce y muestra tu historial.
// Cada acceso va dentro de try/catch porque algunos navegadores bloquean el almacenamiento
// (por ejemplo, en ventanas privadas): en ese caso la página sigue funcionando, sin memoria.

const CLAVE_TOKEN = 'xpressms_token';

export function leerToken() {
  try {
    return localStorage.getItem(CLAVE_TOKEN);
  } catch (error) {
    return null;
  }
}

export function guardarToken(token) {
  try {
    localStorage.setItem(CLAVE_TOKEN, token);
  } catch (error) {
    /* sin almacenamiento: seguimos igual */
  }
}

export function borrarToken() {
  try {
    localStorage.removeItem(CLAVE_TOKEN);
  } catch (error) {
    /* sin almacenamiento: nada que borrar */
  }
}
