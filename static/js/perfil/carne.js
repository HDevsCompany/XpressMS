// carne.js: el «Pase XpressMS» de «Mi perfil y datos»: una tarjeta horizontal (formato ID-1, como un DNI) con un panel de
// color a la izquierda (foto, «Pase XpressMS», estado) y los datos a la derecha. Al tocarla gira (medio segundo) y
// muestra el reverso: una tarjeta blanca con el código de barras para que el técnico lo escanee.
//   - Portada: 6 estilos a elegir. Foto: se elige en este dispositivo. Ambas se guardan solo en este navegador
//     (no viajan al servidor), así que no cambian nada de la cuenta.
//   - «Mostrar en tienda»: pantalla con las dos caras grandes (anverso y reverso).
//   - «Descargar»: dos imágenes PNG de 1012 × 638 px (300 ppp; una por cara, dibujadas aquí mismo) o un PDF en hoja A4
//     con las dos caras a tamaño real (85,60 × 53,98 mm).
// Todo texto se pone con textContent (nunca innerHTML). Las medidas del pase están en "cqw" (1 % de su propio ancho)
// para que se vea igual de proporcionado en una pantalla grande que en un celular; las imágenes usan las mismas medidas.

import { anchosCode128 } from '../codigo128.js';
import { crear } from '../dom.js';

const ESPACIOS = 'http://www.w3.org/2000/svg';

const LEMA = 'Hardware & Development';
const PIE_DEL_DORSO = 'Pase de uso personal · xpressms · H&Devs';
const CLAVE_PORTADA = 'xpressms_carne_portada';
const CLAVE_FOTO = 'xpressms_carne_foto';
const CLAVE_PORTADA_IMG = 'xpressms_carne_portada_img'; // la imagen de «Mi portada» (la que sube el cliente)

// Los estilos de portada: nombre y fondo de la muestra (los colores reales del pase están en perfil.css).
export const PORTADAS = [
  { id: 'verde', nombre: 'Verde H&Devs', muestra: 'linear-gradient(135deg, #14532d, #16a34a 55%, #4ade80)' },
  { id: 'azul', nombre: 'Azul clásico', muestra: 'linear-gradient(135deg, #1e3a8a, #2563eb 55%, #60a5fa)' },
  { id: 'noche', nombre: 'Noche', muestra: 'linear-gradient(135deg, #020617, #1e293b 55%, #475569)' },
  { id: 'atardecer', nombre: 'Atardecer', muestra: 'linear-gradient(135deg, #f59e0b, #f97316 40%, #db2777 75%, #7c3aed)' },
  { id: 'violeta', nombre: 'Violeta', muestra: 'linear-gradient(135deg, #4c1d95, #7c3aed 55%, #a78bfa)' },
  {
    id: 'circuito',
    nombre: 'Circuito',
    muestra:
      'radial-gradient(circle, rgba(255,255,255,0.5) 1px, transparent 1.4px) 0 0 / 7px 7px, linear-gradient(135deg, #082f49, #0e7490 55%, #22d3ee)',
  },
];

// La portada que sube el cliente se llama «propia». Su imagen se guarda solo en este navegador.
export const PORTADA_PROPIA = 'propia';
export const imagenDePortadaPropia = () => leer(CLAVE_PORTADA_IMG);

// Pone la portada en un elemento (el pase, una copia o la vista previa): su id y, si es «propia», su imagen.
export function aplicarPortada(elemento, id, imagen = leer(CLAVE_PORTADA_IMG)) {
  elemento.dataset.portada = id;
  if (id === PORTADA_PROPIA && imagen) elemento.style.setProperty('--imagen-propia', `url("${imagen}")`);
  else elemento.style.removeProperty('--imagen-propia');
}

// Medidas reales de una tarjeta ID-1.
export const ANCHO_MM = 85.6;
export const ALTO_MM = 53.98;

function leer(clave) {
  try {
    return localStorage.getItem(clave);
  } catch {
    return null;
  }
}

function escribir(clave, valor) {
  try {
    if (valor === null) localStorage.removeItem(clave);
    else localStorage.setItem(clave, valor);
    return true;
  } catch {
    return false; // sin almacenamiento (o lleno): la elección dura solo mientras la página esté abierta
  }
}

function svg(etiqueta, atributos = {}) {
  const elemento = document.createElementNS(ESPACIOS, etiqueta);
  Object.entries(atributos).forEach(([nombre, valor]) => elemento.setAttribute(nombre, valor));
  return elemento;
}

function icono(...trazos) {
  const dibujo = svg('svg', { viewBox: '0 0 24 24', 'aria-hidden': 'true' });
  trazos.forEach((d) => dibujo.appendChild(svg('path', { d })));
  return dibujo;
}

// Las iniciales del círculo: las de las dos primeras palabras del nombre ("Pedro Alonso Ramos Quispe" -> "PA").
export function inicialesDeCarne(nombre) {
  const palabras = (nombre || '').split(/\s+/).filter(Boolean);
  if (palabras.length === 0) return '?';
  return palabras
    .slice(0, 2)
    .map((p) => p.charAt(0))
    .join('')
    .toUpperCase();
}

// Código de barras REAL (Code 128) con exactamente el código del cliente. Devuelve null si no se puede representar.
export function dibujarBarras(codigo, clase = 'pf-barras') {
  const anchos = anchosCode128(codigo);
  if (!anchos) return null;
  const SILENCIO = 10; // espacio en blanco obligatorio a cada lado para que el lector detecte el inicio y el fin
  const total = anchos.reduce((suma, ancho) => suma + ancho, 0) + SILENCIO * 2;
  const dibujo = svg('svg', {
    class: clase,
    viewBox: `0 0 ${total} 40`,
    preserveAspectRatio: 'none',
    'shape-rendering': 'crispEdges',
    role: 'img',
    'aria-label': 'Código de barras del cliente ' + codigo,
  });
  dibujo.appendChild(svg('rect', { x: 0, y: 0, width: total, height: 40, fill: '#ffffff' }));
  let x = SILENCIO;
  anchos.forEach((ancho, indice) => {
    if (indice % 2 === 0) dibujo.appendChild(svg('rect', { x, y: 0, width: ancho, height: 40, fill: '#000000' }));
    x += ancho;
  });
  return dibujo;
}

// El tipo de documento en corto, para que quepa en su casilla («Carnet de extranjería» pasa a «C. extranjería»).
function tipoDeDocumentoCorto(perfil) {
  const tipo = perfil.tipo_documento || (perfil.documento_completo || '').split(' ')[0] || '';
  return tipo === 'Carnet de extranjería' ? 'C. extranjería' : tipo;
}

// Los datos que van en el frente, en el orden en que se ven. El tipo de documento y su número van en casillas
// separadas, porque no todos tienen DNI (también hay RUC, carnet de extranjería, pasaporte).
function datosDelFrente(perfil) {
  const partes = (perfil.documento_completo || '').split(' ');
  const numero = perfil.numero_documento || partes.slice(1).join(' ');
  const datos = [
    { etiqueta: 'Tipo de documento', valor: tipoDeDocumentoCorto(perfil) },
    { etiqueta: 'N.º de documento', valor: numero },
  ];
  if (perfil.sexo) datos.push({ etiqueta: 'Sexo', valor: perfil.sexo });
  if (perfil.nacimiento_enmascarado) datos.push({ etiqueta: 'Nacimiento', valor: perfil.nacimiento_enmascarado });
  datos.push({ etiqueta: 'Celular', valor: perfil.celular, verificado: perfil.celular_verificado });
  // Un correo largo ocupa toda la fila para leerse entero; la dirección siempre la ocupa.
  if (perfil.correo) datos.push({ etiqueta: 'Correo', valor: perfil.correo, ancho: perfil.correo.length > 26 });
  datos.push({ etiqueta: 'Dirección', valor: perfil.direccion_enmascarada, ancho: true });
  return datos;
}

// Reparte los datos en una cuadrícula de dos columnas (los «anchos» ocupan la fila entera): da la fila y la columna de
// cada uno y cuántas filas hay. La pantalla (CSS grid) y la imagen PNG usan este mismo reparto.
function repartirDatos(datos) {
  let fila = 0;
  let columna = 0;
  const celdas = datos.map((dato) => {
    if (dato.ancho) {
      if (columna) fila += 1;
      columna = 0;
      const celda = { ...dato, fila, columna: 0 };
      fila += 1;
      return celda;
    }
    const celda = { ...dato, fila, columna };
    columna += 1;
    if (columna === 2) {
      columna = 0;
      fila += 1;
    }
    return celda;
  });
  return { celdas, filas: fila + (columna ? 1 : 0) };
}

// Tamaño (en cqw) del nombre: 5 si es corto; baja hasta 3 cuando es muy largo, para que entre en dos renglones.
function tamanoDelNombre(nombre) {
  return 5 * Math.min(1, Math.max(0.6, 30 / Math.max(1, nombre.length)));
}

// ---------- Las dos caras (para el pase, la pantalla de tienda y la hoja del PDF) ----------

// Construye el frente y el reverso. "foto" es la dirección (data:) de la foto guardada, o null para mostrar las iniciales.
// Devuelve las dos caras ya armadas; cada una se mete en un contenedor con la clase pf-copia (o pf-carne) para medirse en cqw.
export function construirCaras(perfil, foto = null) {
  const frente = crear('div', 'pf-cara pf-frente');
  const dorso = crear('div', 'pf-cara pf-dorso');

  // ----- Frente: panel de color a la izquierda -----
  const lateral = crear('div', 'pf-lateral');

  const editar = crear('button', 'pf-mini pf-mini-editar');
  editar.type = 'button';
  editar.setAttribute('aria-label', 'Personalizar portada');
  editar.append(icono('M4 20l4-1 10-10-3-3L5 16z', 'M13.5 7.5l3 3'));

  const voltear = crear('button', 'pf-mini pf-mini-girar');
  voltear.type = 'button';
  voltear.setAttribute('aria-label', 'Ver el reverso del pase');
  voltear.append(icono('M4 12a8 8 0 0113.7-5.7L20 8.5', 'M20 4v4.5h-4.5', 'M20 12a8 8 0 01-13.7 5.7L4 15.5', 'M4 20v-4.5h4.5'));

  const fotoEl = crear('div', 'pf-foto');
  const iniciales = crear('span', '', inicialesDeCarne(perfil.nombre));
  const imagenFoto = document.createElement('img');
  imagenFoto.alt = '';
  fotoEl.append(iniciales, imagenFoto);
  const ponerFoto = (direccion) => {
    if (direccion) {
      imagenFoto.src = direccion;
      imagenFoto.hidden = false;
      iniciales.hidden = true;
    } else {
      imagenFoto.removeAttribute('src');
      imagenFoto.hidden = true;
      iniciales.hidden = false;
    }
  };
  ponerFoto(foto);

  const rotulo = crear('p', 'pf-rotulo');
  rotulo.append(crear('span', '', 'Pase'), crear('span', '', 'XpressMS'));
  const estado = crear('span', 'pf-estado');
  estado.append(crear('i'), document.createTextNode(perfil.estado));
  lateral.append(editar, voltear, fotoEl, rotulo, estado);

  // ----- Frente: los datos -----
  const principal = crear('div', 'pf-principal');
  const marcaFila = crear('div', 'pf-marca-fila');
  marcaFila.append(crear('b', 'pf-marca-texto', 'H&Devs'), crear('span', 'pf-servicio', LEMA));

  const flujo = crear('div', 'pf-flujo');
  const nombre = crear('h3', 'pf-carne-nombre', perfil.nombre);
  nombre.style.fontSize = tamanoDelNombre(perfil.nombre).toFixed(2) + 'cqw';
  const lista = crear('dl', 'pf-datos');
  repartirDatos(datosDelFrente(perfil)).celdas.forEach((dato) => {
    const celda = crear('div', dato.ancho ? 'pf-dato pf-dato-ancho' : 'pf-dato');
    const valor = crear('dd');
    valor.append(crear('span', 'pf-valor', dato.valor));
    // Un valor largo se achica un poco para que se lea entero (el ancho de la casilla se calcula con el reparto de arriba).
    const capacidad = (dato.ancho ? 61.2 : dato.columna === 0 ? 28.8 : 30.8) - (dato.verificado ? 3.6 : 0);
    const escala = Math.min(1, capacidad / ((dato.etiqueta === 'Correo' || dato.etiqueta === 'Dirección' ? 0.64 : 0.56) * 2.95 * Math.max(1, dato.valor.length)));
    if (escala < 1) valor.style.fontSize = (2.95 * Math.max(0.65, escala)).toFixed(2) + 'cqw';
    if (dato.verificado) {
      const marca = crear('span', 'pf-verificado');
      marca.title = 'Verificado';
      marca.append(icono('M6 12.5l4 4 8-9'));
      valor.append(marca);
    }
    celda.append(crear('dt', '', dato.etiqueta), valor);
    lista.append(celda);
  });
  flujo.append(nombre, crear('p', 'pf-carne-tipo', perfil.tipo_persona), lista);
  principal.append(marcaFila, flujo);
  frente.append(lateral, principal);

  // ----- Reverso: tarjeta blanca con la banda de color arriba -----
  const banda = crear('div', 'pf-dorso-banda');
  banda.append(crear('b', 'pf-dorso-marca-texto', 'H&Devs'), crear('span', 'pf-dorso-servicio', LEMA));
  const caja = crear('div', 'pf-dorso-barras');
  const barras = dibujarBarras(perfil.codigo);
  if (barras) caja.append(barras);
  else caja.append(crear('b', '', perfil.codigo));
  dorso.append(
    banda,
    crear('p', 'pf-dorso-etiqueta', 'Código'),
    caja,
    crear('p', 'pf-dorso-codigo', perfil.codigo),
    crear('div', 'pf-dorso-linea'),
    crear('p', 'pf-dorso-pie', PIE_DEL_DORSO),
  );

  return { frente, dorso, botonEditar: editar, botonVoltear: voltear, ponerFoto };
}

// Una cara suelta dentro de su propio contenedor (con la portada puesta): para la pantalla de tienda, la hoja del PDF
// y las vistas previas. Sin giro: el reverso se muestra tal cual.
export function copiaDeCara(cara, portada, clase = '', imagenPropia = undefined) {
  const caja = crear('div', 'pf-copia ' + clase);
  aplicarPortada(caja, portada, imagenPropia);
  cara.removeAttribute('aria-hidden');
  caja.append(cara);
  return caja;
}

// ---------- Las imágenes PNG ----------

function rectanguloRedondeado(c, x, y, ancho, alto, radio) {
  c.beginPath();
  c.moveTo(x + radio, y);
  c.arcTo(x + ancho, y, x + ancho, y + alto, radio);
  c.arcTo(x + ancho, y + alto, x, y + alto, radio);
  c.arcTo(x, y + alto, x, y, radio);
  c.arcTo(x, y, x + ancho, y, radio);
  c.closePath();
}

function texto(c, cadena, x, y, { tam, color, peso = 400, alinear = 'left', espacio = 0, ancho = null, familia = 'system-ui, "Segoe UI", Arial, sans-serif' }) {
  c.font = `${peso} ${tam}px ${familia}`;
  c.fillStyle = color;
  c.textAlign = alinear;
  c.textBaseline = 'middle';
  if ('letterSpacing' in c) c.letterSpacing = `${espacio}px`;
  let tamano = tam;
  // Si el texto no cabe en el ancho dado, se achica un poco.
  while (ancho && c.measureText(cadena).width > ancho && tamano > tam * 0.6) {
    tamano -= 1;
    c.font = `${peso} ${tamano}px ${familia}`;
  }
  c.fillText(cadena, x, y);
}

// Las franjas de luz en diagonal (misma idea que perfil.css): paralelogramos blancos casi transparentes.
function franjas(c, u, x, y, alto, lista) {
  lista.forEach(([desde, ancho, opacidad]) => {
    const inclina = alto * 0.2126; // tan(12°)
    c.fillStyle = `rgba(255,255,255,${opacidad})`;
    c.beginPath();
    c.moveTo(x + desde * u + inclina / 2, y);
    c.lineTo(x + (desde + ancho) * u + inclina / 2, y);
    c.lineTo(x + (desde + ancho) * u - inclina / 2, y + alto);
    c.lineTo(x + desde * u - inclina / 2, y + alto);
    c.closePath();
    c.fill();
  });
}

// La trama de puntos de la portada «Circuito».
function puntos(c, u, x, y, ancho, alto) {
  c.fillStyle = 'rgba(255,255,255,0.3)';
  const paso = 3 * u;
  for (let py = y + paso / 2; py < y + alto; py += paso) {
    for (let px = x + paso / 2; px < x + ancho; px += paso) {
      c.beginPath();
      c.arc(px, py, 0.34 * u, 0, Math.PI * 2);
      c.fill();
    }
  }
}

// Dibuja una imagen llenando el rectángulo (recortando lo que sobre), con un velo oscuro para que se lean los textos.
function llenarConImagen(c, imagen, x, y, ancho, alto) {
  const escala = Math.max(ancho / imagen.naturalWidth, alto / imagen.naturalHeight);
  const w = imagen.naturalWidth * escala;
  const h = imagen.naturalHeight * escala;
  c.save();
  c.beginPath();
  c.rect(x, y, ancho, alto); // lo que sobra de la imagen no se dibuja
  c.clip();
  c.drawImage(imagen, x + (ancho - w) / 2, y + (alto - h) / 2, w, h);
  c.fillStyle = 'rgba(15,23,42,0.18)';
  c.fillRect(x, y, ancho, alto);
  c.restore();
}

function cargarImagen(direccion) {
  return new Promise((resolver) => {
    if (!direccion) return resolver(null);
    const imagen = new Image();
    imagen.onload = () => resolver(imagen);
    imagen.onerror = () => resolver(null);
    imagen.src = direccion;
  });
}

// Parte un texto en hasta dos renglones (por palabras) para que entre en el ancho dado.
function partirEnRenglones(c, cadena, tam, ancho) {
  if ('letterSpacing' in c) c.letterSpacing = `${-0.015 * tam}px`; // igual que el nombre dibujado
  c.font = `800 ${tam}px system-ui, "Segoe UI", Arial, sans-serif`;
  if (c.measureText(cadena).width <= ancho) return [cadena];
  const palabras = cadena.split(/\s+/);
  let primero = '';
  let i = 0;
  while (i < palabras.length) {
    const prueba = primero ? primero + ' ' + palabras[i] : palabras[i];
    if (primero && c.measureText(prueba).width > ancho) break;
    primero = prueba;
    i += 1;
  }
  const resto = palabras.slice(i).join(' ');
  return resto ? [primero, resto] : [primero];
}

// Dibuja UNA cara del pase en un lienzo de 1012 × 638 px (85,6 mm a 300 ppp), con las mismas medidas que la pantalla.
function dibujarCara(lado, perfil, colores, foto, datosFrente) {
  const W = 1012;
  const H = 638;
  const u = W / 100; // 1 "cqw"
  const lienzo = document.createElement('canvas');
  lienzo.width = W;
  lienzo.height = H;
  const c = lienzo.getContext('2d');
  rectanguloRedondeado(c, 0, 0, W, H, 4.2 * u);
  c.clip();
  c.fillStyle = '#ffffff';
  c.fillRect(0, 0, W, H);

  if (lado === 'frente') {
    const g = c.createLinearGradient(0, 0, 30 * u * 0.6, H);
    g.addColorStop(0, colores.c1);
    g.addColorStop(0.58, colores.c2);
    g.addColorStop(1, colores.c3);
    c.fillStyle = g;
    c.fillRect(0, 0, 30 * u, H);
    if (colores.imagen) llenarConImagen(c, colores.imagen, 0, 0, 30 * u, H);
    franjas(c, u, 0, 0, H, [[8, 7, 0.08], [18, 4, 0.1]]);
    if (colores.puntos === '1') puntos(c, u, 0, 0, 30 * u, H);

    // Foto o iniciales
    const cx = 15 * u;
    const cy = 23.7 * u;
    const r = 9.85 * u;
    c.save();
    c.beginPath();
    c.arc(cx, cy, r, 0, Math.PI * 2);
    c.clip();
    const gf = c.createLinearGradient(cx - r, cy - r, cx + r, cy + r);
    gf.addColorStop(0, colores.tinte1);
    gf.addColorStop(1, colores.tinte2);
    c.fillStyle = gf;
    c.fillRect(cx - r, cy - r, r * 2, r * 2);
    if (foto) c.drawImage(foto, cx - r, cy - r, r * 2, r * 2);
    c.restore();
    c.lineWidth = 0.7 * u;
    c.strokeStyle = 'rgba(255,255,255,0.85)';
    c.beginPath();
    c.arc(cx, cy, r, 0, Math.PI * 2);
    c.stroke();
    if (!foto) texto(c, inicialesDeCarne(perfil.nombre), cx, cy + 0.4 * u, { tam: 7.2 * u, color: colores.acento, peso: 800, alinear: 'center' });

    texto(c, 'PASE', cx, 37.7 * u, { tam: 1.7 * u, color: 'rgba(255,255,255,0.92)', peso: 700, alinear: 'center', espacio: 3 });
    texto(c, 'XPRESSMS', cx, 40.4 * u, { tam: 1.7 * u, color: 'rgba(255,255,255,0.92)', peso: 700, alinear: 'center', espacio: 3 });
    // Estado
    const anchoPildora = 14.6 * u;
    rectanguloRedondeado(c, cx - anchoPildora / 2, 46.8 * u - 1.9 * u, anchoPildora, 3.8 * u, 1.9 * u);
    c.fillStyle = '#ffffff';
    c.fill();
    c.fillStyle = colores.punto;
    c.beginPath();
    c.arc(cx - anchoPildora / 2 + 2.4 * u, 46.8 * u, 0.55 * u, 0, Math.PI * 2);
    c.fill();
    texto(c, perfil.estado.toUpperCase(), cx - anchoPildora / 2 + 3.5 * u, 46.9 * u, { tam: 1.55 * u, color: colores.acento, peso: 800, espacio: 1.5 });

    // Datos (mismas medidas que .pf-principal y .pf-flujo de perfil.css)
    const x0 = 34.2 * u;
    const derecha = W - 4.6 * u;
    const anchoUtil = derecha - x0;
    texto(c, 'H&Devs', x0, 5.6 * u, { tam: 3.5 * u, color: colores.acento, peso: 800 });
    texto(c, LEMA.toUpperCase(), derecha, 5.9 * u, { tam: 1.5 * u, color: '#64748b', peso: 700, alinear: 'right', espacio: 2.4 });

    // El nombre: hasta dos renglones.
    const tamNombre = tamanoDelNombre(perfil.nombre) * u;
    const altoRenglon = tamNombre * 1.08;
    const renglones = partirEnRenglones(c, perfil.nombre, tamNombre, anchoUtil);
    const arriba = 10.2 * u;
    renglones.forEach((linea, i) => {
      texto(c, linea, x0, arriba + altoRenglon * (i + 0.5), { tam: tamNombre, color: '#0f172a', peso: 800, espacio: -0.015 * tamNombre, ancho: anchoUtil });
    });
    const topeTipo = arriba + altoRenglon * renglones.length + 1.0 * u;
    texto(c, perfil.tipo_persona, x0, topeTipo + 1.5 * u, { tam: 2.4 * u, color: colores.acento, peso: 700 });

    // La cuadrícula de datos: el alto que queda se reparte por igual entre las filas.
    const { celdas, filas } = repartirDatos(datosFrente);
    const topeDatos = topeTipo + 3.0 * u + 1.8 * u;
    const altoFila = (H - 3.6 * u - topeDatos) / filas;
    const columna0 = (anchoUtil - 1.5 * u) * (29 / 60);
    const columna1 = (anchoUtil - 1.5 * u) * (31 / 60);
    celdas.forEach((dato) => {
      const x = dato.columna === 0 ? x0 : x0 + columna0 + 1.5 * u;
      const anchoCelda = dato.ancho ? anchoUtil : dato.columna === 0 ? columna0 : columna1;
      const y = topeDatos + dato.fila * altoFila;
      texto(c, dato.etiqueta.toUpperCase(), x, y + 1.02 * u, { tam: 1.7 * u, color: '#64748b', peso: 700, espacio: 0.14 * 1.7 * u, ancho: anchoCelda });
      const yValor = y + 4.51 * u;
      const reserva = dato.verificado ? 3.8 * u : 0;
      texto(c, dato.valor, x, yValor, { tam: 2.95 * u, color: '#0f172a', peso: 600, ancho: anchoCelda - reserva });
      if (dato.verificado) {
        // El círculo verde con la marca de «verificado», pegado al final del valor.
        const radio = 1.4 * u;
        const fin = x + c.measureText(dato.valor).width + 0.8 * u + radio;
        c.fillStyle = '#22c55e';
        c.beginPath();
        c.arc(fin, yValor, radio, 0, Math.PI * 2);
        c.fill();
        c.strokeStyle = '#ffffff';
        c.lineWidth = 0.34 * u;
        c.lineCap = 'round';
        c.lineJoin = 'round';
        c.beginPath();
        c.moveTo(fin - 0.65 * u, yValor);
        c.lineTo(fin - 0.12 * u, yValor + 0.55 * u);
        c.lineTo(fin + 0.78 * u, yValor - 0.55 * u);
        c.stroke();
      }
    });
    return lienzo;
  }

  // ----- Reverso -----
  const banda = 13 * u;
  const g = c.createLinearGradient(0, 0, W, banda * 1.4);
  g.addColorStop(0, colores.c1);
  g.addColorStop(0.6, colores.c2);
  g.addColorStop(1, colores.c3);
  c.fillStyle = g;
  c.fillRect(0, 0, W, banda);
  if (colores.imagen) llenarConImagen(c, colores.imagen, 0, 0, W, banda);
  franjas(c, u, 0, 0, banda, [[58, 9, 0.08], [72, 5, 0.1], [84, 3, 0.07]]);
  if (colores.puntos === '1') puntos(c, u, 0, 0, W, banda);
  texto(c, 'H&Devs', 5 * u, banda / 2, { tam: 4.2 * u, color: '#ffffff', peso: 800 });
  texto(c, LEMA.toUpperCase(), W - 5 * u, banda / 2, { tam: 1.6 * u, color: 'rgba(255,255,255,0.92)', peso: 700, alinear: 'right', espacio: 2.6 });

  texto(c, 'CÓDIGO', W / 2, 17.8 * u, { tam: 1.5 * u, color: '#64748b', peso: 700, alinear: 'center', espacio: 2.4 });
  const anchos = anchosCode128(perfil.codigo);
  if (anchos) {
    const SILENCIO = 10;
    const total = anchos.reduce((s, a) => s + a, 0) + SILENCIO * 2;
    const ancho = 84 * u;
    const modulo = ancho / total;
    let x = 8 * u + SILENCIO * modulo;
    c.fillStyle = '#000000';
    anchos.forEach((a, i) => {
      if (i % 2 === 0) c.fillRect(x, 21.5 * u, a * modulo, 19 * u);
      x += a * modulo;
    });
  }
  texto(c, perfil.codigo, W / 2, 46.0 * u, { tam: 4.4 * u, color: '#0f172a', peso: 600, alinear: 'center', espacio: 0.42 * 4.4 * u, familia: 'ui-monospace, Consolas, monospace' });
  c.strokeStyle = '#cbd5e1';
  c.lineWidth = 0.16 * u;
  c.setLineDash([0.7 * u, 0.55 * u]);
  c.beginPath();
  c.moveTo(5 * u, 53.4 * u);
  c.lineTo(W - 5 * u, 53.4 * u);
  c.stroke();
  c.setLineDash([]);
  texto(c, PIE_DEL_DORSO, W / 2, 57.2 * u, { tam: 1.4 * u, color: '#94a3b8', peso: 600, alinear: 'center', espacio: 0.08 * 1.4 * u });
  return lienzo;
}

// Una sola imagen con el anverso arriba y el reverso debajo, sobre fondo blanco y con un margen alrededor.
function juntarCaras(anverso, reverso) {
  const MARGEN = 60;
  const SEPARACION = 50;
  const lienzo = document.createElement('canvas');
  lienzo.width = anverso.width + MARGEN * 2;
  lienzo.height = anverso.height + reverso.height + SEPARACION + MARGEN * 2;
  const c = lienzo.getContext('2d');
  c.fillStyle = '#ffffff';
  c.fillRect(0, 0, lienzo.width, lienzo.height);
  c.drawImage(anverso, MARGEN, MARGEN);
  c.drawImage(reverso, MARGEN, MARGEN + anverso.height + SEPARACION);
  return lienzo;
}

function bajarLienzo(lienzo, nombre) {
  return new Promise((resolver) => {
    lienzo.toBlob((blob) => {
      const direccion = URL.createObjectURL(blob);
      const enlace = document.createElement('a');
      enlace.download = nombre;
      enlace.href = direccion;
      document.body.append(enlace);
      enlace.click();
      enlace.remove();
      setTimeout(() => URL.revokeObjectURL(direccion), 4000);
      resolver();
    }, 'image/png');
  });
}

// Crea el pase. "perfil" es lo que devuelve /perfil/datos. "alEditar" se llama al pulsar el lápiz del panel (la página
// abre ahí la ventana de portada). Devuelve el elemento y lo que la página necesita para manejarlo.
export function crearCarne(perfil, { alEditar = () => {}, alDescargar = () => {} } = {}) {
  const raiz = crear('div', 'pf-carne');
  const guardada = leer(CLAVE_PORTADA);
  const valida = PORTADAS.some((p) => p.id === guardada) || (guardada === PORTADA_PROPIA && Boolean(leer(CLAVE_PORTADA_IMG)));
  aplicarPortada(raiz, valida ? guardada : 'verde');
  if (perfil.variante === 'empresa') raiz.dataset.variante = 'empresa';

  const giro = crear('div', 'pf-carne-giro');
  const caras = construirCaras(perfil, leer(CLAVE_FOTO));
  const { frente, dorso } = caras;
  dorso.setAttribute('aria-hidden', 'true');
  caras.botonEditar.addEventListener('click', (evento) => {
    evento.stopPropagation();
    alEditar();
  });
  caras.botonVoltear.addEventListener('click', (evento) => {
    evento.stopPropagation();
    girar();
  });
  giro.append(frente, dorso);
  raiz.append(giro);

  // Tocar el pase lo gira.
  raiz.addEventListener('click', () => girar());

  let girado = false;
  function girar() {
    girado = !girado;
    raiz.classList.toggle('girado', girado);
    frente.setAttribute('aria-hidden', String(girado));
    dorso.setAttribute('aria-hidden', String(!girado));
    return girado;
  }

  const portadaActual = () => raiz.dataset.portada;
  const fotoActual = () => leer(CLAVE_FOTO);

  // Elige la portada. Para «propia» se da también la imagen (data:) que subió el cliente. Devuelve false si no se pudo guardar.
  function elegirPortada(id, imagen = null) {
    if (id === PORTADA_PROPIA) {
      const direccion = imagen || leer(CLAVE_PORTADA_IMG);
      if (!direccion) return false;
      const guardada = imagen ? escribir(CLAVE_PORTADA_IMG, imagen) : true;
      aplicarPortada(raiz, id, direccion);
      return escribir(CLAVE_PORTADA, id) && guardada;
    }
    if (!PORTADAS.some((p) => p.id === id)) return false;
    aplicarPortada(raiz, id);
    return escribir(CLAVE_PORTADA, id);
  }

  // Pone (o quita, con null) la foto del pase. Se guarda solo en este navegador. Devuelve false si no se pudo guardar.
  function cambiarFoto(direccion) {
    caras.ponerFoto(direccion);
    return escribir(CLAVE_FOTO, direccion || null);
  }

  // ----- Mostrar en tienda: las dos caras grandes, una al lado de la otra -----
  function mostrarEnTienda() {
    const velo = crear('div', 'pf-tienda');
    velo.setAttribute('role', 'dialog');
    velo.setAttribute('aria-modal', 'true');
    velo.setAttribute('aria-label', 'Tu pase para mostrar en tienda');

    const cerrar = crear('button', 'pf-tienda-cerrar');
    cerrar.type = 'button';
    cerrar.setAttribute('aria-label', 'Cerrar');
    cerrar.append(icono('M6 6l12 12M18 6L6 18'));

    const fotoAhora = leer(CLAVE_FOTO);
    const grandes = construirCaras(perfil, fotoAhora);
    [grandes.botonEditar, grandes.botonVoltear].forEach((b) => b.remove());
    const portada = portadaActual();
    const zona = crear('div', 'pf-tienda-caras');
    [['Anverso', grandes.frente], ['Reverso', grandes.dorso]].forEach(([nombre, cara]) => {
      const figura = crear('figure', 'pf-tienda-figura');
      figura.append(crear('figcaption', '', nombre), copiaDeCara(cara, portada, 'pf-copia-grande'));
      zona.append(figura);
    });

    const pildoras = crear('div', 'pf-tienda-pildoras');
    const botonTienda = crear('button', 'pf-pildora pf-pildora-azul');
    botonTienda.type = 'button';
    botonTienda.append(icono('M4 9V4h5', 'M20 9V4h-5', 'M4 15v5h5', 'M20 15v5h-5'), document.createTextNode('Mostrar en tienda'));
    const botonBajar = crear('button', 'pf-pildora');
    botonBajar.type = 'button';
    botonBajar.append(icono('M12 4v11', 'M7 11l5 5 5-5', 'M5 20h14'), document.createTextNode('Descargar'));
    const botonVoltear = crear('button', 'pf-pildora');
    botonVoltear.type = 'button';
    botonVoltear.append(icono('M4 12a8 8 0 0113.7-5.7L20 8.5', 'M20 4v4.5h-4.5', 'M20 12a8 8 0 01-13.7 5.7L4 15.5', 'M4 20v-4.5h4.5'), document.createTextNode('Voltear'));
    pildoras.append(botonTienda, botonBajar, botonVoltear);

    const ayuda = crear('p', 'pf-tienda-ayuda', 'Toca fuera del pase o presiona Esc para cerrar');
    velo.append(cerrar, zona, pildoras, ayuda);

    const anterior = document.activeElement;
    function cerrarVelo() {
      document.removeEventListener('keydown', alTeclear);
      if (document.fullscreenElement && document.exitFullscreen) document.exitFullscreen().catch(() => {});
      velo.remove();
      if (anterior && typeof anterior.focus === 'function') anterior.focus({ preventScroll: true });
    }
    function alTeclear(evento) {
      // Si hay otra ventana encima (la de descargar), el Esc es para ella.
      if (evento.key === 'Escape' && !document.querySelector('.pf-ventana-velo')) cerrarVelo();
    }
    cerrar.addEventListener('click', cerrarVelo);
    velo.addEventListener('click', (evento) => {
      if (evento.target === velo || evento.target === zona || evento.target === ayuda || evento.target === pildoras) cerrarVelo();
    });
    botonTienda.addEventListener('click', () => {
      // Pantalla completa (si el navegador lo permite): el pase se ve sin barras alrededor.
      if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
      else if (velo.requestFullscreen) velo.requestFullscreen().catch(() => {});
    });
    botonBajar.addEventListener('click', () => alDescargar());
    botonVoltear.addEventListener('click', () => velo.classList.toggle('invertido'));
    document.addEventListener('keydown', alTeclear);
    document.body.append(velo);
    botonTienda.focus({ preventScroll: true });
  }

  // ----- Descargas -----
  async function coloresActuales(id = portadaActual()) {
    const prueba = crear('div', 'pf-copia');
    aplicarPortada(prueba, id);
    prueba.style.cssText = 'position:absolute;left:-9999px;width:10px;height:10px';
    document.body.append(prueba);
    const estilo = getComputedStyle(prueba);
    const leerVar = (nombre) => estilo.getPropertyValue(nombre).trim();
    const colores = {
      c1: leerVar('--c1'),
      c2: leerVar('--c2'),
      c3: leerVar('--c3'),
      acento: leerVar('--acento'),
      punto: leerVar('--punto'),
      tinte1: leerVar('--tinte1'),
      tinte2: leerVar('--tinte2'),
      puntos: leerVar('--puntos') || '0',
      imagen: id === PORTADA_PROPIA ? await cargarImagen(leer(CLAVE_PORTADA_IMG)) : null,
    };
    prueba.remove();
    return colores;
  }

  // Tres imágenes PNG: el anverso, el reverso (cada uno de 1012 × 638 px: 85,6 × 54 mm a 300 ppp) y una con los dos, uno debajo del otro.
  async function descargarPng() {
    const colores = await coloresActuales();
    const foto = await cargarImagen(leer(CLAVE_FOTO));
    const datos = datosDelFrente(perfil);
    const anverso = dibujarCara('frente', perfil, colores, foto, datos);
    const reverso = dibujarCara('reverso', perfil, colores, foto, datos);
    const juntas = juntarCaras(anverso, reverso);
    const pausa = () => new Promise((r) => setTimeout(r, 350)); // el navegador no siempre acepta descargas pegadas
    await bajarLienzo(anverso, `pase-${perfil.codigo}-anverso.png`);
    await pausa();
    await bajarLienzo(reverso, `pase-${perfil.codigo}-reverso.png`);
    await pausa();
    await bajarLienzo(juntas, `pase-${perfil.codigo}-anverso-y-reverso.png`);
  }

  // PDF: se arma una hoja A4 con las dos caras a tamaño real y se abre la impresión del navegador («Guardar como PDF»).
  function descargarPdf() {
    const hoja = crear('section', 'pf-hoja-pdf');
    const portada = portadaActual();
    const impresas = construirCaras(perfil, leer(CLAVE_FOTO));
    [impresas.botonEditar, impresas.botonVoltear].forEach((b) => b.remove());
    const titulo = crear('header', 'pf-hoja-titulo');
    titulo.append(crear('span', '', 'Pase XpressMS · tamaño real'));
    hoja.append(titulo);
    [impresas.frente, impresas.dorso].forEach((cara) => {
      const bloque = crear('div', 'pf-hoja-bloque');
      bloque.append(copiaDeCara(cara, portada, 'pf-copia-real'));
      hoja.append(bloque);
    });
    hoja.append(crear('p', 'pf-hoja-nota', 'Recorta por el borde. Medida 85,60 × 53,98 mm (como una tarjeta bancaria).'));
    document.body.append(hoja);
    document.body.classList.add('pf-imprimiendo');
    const limpiar = () => {
      window.removeEventListener('afterprint', limpiar);
      document.body.classList.remove('pf-imprimiendo');
      hoja.remove();
    };
    window.addEventListener('afterprint', limpiar);
    // Un instante para que el navegador pinte la hoja antes de abrir la impresión.
    setTimeout(() => {
      window.print();
      setTimeout(limpiar, 120000); // por si el navegador no avisa al terminar
    }, 60);
  }

  return {
    elemento: raiz,
    girar,
    elegirPortada,
    cambiarFoto,
    fotoActual,
    mostrarEnTienda,
    descargarPng,
    descargarPdf,
    portadaActual,
    iniciales: inicialesDeCarne(perfil.nombre),
  };
}
