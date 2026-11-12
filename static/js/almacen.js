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

// El tema de color del chat (lo escribe static/js/tema.js). Un cliente reconocido no tiene tema: se borra.
const CLAVE_TEMA = 'xpressms_tema';

export function borrarTema() {
  try {
    localStorage.removeItem(CLAVE_TEMA);
  } catch (error) {
    /* sin almacenamiento: nada que borrar */
  }
}

export function borrarToken() {
  try {
    localStorage.removeItem(CLAVE_TOKEN);
    localStorage.removeItem(CLAVE_TOKEN_TIENDA); // quien cierra sesión no deja a Juno de la tienda reconociéndolo
    localStorage.removeItem(CLAVE_TOKEN_SERVICIOS); // ni a Juno de la página de servicios
  } catch (error) {
    /* sin almacenamiento: nada que borrar */
  }
}

// La ventana de Juno en la tienda (/catalogo) tiene su propia conversación, aparte del chat principal.
const CLAVE_TOKEN_TIENDA = 'xpressms_token_tienda';

export function leerTokenTienda() {
  try {
    return localStorage.getItem(CLAVE_TOKEN_TIENDA);
  } catch (error) {
    return null;
  }
}

export function guardarTokenTienda(token) {
  try {
    localStorage.setItem(CLAVE_TOKEN_TIENDA, token);
  } catch (error) {
    /* sin almacenamiento: la conversación de la tienda dura lo que la página */
  }
}

// La ventana de Juno en la página de servicios (/servicios) también tiene su propia conversación.
const CLAVE_TOKEN_SERVICIOS = 'xpressms_token_servicios';

export function leerTokenServicios() {
  try {
    return localStorage.getItem(CLAVE_TOKEN_SERVICIOS);
  } catch (error) {
    return null;
  }
}

export function guardarTokenServicios(token) {
  try {
    localStorage.setItem(CLAVE_TOKEN_SERVICIOS, token);
  } catch (error) {
    /* sin almacenamiento: la conversación de servicios dura lo que la página */
  }
}

// ---------- Del perfil al lienzo ----------
// «Mi perfil y datos» solo muestra: cuando la persona pide algo («Corregir con Juno»), la página deja el mensaje aquí
// (sessionStorage: dura lo que la pestaña) y abre el lienzo, que lo envía a Juno apenas arranca.
const CLAVE_PENDIENTE = 'xpressms_mensaje_pendiente';
const CLAVE_NUEVA = 'xpressms_nueva_conversacion';
const CLAVE_DESTINO = 'xpressms_destino';

function dejar(clave, valor) {
  try {
    sessionStorage.setItem(clave, valor);
  } catch (error) {
    /* sin almacenamiento: la petición se pierde, la persona la escribe en el lienzo */
  }
}

function tomar(clave) {
  try {
    const valor = sessionStorage.getItem(clave);
    sessionStorage.removeItem(clave);
    return valor;
  } catch (error) {
    return null;
  }
}

export const dejarMensajePendiente = (mensaje) => dejar(CLAVE_PENDIENTE, mensaje);
export const tomarMensajePendiente = () => tomar(CLAVE_PENDIENTE);
export const dejarNuevaConversacion = () => dejar(CLAVE_NUEVA, '1');
export const tomarNuevaConversacion = () => tomar(CLAVE_NUEVA) === '1';
// Adónde volver después de iniciar sesión (la ventana de ingreso siempre termina en el lienzo).
export const dejarDestino = (ruta) => dejar(CLAVE_DESTINO, ruta);
export const tomarDestino = () => tomar(CLAVE_DESTINO);
