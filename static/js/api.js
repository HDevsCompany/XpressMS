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

// Al abrir el chat: crea la conversación (o recupera la anterior si el token es válido).
// "motivo" dice por dónde llegó la persona ('registro' o 'mirar') y decide el saludo de Juno.
// Devuelve { token, agente: { nombre, icono }, mensajes, opciones, componente }.
export async function iniciar(token, motivo = null) {
  const respuesta = await enviarJson('/agente/iniciar', { token, motivo });
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

// Página de ingreso, lado «Soy cliente»: el servidor revisa documento + últimos 3 dígitos del celular.
// Si coinciden, vincula la conversación con el cliente y devuelve { token }. Si no, devuelve { mensaje }.
// A diferencia de las otras funciones, no lanza error por un 401 o 429: devuelve el estado para mostrar el mensaje.
// Devuelve { ok, estado, datos }.
export async function identificar(token, tipo, numero, ultimos) {
  const respuesta = await enviarJson('/agente/identificar', { token, tipo, numero, ultimos });
  let datos = null;
  try {
    datos = await respuesta.json();
  } catch {
    /* sin cuerpo JSON: se usa un mensaje genérico */
  }
  return { ok: respuesta.ok, estado: respuesta.status, datos };
}

// Página principal: ¿este navegador ya tiene una conversación con un cliente reconocido?
// Si falla la consulta, se responde false (se muestra la tarjeta de ingreso como siempre).
export async function sesionReconocida(token) {
  try {
    const respuesta = await enviarJson('/agente/sesion', { token });
    if (!respuesta.ok) return false;
    const datos = await respuesta.json();
    return datos.reconocido === true;
  } catch {
    return false;
  }
}

// El lienzo pregunta quién es: ¿este navegador ya tiene un cliente reconocido y cómo se llama?
// Devuelve { reconocido, nombre } ("nombre" solo viene si está reconocido). Si falla la consulta, se responde como
// visitante (reconocido: false): el lienzo se ve igual, solo sin el nombre.
export async function datosDeSesion(token) {
  try {
    const respuesta = await enviarJson('/agente/sesion', { token });
    if (!respuesta.ok) return { reconocido: false, nombre: null };
    const datos = await respuesta.json();
    return { reconocido: datos.reconocido === true, nombre: datos.nombre || null };
  } catch {
    return { reconocido: false, nombre: null };
  }
}

// «Nueva conversación» de un cliente reconocido: el servidor abre otra conversación ya vinculada a él y devuelve su token.
// Si el token no es de un cliente reconocido (o no hay conexión), lanza un error.
export async function nuevaConversacion(token) {
  const respuesta = await enviarJson('/agente/nueva', { token });
  if (!respuesta.ok) throw new Error('Error ' + respuesta.status);
  const datos = await respuesta.json();
  return datos.token;
}

// «Mi perfil y datos»: todo lo que muestra la página (datos, contacto, privacidad, direcciones, personas autorizadas).
// Devuelve { estado, perfil }: estado "ok" (con el perfil), "visitante" (el servidor no reconoce a un cliente: 401) o "error".
export async function datosDePerfil(token) {
  try {
    const respuesta = await enviarJson('/perfil/datos', { token });
    if (respuesta.status === 401) return { estado: 'visitante', perfil: null };
    if (!respuesta.ok) return { estado: 'error', perfil: null };
    return { estado: 'ok', perfil: await respuesta.json() };
  } catch {
    return { estado: 'error', perfil: null };
  }
}
