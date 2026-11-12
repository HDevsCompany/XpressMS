// inicio.js: arma la página principal (/). Pide los datos públicos (/publico/inicio), dibuja las tarjetas,
// conecta los botones (Iniciar sesión, Juno) y revisa si este navegador ya tiene un cliente reconocido.
// Los textos que vienen de la base se escriben con textContent (nunca como HTML), por seguridad.

import { borrarTema, borrarToken, leerToken } from '../almacen.js';
import { sesionReconocida } from '../api.js';
import { empezarGuia } from './guia.js';
import { abrirIngreso } from './ingreso.js';
import { abrirJuno, iniciarJuno } from './juno.js';

// ---------- Íconos (constantes del sistema, no vienen de la base) ----------
const ICONOS = {
  laptop: '<svg viewBox="0 0 24 24"><rect x="4" y="5" width="16" height="11" rx="1.5"/><path d="M2 19h20"/></svg>',
  chip: '<svg viewBox="0 0 24 24"><rect x="6" y="6" width="12" height="12" rx="2"/><path d="M9 2v4M15 2v4M9 18v4M15 18v4M2 9h4M2 15h4M18 9h4M18 15h4"/></svg>',
  periferico: '<svg viewBox="0 0 24 24"><rect x="6" y="3" width="12" height="18" rx="6"/><path d="M12 7v4"/></svg>',
  red: '<svg viewBox="0 0 24 24"><path d="M5 12a10 10 0 0 1 14 0M8.5 15.5a5 5 0 0 1 7 0"/><circle cx="12" cy="19" r="1"/></svg>',
  cable: '<svg viewBox="0 0 24 24"><path d="M9 3v5M15 3v5M7 8h10v4a5 5 0 0 1-10 0zM12 17v4"/></svg>',
  monitor: '<svg viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4"/></svg>',
  disco: '<svg viewBox="0 0 24 24"><rect x="4" y="3" width="16" height="18" rx="2"/><circle cx="12" cy="14" r="3"/><path d="M8 7h4"/></svg>',
  caja: '<svg viewBox="0 0 24 24"><path d="M21 8 12 3 3 8l9 5zM3 8v8l9 5 9-5V8M12 13v8"/></svg>',
  llave: '<svg viewBox="0 0 24 24"><path d="M14.7 6.3a4 4 0 0 0-5.4 5.4L3 18l3 3 6.3-6.3a4 4 0 0 0 5.4-5.4l-2.5 2.5-2.5-.5-.5-2.5z"/></svg>',
};

// Escoge un ícono según el nombre (sin tildes ni mayúsculas). Lo que no se reconoce usa una caja.
const PALABRAS = [
  ['laptop', ['laptop', 'portatil', 'notebook']],
  ['monitor', ['monitor', 'pantalla']],
  ['disco', ['disco', 'ssd', 'almacenamiento', 'memoria usb']],
  ['chip', ['componente', 'procesador', 'memoria', 'tarjeta', 'placa']],
  ['periferico', ['periferic', 'mouse', 'teclado', 'raton']],
  ['red', ['red', 'router', 'wifi', 'conectividad']],
  ['cable', ['cable', 'accesorio', 'adaptador']],
];

function iconoPara(nombre) {
  const limpio = nombre.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();
  for (const [icono, claves] of PALABRAS) {
    if (claves.some((clave) => limpio.includes(clave))) return icono;
  }
  return 'caja';
}

// ---------- Utilidades ----------
function crear(etiqueta, clase, texto) {
  const elemento = document.createElement(etiqueta);
  if (clase) elemento.className = clase;
  if (texto !== undefined) elemento.textContent = texto;
  return elemento;
}

function icono(nombre) {
  const caja = crear('span', 'pub-icono');
  caja.innerHTML = ICONOS[nombre]; // constante del sistema
  return caja;
}

// "1299.00" -> "S/ 1,299.00"
function soles(texto) {
  const numero = Number(texto);
  if (!Number.isFinite(numero)) return '';
  return 'S/ ' + numero.toLocaleString('es-PE', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function botonJuno(texto) {
  const boton = crear('button', 'pub-boton-contorno', texto);
  boton.type = 'button';
  boton.addEventListener('click', abrirJuno);
  return boton;
}

// ---------- Dibujo de las secciones ----------
function dibujarCategorias(categorias) {
  const lista = document.getElementById('lista-categorias');
  lista.replaceChildren();
  categorias.forEach((categoria) => {
    const tarjeta = crear('div', 'pub-categoria');
    tarjeta.append(icono(iconoPara(categoria.nombre)), crear('span', '', categoria.nombre));
    lista.append(tarjeta);
  });
  document.getElementById('seccion-categorias').hidden = categorias.length === 0;
}

function dibujarDestacados(productos) {
  const lista = document.getElementById('lista-destacados');
  lista.replaceChildren();
  productos.forEach((producto) => {
    const tarjeta = crear('article', 'pub-producto');
    const imagen = crear('div', 'pub-imagen');
    imagen.innerHTML = ICONOS[iconoPara(producto.categoria_nombre + ' ' + producto.nombre)] || ICONOS.caja;

    const cuerpo = crear('div', 'pub-producto-cuerpo');
    cuerpo.append(
      crear('b', '', producto.nombre),
      crear('span', 'pub-precio', soles(producto.precio)),
      crear('small', '', producto.stock <= 3 ? `Quedan ${producto.stock}` : 'Disponible'),
      botonJuno('Preguntar a Juno'),
    );
    tarjeta.append(imagen, cuerpo);
    lista.append(tarjeta);
  });
  document.getElementById('seccion-destacados').hidden = productos.length === 0;
}

function dibujarServicios(servicios) {
  const lista = document.getElementById('lista-servicios');
  lista.replaceChildren();
  servicios.forEach((servicio) => {
    const tarjeta = crear('article', 'pub-servicio');
    const datos = crear('div', 'pub-servicio-datos');
    if (servicio.tiempo_estimado) datos.append(crear('span', '', 'Duración aprox.: ' + servicio.tiempo_estimado));
    const precio = servicio.precio_base ? 'Desde ' + soles(servicio.precio_base) : 'A cotizar';
    datos.append(crear('strong', '', precio));
    tarjeta.append(icono('llave'), crear('b', '', servicio.nombre), crear('p', '', servicio.descripcion), datos, botonJuno('Consultar con Juno'));
    lista.append(tarjeta);
  });
  document.getElementById('servicios').hidden = servicios.length === 0;
}

async function cargarDatos() {
  try {
    const respuesta = await fetch('/publico/inicio');
    if (!respuesta.ok) throw new Error('Error ' + respuesta.status);
    const datos = await respuesta.json();
    dibujarCategorias(datos.categorias);
    dibujarDestacados(datos.destacados);
    dibujarServicios(datos.servicios);
    const hayAlgo = datos.categorias.length || datos.destacados.length || datos.servicios.length;
    document.getElementById('aviso-vacio').hidden = Boolean(hayAlgo);
  } catch {
    // Sin datos la página igual sirve: Juno responde y se puede iniciar sesión.
    document.getElementById('aviso-vacio').hidden = false;
  }
}

// ---------- Sesión: ¿este navegador ya tiene un cliente reconocido? ----------
async function revisarSesion() {
  const token = leerToken();
  if (!token || !(await sesionReconocida(token))) return;
  document.getElementById('boton-ingresar').hidden = true;
  document.getElementById('sesion-activa').hidden = false;
}

document.getElementById('boton-salir').addEventListener('click', () => {
  // Olvida al cliente en este navegador (sus datos no se borran de la base de datos).
  borrarToken();
  borrarTema();
  location.reload();
});

document.getElementById('boton-ingresar').addEventListener('click', () => abrirIngreso({ lado: 'cliente' }));

// Enlaces de la barra: el que corresponde a la sección queda marcado.
const enlaces = document.querySelectorAll('.pub-enlaces a');
enlaces.forEach((enlace) => {
  enlace.addEventListener('click', () => {
    enlaces.forEach((otro) => otro.classList.toggle('activo', otro === enlace));
  });
});

// ---------- Juno y su guía ----------
const juno = { abrir: abrirJuno };
iniciarJuno({
  alPedirGuia: (cual) => {
    // Juno se guarda y la guía señala dónde pulsar. Si se cancela, Juno vuelve (salvo que la persona ya esté ingresando).
    document.getElementById('juno-cerrar').click();
    empezarGuia(cual, {
      alTerminar: (motivo) => {
        if (motivo === 'cancelada' && !document.querySelector('.pub-velo')) juno.abrir();
      },
    });
  },
});

cargarDatos();
revisarSesion();
