// api.js (panel): la ÚNICA parte del panel que habla con el servidor Rust.
// Si cambian las rutas del backend, solo se toca este archivo.
// Cada función devuelve { ok, estado, datos }: "datos" trae el JSON de la respuesta (o null si no hay).

async function pedir(ruta, opciones = {}) {
  const respuesta = await fetch(ruta, opciones);
  let datos = null;
  // Las respuestas sin contenido (201, 204) no traen JSON; las demás pueden traer datos o el mensaje de error.
  if ((respuesta.headers.get('content-type') || '').includes('application/json')) {
    try {
      datos = await respuesta.json();
    } catch {
      datos = null; // el cuerpo vino vacío
    }
  }
  return { ok: respuesta.ok, estado: respuesta.status, datos };
}

function enviarJson(ruta, cuerpo) {
  return pedir(ruta, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(cuerpo),
  });
}

// Usuario y clave. Si son correctos, el servidor guarda la cookie de sesión (el script no la ve).
export function entrar(usuario, clave) {
  return enviarJson('/acceso/personal/entrar', { usuario, clave });
}

export function salir() {
  return pedir('/acceso/personal/salir', { method: 'POST' });
}

// Quién está en sesión: { codigo, usuario, nombre, roles, debe_cambiar_clave }.
export function quienSoy() {
  return pedir('/acceso/personal/yo');
}

export function cambiarClave(actual, nueva) {
  return enviarJson('/acceso/personal/cambiar-clave', { actual, nueva });
}

// ---------- Administración (solo Administrador) ----------
// Estas funciones envían al navegador de vuelta a /ingresar si la sesión venció (401),
// o al inicio del panel si la cuenta no tiene permiso (403).

function enviar(metodo, ruta, cuerpo) {
  const opciones = { method: metodo };
  if (cuerpo !== undefined) {
    opciones.headers = { 'Content-Type': 'application/json' };
    opciones.body = JSON.stringify(cuerpo);
  }
  return pedir(ruta, opciones);
}

async function protegido(promesa) {
  const resultado = await promesa;
  if (resultado.estado === 401) window.location.href = '/ingresar';
  if (resultado.estado === 403) window.location.href = '/panel';
  return resultado;
}

export const listarEquipos = () => protegido(enviar('GET', '/panel/api/equipos'));
export const crearEquipo = (nombre) => protegido(enviar('POST', '/panel/api/equipos', { nombre }));
export const renombrarEquipo = (codigo, nombre) =>
  protegido(enviar('PUT', '/panel/api/equipos/' + encodeURIComponent(codigo), { nombre }));
export const eliminarEquipo = (codigo) =>
  protegido(enviar('DELETE', '/panel/api/equipos/' + encodeURIComponent(codigo)));

export const listarServicios = () => protegido(enviar('GET', '/panel/api/servicios'));
// "datos": { nombre, descripcion, precio_base, tiempo_estimado, equipos: [códigos] }
export const crearServicio = (datos) => protegido(enviar('POST', '/panel/api/servicios', datos));
export const actualizarServicio = (codigo, datos) =>
  protegido(enviar('PUT', '/panel/api/servicios/' + encodeURIComponent(codigo), datos));
export const eliminarServicio = (codigo) =>
  protegido(enviar('DELETE', '/panel/api/servicios/' + encodeURIComponent(codigo)));

// La imagen del servicio: el cuerpo ES la imagen (JPG, PNG o WEBP). El navegador la reduce antes de enviarla.
export const subirImagenServicio = (codigo, imagen) =>
  protegido(
    pedir('/panel/api/servicios/' + encodeURIComponent(codigo) + '/imagen', {
      method: 'PUT',
      headers: { 'Content-Type': imagen.type || 'image/jpeg' },
      body: imagen,
    }),
  );
export const quitarImagenServicio = (codigo) =>
  protegido(enviar('DELETE', '/panel/api/servicios/' + encodeURIComponent(codigo) + '/imagen'));

// ---------- Juno en el panel de servicios (solo Administrador) ----------
// Se envía { texto, accion, estado } y vuelve { respuesta, opciones, estado, borrador, hecho, foco }.
// El "estado" lo guarda el navegador y se lo devuelve a Juno en cada mensaje.
export const conversarJunoServicios = (cuerpo) => protegido(enviar('POST', '/panel/api/juno/servicios', cuerpo));

// ---------- Órdenes de servicio (Administrador y Técnico) ----------
// El Administrador ve todas las órdenes; el Técnico, solo las asignadas a él (lo decide el servidor).
// Una orden: { codigo, cliente, telefono, servicio, equipo, problema, fecha_cita, estado_codigo, estado_nombre,
//              tecnico_codigo, tecnico_nombre }.
// El detalle trae { orden, historial: [{ estado, fecha, usuario, nota }], siguientes: [{ codigo, nombre }] }.
export const listarOrdenes = () => protegido(enviar('GET', '/panel/api/ordenes'));
export const detalleOrden = (codigo) =>
  protegido(enviar('GET', '/panel/api/ordenes/' + encodeURIComponent(codigo)));
// Técnicos a los quienes se puede asignar (solo Administrador): [{ codigo, nombre }].
export const listarTecnicos = () => protegido(enviar('GET', '/panel/api/tecnicos'));
// "tecnico" es el código del técnico, o null para dejar la orden sin técnico.
export const asignarTecnico = (codigo, tecnico) =>
  protegido(enviar('PUT', '/panel/api/ordenes/' + encodeURIComponent(codigo) + '/tecnico', { tecnico }));
// "estado" es el código del nuevo estado; "nota" es opcional.
export const cambiarEstadoOrden = (codigo, estado, nota) =>
  protegido(enviar('PUT', '/panel/api/ordenes/' + encodeURIComponent(codigo) + '/estado', { estado, nota: nota || null }));

// ---------- Productos, categorías y apartados (Administrador y Vendedor) ----------
// Categoría: { codigo, nombre, productos } (productos = cuántos activos tiene).
// Producto: { codigo, categoria_codigo, categoria_nombre, nombre, descripcion, precio: "1299.00", stock }.
// "datos" del producto: { categoria, nombre, descripcion, precio, stock } (categoria = código).
// Apartado: { codigo, producto, cantidad, precio_unitario, total, estado_codigo, estado_nombre, vence, cliente, telefono }.
export const listarCategorias = () => protegido(enviar('GET', '/panel/api/categorias'));
export const crearCategoria = (nombre) => protegido(enviar('POST', '/panel/api/categorias', { nombre }));
export const renombrarCategoria = (codigo, nombre) =>
  protegido(enviar('PUT', '/panel/api/categorias/' + encodeURIComponent(codigo), { nombre }));
export const eliminarCategoria = (codigo) =>
  protegido(enviar('DELETE', '/panel/api/categorias/' + encodeURIComponent(codigo)));

export const listarProductos = () => protegido(enviar('GET', '/panel/api/productos'));
export const crearProducto = (datos) => protegido(enviar('POST', '/panel/api/productos', datos));
export const actualizarProducto = (codigo, datos) =>
  protegido(enviar('PUT', '/panel/api/productos/' + encodeURIComponent(codigo), datos));
export const eliminarProducto = (codigo) =>
  protegido(enviar('DELETE', '/panel/api/productos/' + encodeURIComponent(codigo)));

export const listarApartados = () => protegido(enviar('GET', '/panel/api/apartados'));
export const entregarApartado = (codigo) =>
  protegido(enviar('POST', '/panel/api/apartados/' + encodeURIComponent(codigo) + '/entregar'));
export const cancelarApartado = (codigo) =>
  protegido(enviar('POST', '/panel/api/apartados/' + encodeURIComponent(codigo) + '/cancelar'));

// ---------- Personal (solo Administrador) ----------
// "datos" del alta: { tipo_documento, numero_documento, paterno, materno, nombres, telefono, correo, usuario, roles }.
// "datos" de la edición: { paterno, materno, nombres, telefono, correo, roles }.
// Al dar de alta y al restablecer, la respuesta trae { usuario, clave_temporal } (se muestra UNA sola vez).
export const listarPersonal = () => protegido(enviar('GET', '/panel/api/personal'));
export const crearPersonal = (datos) => protegido(enviar('POST', '/panel/api/personal', datos));
export const actualizarPersonal = (codigo, datos) =>
  protegido(enviar('PUT', '/panel/api/personal/' + encodeURIComponent(codigo), datos));
export const restablecerClavePersonal = (codigo) =>
  protegido(enviar('POST', '/panel/api/personal/' + encodeURIComponent(codigo) + '/restablecer-clave'));
export const eliminarPersonal = (codigo) =>
  protegido(enviar('DELETE', '/panel/api/personal/' + encodeURIComponent(codigo)));
