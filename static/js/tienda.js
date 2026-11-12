// tienda.js: la página /catalogo (la tienda). Solo MUESTRA: pide los productos al servidor y los dibuja; no tiene buscador ni filtros.
// Quien filtra y quien maneja el carrito es Juno, la burbuja flotante (tienda/juno.js): cuando ella devuelve una vitrina nueva o un
// carrito al día, esta página lo dibuja. Los botones de la página («Añadir al carrito», «Más info», +, −...) no cambian nada por su
// cuenta: le piden a Juno que lo haga.
// La página es pública: se ve igual con o sin sesión (la sesión solo cambia el encabezado y el menú).

import { datosDeSesion, nuevaConversacion } from './api.js';
import {
  borrarTema,
  borrarToken,
  dejarDestino,
  dejarMensajePendiente,
  dejarNuevaConversacion,
  guardarToken,
  leerToken,
  leerTokenTienda,
} from './almacen.js';
import { iniciarCarrito } from './componentes/carrito.js';
import { marcarEnCarrito, mostrarCatalogo } from './componentes/catalogo.js';
import { crear } from './dom.js';
import { iniciarMenu, mostrarCliente, mostrarVisitante } from './menu.js';
import { abrirIngreso } from './publico/ingreso.js';
import { iniciarJuno } from './tienda/juno.js';

const grilla = document.getElementById('catalogo-grid');
const estado = document.getElementById('catalogo-estado');
const aviso = document.getElementById('catalogo-aviso');
const reintentar = document.getElementById('catalogo-reintentar');

const TOPES = [100, 300, 500, 1000, 2000, 5000];

// ---------- Juno y el carrito ----------

const EN_CELULAR = '(max-width: 820px)';

// Los productos que esta página ya vio (por código): el carrito los usa para dibujar la foto de cada línea.
const productosConocidos = new Map();
let vitrinaActual = null;

// «Añadir al carrito»: no añade nada por su cuenta, se lo pide a Juno. En computador se abre su ventana y se ve cómo lo hace;
// en celular la ventana taparía la tienda, así que Juno lo hace sin abrirse y la página avisa con un mensaje breve.
function pedirAnadir(producto) {
  const orden = `Añadir al carrito: ${producto.nombre}`;
  if (window.matchMedia(EN_CELULAR).matches) juno.ordenar(orden);
  else juno.pedir(orden);
}

// «Preguntarle a Juno» (en la ficha): abre la ventana y le pregunta por ese producto.
function preguntarPor(producto) {
  juno.pedir(`Tengo una duda sobre: ${producto.nombre}`);
}

const acciones = {
  anadir: pedirAnadir,
  preguntar: preguntarPor,
  verCarrito: () => carrito.abrir(),
  cantidadEnCarrito: (id) => carrito.cantidades().get(id) || 0,
};

// Un aviso breve (solo cuando Juno cambió el carrito con su ventana cerrada y no hay nada más abierto).
let avisoActual = null;
function avisar(texto) {
  avisoActual?.remove();
  const aviso = crear('div', 'cr-toast');
  aviso.setAttribute('role', 'status');
  const foto = crear('img');
  foto.src = '/static/img/juno.png';
  foto.alt = '';
  const ver = crear('button', '', 'Ver carrito');
  ver.type = 'button';
  ver.addEventListener('click', () => {
    aviso.remove();
    carrito.abrir();
  });
  aviso.append(foto, crear('span', '', texto), ver);
  document.body.append(aviso);
  avisoActual = aviso;
  setTimeout(() => aviso.remove(), 5000);
}

const juno = iniciarJuno({
  alCatalogo: (vitrina) => dibujar(vitrina),
  alIdentificar: () => mostrarSesion(),
  alCarrito: (estado, texto) => {
    carrito.actualizar(estado, texto);
    if (texto && !juno.estaAbierta() && !carrito.hayAlgoAbierto()) avisar(texto);
  },
  alPedido: (pedido) => {
    carrito.pedidoApartado(pedido);
    descontarExistencias(pedido);
  },
  alVerCarrito: () => carrito.abrir(),
});

const carrito = iniciarCarrito({
  ordenar: (texto) => juno.ordenar(texto),
  productoDe: (id) => productosConocidos.get(id),
  productoPorNombre: (nombre) => [...productosConocidos.values()].find((p) => p.nombre === nombre),
  alHablarConJuno: () => juno.abrir(),
  alCambiar: (cantidades) => marcarEnCarrito(cantidades),
});

// Tras apartar un pedido, la vitrina muestra ya las unidades que quedan (sin esperar a que se recargue la página).
function descontarExistencias(pedido) {
  for (const linea of pedido.productos) {
    const producto = [...productosConocidos.values()].find((p) => p.nombre === linea.nombre);
    if (producto) producto.unidades = Math.max(0, producto.unidades - linea.cantidad);
  }
  if (vitrinaActual) mostrarCatalogo(vitrinaActual, acciones);
}

// ---------- La vitrina ----------

function mostrarAviso(titulo, texto, conReintento) {
  grilla.replaceChildren();
  estado.hidden = true;
  document.getElementById('catalogo-aviso-titulo').textContent = titulo;
  document.getElementById('catalogo-aviso-texto').textContent = texto;
  reintentar.hidden = !conReintento;
  aviso.hidden = false;
}

function dibujar(vitrina) {
  aviso.hidden = true;
  estado.hidden = false;
  vitrinaActual = vitrina;
  for (const producto of vitrina.productos) productosConocidos.set(producto.id, producto);
  mostrarCatalogo(vitrina, acciones);
}

// Las sugerencias de Juno según lo que hay en la tienda: dos categorías y un tope de precio que separe productos.
function sugerenciasDe(productos) {
  const categorias = [...new Set(productos.map((p) => p.categoria))].slice(0, 2).map((c) => `Solo ${c}`);
  const precios = productos.map((p) => parseFloat(p.precio.replace('S/', '').replace(/,/g, '')));
  const tope = TOPES.find((t) => precios.some((p) => p <= t) && precios.some((p) => p > t));
  return tope ? [...categorias, `Menos de S/ ${tope}`] : categorias;
}

async function cargar() {
  aviso.hidden = true;
  estado.hidden = false;
  document.getElementById('catalogo-estado-texto').textContent = 'Cargando el catálogo…';
  grilla.replaceChildren();
  try {
    const respuesta = await fetch('/publico/catalogo');
    if (!respuesta.ok) throw new Error('Error ' + respuesta.status);
    const vitrina = await respuesta.json();
    if (vitrina.productos.length === 0) {
      mostrarAviso('Aún no hay productos', 'Estamos preparando el catálogo. Vuelve pronto o escríbele a Juno en el chat.', false);
      return;
    }
    dibujar(vitrina);
    juno.ponerSugerencias(sugerenciasDe(vitrina.productos));
    juno.recuperar(); // si Juno tenía un carrito guardado, aparece en el botón «Carrito»
  } catch {
    mostrarAviso('No pude cargar el catálogo', 'Revisa tu conexión e inténtalo de nuevo.', true);
  }
}

reintentar.addEventListener('click', cargar);

// ---------- Sesión, encabezado y menú ----------

// El encabezado y el menú según haya o no un cliente reconocido. Si la persona se registró o se identificó DENTRO de la ventana
// de Juno, su chat principal todavía no la conoce: se le abre uno nuevo (vacío) ya vinculado a ella, sin tocar nada de lo conversado.
async function mostrarSesion() {
  const principal = leerToken();
  let sesion = principal ? await datosDeSesion(principal) : { reconocido: false, nombre: null };
  if (!sesion.reconocido) {
    const deLaTienda = leerTokenTienda();
    const otra = deLaTienda ? await datosDeSesion(deLaTienda) : { reconocido: false, nombre: null };
    if (otra.reconocido) {
      try {
        guardarToken(await nuevaConversacion(deLaTienda));
      } catch {
        /* sin conexión: el chat principal la reconocerá cuando vuelva a identificarse */
      }
      sesion = otra;
    }
  }
  if (sesion.reconocido) mostrarCliente(sesion.nombre);
  else mostrarVisitante();
}

// Pide algo a Juno desde el menú: deja el mensaje y abre el chat, donde Juno lo atiende.
function pedirAJunoEnElChat(mensaje) {
  dejarMensajePendiente(mensaje);
  location.assign('/chat');
}

iniciarMenu({
  alElegir: pedirAJunoEnElChat,
  alNueva: () => {
    dejarNuevaConversacion();
    location.assign('/chat');
  },
  alIngresar: () => {
    dejarDestino('/catalogo'); // la ventana de ingreso termina en el lienzo; desde ahí se vuelve a la tienda
    abrirIngreso({ lado: 'cliente' });
  },
  alSalir: () => {
    borrarToken();
    borrarTema();
    location.assign('/');
  },
});

// En el menú, esta página queda marcada.
const enlace = document.getElementById('menu-catalogo');
if (enlace) enlace.setAttribute('aria-current', 'page');

mostrarVisitante(); // por defecto; se corrige enseguida si hay un cliente reconocido
mostrarSesion();
cargar();
