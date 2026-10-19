// tarjeta_datos.js: la credencial (carnet) con los datos encontrados de una persona o empresa.
// Estilos en static/css/carnet.css. La foto NO se usa: se dibuja un círculo con las iniciales
// (no guardamos ni mostramos fotografías de documentos).

import { crear } from '../dom.js';
import { agregar } from '../burbujas.js';
import { anchosCode128 } from '../codigo128.js';

const NS_SVG = 'http://www.w3.org/2000/svg';

// Crea un elemento SVG con sus atributos. Ejemplo: svg('rect', { x: 0, y: 0 })
function svg(etiqueta, atributos = {}) {
  const elemento = document.createElementNS(NS_SVG, etiqueta);
  Object.entries(atributos).forEach(([nombre, valor]) => elemento.setAttribute(nombre, valor));
  return elemento;
}

// Formas geométricas translúcidas de la banda azul (como el carnet de referencia).
function crearFormas() {
  const dibujo = svg('svg', { class: 'carnet-formas', viewBox: '0 0 340 92', preserveAspectRatio: 'none', 'aria-hidden': 'true' });
  [
    { points: '0,0 150,0 105,92 0,92', opacity: 0.1 },
    { points: '95,0 190,0 145,92 50,92', opacity: 0.08 },
    { points: '170,0 215,0 170,92 125,92', opacity: 0.12 },
    { points: '250,0 340,0 340,40 215,92', opacity: 0.07 },
  ].forEach((forma) => dibujo.appendChild(svg('polygon', { points: forma.points, fill: '#ffffff', 'fill-opacity': forma.opacity })));
  return dibujo;
}

// Código de barras REAL (Code 128) que contiene exactamente el código de cliente, nada más.
// Lo puede leer cualquier lector o app de celular. Devuelve null si el texto no se puede representar.
function crearCodigoDeBarras(codigo) {
  const anchos = anchosCode128(codigo);
  if (!anchos) return null;

  const SILENCIO = 10; // espacio en blanco obligatorio a cada lado para que el lector detecte el inicio y el fin
  const total = anchos.reduce((suma, ancho) => suma + ancho, 0) + SILENCIO * 2;
  const dibujo = svg('svg', {
    class: 'carnet-barras',
    viewBox: `0 0 ${total} 40`,
    preserveAspectRatio: 'none',
    'shape-rendering': 'crispEdges',
    role: 'img',
    'aria-label': 'Código de barras del cliente ' + codigo,
  });
  dibujo.appendChild(svg('rect', { x: 0, y: 0, width: total, height: 40, fill: '#ffffff' })); // fondo blanco siempre
  let x = SILENCIO;
  anchos.forEach((ancho, indice) => {
    // Los anchos alternan: barra (negra), espacio (blanco), barra, espacio...
    if (indice % 2 === 0) dibujo.appendChild(svg('rect', { x, y: 0, width: ancho, height: 40, fill: '#000000' }));
    x += ancho;
  });
  return dibujo;
}

// Icono de verificación (círculo con símbolo). El texto solo se ve al pasar el mouse. Colores en carnet.css.
const VERIFICACIONES = {
  verificado: { texto: 'Verificado', simbolo: [['path', { d: 'M5.5 10.5l3 3 6-6.5' }]] },
  pendiente: { texto: 'Sin verificar', simbolo: [['path', { d: 'M10 5.5V10l3 2' }]] },
  fallido: { texto: 'No verificado', simbolo: [['path', { d: 'M6.5 6.5l7 7M13.5 6.5l-7 7' }]] },
};

function crearVerificacion(estado, titulo) {
  const config = VERIFICACIONES[estado];
  if (!config) return null;
  const marca = crear('span', 'verif ' + estado);
  const dibujo = svg('svg', { viewBox: '0 0 20 20', 'aria-hidden': 'true' });
  dibujo.appendChild(svg('circle', { cx: 10, cy: 10, r: 9, class: 'verif-fondo' }));
  config.simbolo.forEach(([etiqueta, atributos]) => dibujo.appendChild(svg(etiqueta, { ...atributos, class: 'verif-trazo' })));
  marca.appendChild(dibujo);
  marca.title = titulo || config.texto; // al pasar el mouse sobre el icono se lee el estado
  marca.setAttribute('role', 'img');
  marca.setAttribute('aria-label', titulo || config.texto); // lectores de pantalla
  return marca;
}

// Iniciales para el círculo: persona -> nombre + primer apellido ("Juan Carlos Perez Gomez" -> "JC");
// empresa -> primeras letras de las dos primeras palabras.
function iniciales(nombre) {
  const palabras = nombre.split(/\s+/).filter(Boolean);
  const letras = palabras.slice(0, 2).map((palabra) => palabra[0]);
  return letras.join('').toUpperCase();
}

// "datos" es { variante, nombre, subtitulo, campos: [{ etiqueta, valor, verificacion? }], estado, codigo? } (lo envía el servidor).
// Sin "codigo" es la tarjeta de confirmación; con "codigo" es la credencial final del cliente registrado.
export function mostrarTarjetaDatos(datos) {
  const carnet = crear('div', 'carnet ' + datos.variante + (datos.codigo ? ' registrado' : ''));

  // --- Banda azul con formas y logo ---
  const banda = crear('div', 'carnet-banda');
  const logo = crear('div', 'carnet-logo');
  logo.append(crear('strong', '', 'H&Devs'), crear('span', '', 'Servicio técnico'));
  banda.append(crearFormas(), logo);

  // --- Iniciales + nombre ---
  const identidad = crear('div', 'carnet-identidad');
  identidad.appendChild(crear('div', 'carnet-foto', iniciales(datos.nombre)));
  const nombre = crear('div', 'carnet-nombre');
  nombre.append(crear('h3', '', datos.nombre), crear('p', '', datos.subtitulo));
  identidad.appendChild(nombre);

  // --- Datos en cuadrícula (aparecen uno tras otro) ---
  const lista = crear('dl', 'carnet-datos');
  const filas = datos.campos.map((campo) => {
    // Los datos largos y el correo ocupan la fila completa.
    const fila = crear('div', 'carnet-dato' + (campo.valor.length > 22 || campo.etiqueta === 'Correo' ? ' ancho' : ''));
    const valor = crear('dd', '', campo.valor);
    const verificacion = campo.verificacion ? crearVerificacion(campo.verificacion) : null;
    if (verificacion) {
      valor.classList.add('con-verificacion');
      valor.appendChild(verificacion);
    }
    fila.append(crear('dt', '', campo.etiqueta), valor);
    return { campo, fila };
  });

  // El estado es un dato más de la cuadrícula: con código (cliente registrado) es "Activo" con check verde;
  // sin código (tarjeta de confirmación) es "Pendiente de confirmar" con reloj amarillo y ocupa la fila completa.
  const registrado = Boolean(datos.codigo);
  const filaEstado = crear('div', 'carnet-dato carnet-estado ' + (registrado ? 'activo' : 'pendiente ancho'));
  const valorEstado = crear('dd', 'con-verificacion', datos.estado);
  valorEstado.appendChild(crearVerificacion(registrado ? 'verificado' : 'pendiente', datos.estado));
  filaEstado.append(crear('dt', '', 'Estado'), valorEstado);

  // Se coloca justo a la derecha del celular (si hay celular); si no, al final.
  const posicionCelular = filas.findIndex(({ campo }) => campo.etiqueta === 'Celular');
  const orden = filas.map(({ fila }) => fila);
  orden.splice(posicionCelular >= 0 ? posicionCelular + 1 : orden.length, 0, filaEstado);
  orden.forEach((fila, indice) => {
    fila.style.animationDelay = 250 + indice * 80 + 'ms';
    lista.appendChild(fila);
  });

  carnet.append(banda, identidad, lista);

  // --- Credencial final: código de cliente + código de barras ---
  if (datos.codigo) {
    const codigo = crear('div', 'carnet-codigo');
    codigo.appendChild(crear('small', '', 'Código de cliente'));
    const barras = crearCodigoDeBarras(datos.codigo);
    if (barras) codigo.appendChild(barras);
    codigo.appendChild(crear('strong', '', datos.codigo));
    carnet.appendChild(codigo);
  }

  agregar(carnet);
}
