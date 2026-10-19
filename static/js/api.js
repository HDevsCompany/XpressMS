// api.js: la ÚNICA parte del frontend que habla con el servidor Rust.
// Si cambian las rutas del backend, solo se toca este archivo.

// Se lanza cuando el servidor ya no reconoce el token (por ejemplo, la conversación no existe).
export class ErrorSesion extends Error {}

// Envía un JSON por POST y devuelve la respuesta de fetch.
function enviarJson(ruta, cuerpo) {
  return fetch(ruta, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(cuerpo),
  });
}

// Al abrir la página: crea la conversación (o recupera la anterior si el token es válido).
// Devuelve { token, mensajes, opciones, componente }.
export async function iniciar(token) {
  const respuesta = await enviarJson('/agente/iniciar', { token });
  if (!respuesta.ok) throw new Error('Error ' + respuesta.status);
  return respuesta.json();
}

// Envía lo que la persona escribió. "accion" es opcional: la usan los elementos visuales
// (por ejemplo 'aceptar_terminos'). Devuelve { respuesta, opciones, componente, espera }.
export async function enviarMensaje(token, texto, accion = null) {
  const respuesta = await enviarJson('/agente/mensaje', { token, texto, accion });
  if (respuesta.status === 404) throw new ErrorSesion();
  if (!respuesta.ok) throw new Error('Error ' + respuesta.status);
  return respuesta.json();
}
