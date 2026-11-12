// perfil.js: la página «Mi perfil y datos» (/perfil). Solo MUESTRA: pide los datos al servidor (POST /perfil/datos) y los dibuja.
// Todo lo que cambia algo se lo pide a Juno: los botones «… con Juno» dejan el mensaje en sessionStorage y abren el lienzo,
// donde Juno lo ejecuta (corregir un dato, añadir una dirección, autorizar a alguien...).
//
// Izquierda (ancha): el pase con sus acciones rápidas, las direcciones de entrega y recojo y las personas autorizadas.
// Derecha: datos personales, contacto y privacidad.
// Todo texto se pone con textContent (nunca innerHTML): los datos vienen de la base de datos.

import { datosDePerfil, datosDeSesion } from './api.js';
import {
  borrarTema,
  borrarToken,
  dejarDestino,
  dejarMensajePendiente,
  dejarNuevaConversacion,
  leerToken,
} from './almacen.js';
import { crear } from './dom.js';
import { cerrarMenu, iniciarMenu, mostrarCliente, mostrarVisitante } from './menu.js';
import { crearCarne } from './perfil/carne.js';
import { abrirVentanaDescarga, abrirVentanaFoto, abrirVentanaPortada } from './perfil/ventanas.js';
import { abrirIngreso } from './publico/ingreso.js';

const ESPACIOS = 'http://www.w3.org/2000/svg';

const visitante = document.getElementById('pf-visitante');
const cargando = document.getElementById('pf-cargando');
const error = document.getElementById('pf-error');
const contenido = document.getElementById('pf-contenido');
const izquierda = document.getElementById('pf-izquierda');
const derecha = document.getElementById('pf-derecha');
const barra = document.getElementById('pf-barra');

// ---------- Piezas de dibujo ----------

// Un icono de línea. "trazos" son los "d" de los <path>; el estilo (grosor, color) lo da lienzo.css.
function icono(trazos) {
  const dibujo = document.createElementNS(ESPACIOS, 'svg');
  dibujo.setAttribute('viewBox', '0 0 24 24');
  dibujo.setAttribute('aria-hidden', 'true');
  trazos.forEach((d) => {
    const trazo = document.createElementNS(ESPACIOS, 'path');
    trazo.setAttribute('d', d);
    dibujo.appendChild(trazo);
  });
  return dibujo;
}

const ICONOS = {
  persona: ['M12 12a4 4 0 100-8 4 4 0 000 8z', 'M4.5 20.5c0-4 3.4-6.5 7.5-6.5s7.5 2.5 7.5 6.5'],
  documento: ['M3 6h18v12H3z', 'M7 10.5a1.8 1.8 0 103.6 0 1.8 1.8 0 00-3.6 0z', 'M6.5 15c.5-1.2 1.4-1.8 2.3-1.8s1.8.6 2.3 1.8', 'M14 10h4M14 14h3'],
  pastel: ['M5 20h14v-5H5z', 'M7 15v-3h10v3', 'M12 12V8', 'M12 4.5c.8.8 1 1.5 0 2.5-1-1-.8-1.700 0-2.500z'],
  celular: ['M8 3h8a1 1 0 011 1v16a1 1 0 01-1 1H8a1 1 0 01-1-1V4a1 1 0 011-1z', 'M11 18h2'],
  sobre: ['M4 6h16v12H4z', 'M4 7l8 6 8-6'],
  alfiler: ['M12 21s-7-6.2-7-11a7 7 0 0114 0c0 4.8-7 11-7 11z', 'M12 12.5a2.5 2.5 0 100-5 2.5 2.5 0 000 5z'],
  grupo: ['M9 11a3.5 3.5 0 100-7 3.5 3.5 0 000 7z', 'M2.5 20c0-3.6 2.900-6 6.500-6s6.500 2.400 6.500 6', 'M16 4.500a3.300 3.300 0 010 6.500', 'M18 14.200c2.200.6 3.500 2.600 3.500 5.800'],
  escudo: ['M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z', 'M9 12l2 2 4-4'],
  candado: ['M6 11h12v9H6z', 'M8.500 11V8a3.500 3.500 0 017 0v3'],
  ojo: ['M2 12s3.600-7 10-7 10 7 10 7-3.600 7-10 7S2 12 2 12z', 'M12 15a3 3 0 100-6 3 3 0 000 6z'],
  expandir: ['M4 9V4h5', 'M20 9V4h-5', 'M4 15v5h5', 'M20 15v5h-5'],
  descargar: ['M12 4v11', 'M7 11l5 5 5-5', 'M5 20h14'],
  camara: ['M4 8h3l1.500-2h7L17 8h3v11H4z', 'M12 16.500a3.500 3.500 0 100-7 3.500 3.500 0 000 7z'],
  pincel: ['M4 20l4-1 10-10-3-3L5 16z', 'M13.500 7.500l3 3'],
  girar: ['M4 12a8 8 0 0113.700-5.700L20 8.500', 'M20 4v4.500h-4.500', 'M20 12a8 8 0 01-13.700 5.700L4 15.500', 'M4 20v-4.500h4.500'],
  flecha: ['M9 6l6 6-6 6'],
  libro: ['M7 3h10v18H7z', 'M10 8h4M10 12h4M10 16h2'],
  equis: ['M6 6l12 12M18 6L6 18'],
  mas: ['M12 5v14M5 12h14'],
  info: ['M12 21a9 9 0 100-18 9 9 0 000 18z', 'M12 11v5', 'M12 7.800v.1'],
  visto: ['M5.500 12.500l4 4 9-9'],
};

// Pide algo a Juno: deja el mensaje y abre el lienzo, donde Juno lo atiende.
function pedirAJuno(mensaje) {
  dejarMensajePendiente(mensaje);
  location.assign('/chat');
}

// La cara redonda de Juno (para los botones «… con Juno»).
function caraDeJuno() {
  const cara = document.createElement('img');
  cara.src = '/static/img/juno.png';
  cara.alt = '';
  cara.className = 'pf-juno-cara';
  return cara;
}

// El botón en forma de píldora «… con Juno».
function botonJuno(texto, mensaje) {
  const boton = crear('button', 'pf-juno');
  boton.type = 'button';
  boton.append(caraDeJuno(), document.createTextNode(texto));
  boton.addEventListener('click', () => pedirAJuno(mensaje));
  return boton;
}

// Una tarjeta con título (icono azul + nombre) y una línea de explicación debajo. "derecha" es lo que va al lado del título.
function tarjeta(titulo, subtitulo, trazos, derechaDelTitulo = null, iconoSubtitulo = null) {
  const caja = crear('section', 'pf-tarjeta');
  const cabecera = crear('header', 'pf-tarjeta-cab');
  const textos = crear('div', 'pf-tarjeta-titulos');
  const linea = crear('h2');
  linea.append(icono(trazos), document.createTextNode(titulo));
  const sub = crear('p', 'pf-tarjeta-sub');
  if (iconoSubtitulo) sub.append(icono(iconoSubtitulo));
  sub.append(document.createTextNode(subtitulo));
  textos.append(linea, sub);
  cabecera.append(textos);
  if (derechaDelTitulo) cabecera.append(derechaDelTitulo);
  caja.append(cabecera);
  return caja;
}

// Una fila con icono en un cuadro suave, «etiqueta — valor» y, a la derecha, lo que se le quiera poner.
function fila(trazos, etiqueta, valor, extra = null) {
  const linea = crear('div', 'pf-fila');
  const marcaIcono = crear('span', 'pf-fila-icono');
  marcaIcono.append(icono(trazos));
  const textos = crear('div', 'pf-fila-textos');
  const valorEl = crear('span', 'pf-fila-valor');
  if (typeof valor === 'string') valorEl.textContent = valor;
  else valorEl.append(valor);
  textos.append(crear('small', '', etiqueta), valorEl);
  linea.append(marcaIcono, textos);
  if (extra) linea.append(extra);
  return linea;
}

function ficha(texto, clase = 'verde') {
  return crear('span', 'pf-ficha ' + clase, texto);
}

function conVisto(texto) {
  const marca = crear('span', 'pf-visto');
  marca.append(icono(ICONOS.visto), document.createTextNode(texto));
  return marca;
}

// ---------- Pase y acciones rápidas ----------

// Un botón grande de «Acciones rápidas»: título, explicación abajo y una flecha a la derecha.
function botonAccion(trazos, titulo, explicacion, hacer, principal = false) {
  const boton = crear('button', 'pf-accion' + (principal ? ' pf-accion-principal' : ''));
  boton.type = 'button';
  const marcaIcono = crear('span', 'pf-accion-icono');
  marcaIcono.append(icono(trazos));
  const textos = crear('span', 'pf-accion-textos');
  textos.append(crear('b', '', titulo), crear('small', '', explicacion));
  const flecha = crear('span', 'pf-accion-flecha');
  flecha.append(icono(ICONOS.flecha));
  boton.append(marcaIcono, textos, flecha);
  boton.addEventListener('click', () => hacer(boton));
  return boton;
}

function dibujarFilaCarne(perfil) {
  const fila = crear('div', 'pf-fila-carne');

  const zonaCarne = crear('section', 'pf-zona-carne');
  zonaCarne.id = 'pf-carne';
  // Los datos de contexto de las dos ventanas (foto y portada).
  const avisoGuardado = (mensaje) => avisar(mensaje);
  const abrirPortada = () => abrirVentanaPortada({ carne, perfil, alGuardar: avisoGuardado });
  const abrirFoto = () => abrirVentanaFoto({ carne, perfil, alGuardar: avisoGuardado });
  const abrirDescarga = () => abrirVentanaDescarga({ carne, perfil });
  const carne = crearCarne(perfil, { alEditar: abrirPortada, alDescargar: abrirDescarga });
  zonaCarne.append(carne.elemento);

  const acciones = crear('section', 'pf-tarjeta pf-acciones');
  acciones.append(
    crear('h2', 'pf-rotulo-acciones', 'Acciones rápidas'),
    botonAccion(ICONOS.expandir, 'Mostrar en tienda', 'Código ampliado', () => carne.mostrarEnTienda(), true),
    botonAccion(ICONOS.descargar, 'Descargar pase', 'PNG o PDF', abrirDescarga),
    botonAccion(ICONOS.camara, 'Cambiar foto', 'Subir o tomar', abrirFoto),
    botonAccion(ICONOS.pincel, 'Personalizar portada', '6 estilos', abrirPortada),
  );
  const pista = crear('p', 'pf-acciones-pista');
  pista.append(icono(ICONOS.girar), document.createTextNode('Toca el pase para voltearlo'));
  acciones.append(pista);

  fila.append(zonaCarne, acciones);
  return fila;
}

// ---------- Direcciones ----------

// Un mapa de adorno (no es una ubicación real): calles, un parque y un alfiler azul. "variante" cambia el dibujo.
function mapaDeAdorno(variante) {
  const dibujo = document.createElementNS(ESPACIOS, 'svg');
  dibujo.setAttribute('viewBox', '0 0 300 120');
  dibujo.setAttribute('preserveAspectRatio', 'xMidYMid slice');
  dibujo.setAttribute('class', 'pf-mapa-dibujo');
  dibujo.setAttribute('aria-hidden', 'true');
  const agregar = (etiqueta, atributos) => {
    const elemento = document.createElementNS(ESPACIOS, etiqueta);
    Object.entries(atributos).forEach(([nombre, valor]) => elemento.setAttribute(nombre, valor));
    dibujo.appendChild(elemento);
  };
  agregar('rect', { width: 300, height: 120, fill: '#e6eefb' });
  if (variante % 2 === 0) {
    agregar('path', { d: 'M0 20 L95 8 L120 48 L0 92 Z', fill: '#d4ecd9' });
    agregar('path', { d: 'M0 62 L140 120 L0 120 Z', fill: '#cfe3f7' });
    agregar('path', { d: 'M96 0 L128 120', stroke: '#fff', 'stroke-width': 11, fill: 'none' });
    agregar('path', { d: 'M222 0 L196 120', stroke: '#fff', 'stroke-width': 9, fill: 'none' });
    agregar('path', { d: 'M0 74 L300 54', stroke: '#fff', 'stroke-width': 12, fill: 'none' });
  } else {
    agregar('path', { d: 'M178 0 L300 0 L300 58 L214 42 Z', fill: '#d4ecd9' });
    agregar('path', { d: 'M0 90 L84 120 L0 120 Z', fill: '#cfe3f7' });
    agregar('path', { d: 'M58 0 L92 120', stroke: '#fff', 'stroke-width': 10, fill: 'none' });
    agregar('path', { d: 'M176 0 L158 120', stroke: '#fff', 'stroke-width': 11, fill: 'none' });
    agregar('path', { d: 'M0 44 L300 70', stroke: '#fff', 'stroke-width': 12, fill: 'none' });
  }
  const x = variante % 2 === 0 ? 150 : 140;
  agregar('ellipse', { cx: x, cy: 92, rx: 11, ry: 4, fill: 'rgba(15,23,42,0.18)' });
  agregar('path', { d: `M${x} 88 C${x - 20} 64 ${x - 22} 46 ${x - 22} 40 a22 22 0 0144 0 C${x + 22} 46 ${x + 20} 64 ${x} 88z`, fill: '#1d4ed8' });
  agregar('circle', { cx: x, cy: 40, r: 8.500, fill: '#fff' });
  return dibujo;
}

function dibujarDirecciones(perfil) {
  const caja = tarjeta(
    'Direcciones de entrega y recojo',
    'Dónde te llevamos o recogemos tus equipos',
    ICONOS.alfiler,
    botonJuno('Añadir o cambiar con Juno', 'Mis direcciones'),
  );

  const rejilla = crear('div', 'pf-direcciones');
  perfil.direcciones.forEach((d, indice) => {
    const ficha_ = crear('article', 'pf-direccion' + (d.principal ? ' principal' : ''));
    const mapa = crear('div', 'pf-mapa');
    mapa.append(mapaDeAdorno(indice));
    const cuerpo = crear('div', 'pf-direccion-cuerpo');
    const titulo = crear('div', 'pf-direccion-titulo');
    titulo.append(crear('b', '', d.nombre));
    if (d.principal) titulo.append(conVistoFicha('Principal'));
    cuerpo.append(titulo, crear('p', 'pf-direccion-texto', d.direccion));
    if (d.referencia) cuerpo.append(crear('p', 'pf-direccion-ref', 'Ref.: ' + d.referencia));
    if (!d.principal) {
      const hacer = crear('button', 'pf-enlace-suave', 'Hacer principal con Juno');
      hacer.type = 'button';
      hacer.addEventListener('click', () => pedirAJuno(`Quiero hacer principal mi dirección ${d.nombre}`));
      cuerpo.append(hacer);
    }
    ficha_.append(mapa, cuerpo);
    rejilla.append(ficha_);
  });

  if (perfil.direcciones.length < perfil.max_direcciones) {
    const nueva = crear('button', 'pf-direccion pf-nueva');
    nueva.type = 'button';
    const mas = crear('span', 'pf-nueva-mas');
    mas.append(icono(ICONOS.mas));
    nueva.append(mas, crear('b', '', 'Añadir dirección'), crear('small', '', 'Dile a Juno dónde recibir o recoger'));
    nueva.addEventListener('click', () => pedirAJuno('Quiero añadir una dirección'));
    rejilla.append(nueva);
  }
  caja.append(rejilla, nota('Usamos tus direcciones solo para coordinar envíos y recojos de tus equipos.', ICONOS.info));
  return caja;
}

// «✓ Principal»
function conVistoFicha(texto) {
  const marca = crear('span', 'pf-ficha azul');
  marca.append(icono(ICONOS.visto), document.createTextNode(texto));
  return marca;
}

// Una nota con icono al pie de una tarjeta, separada por una línea de puntos.
function nota(texto, trazos) {
  const linea = crear('p', 'pf-nota');
  linea.append(icono(trazos), document.createTextNode(texto));
  return linea;
}

// ---------- Personas autorizadas ----------

// Las iniciales de una persona autorizada: la del nombre y la del primer apellido
// («Rosa Elena Quispe Vargas» -> «RQ»; con tres palabras o menos, la primera y la última).
function inicialesDePersona(nombre) {
  const palabras = nombre.split(/\s+/).filter(Boolean);
  if (palabras.length === 0) return '?';
  const segunda = palabras.length >= 4 ? palabras[2] : palabras[palabras.length - 1];
  return (palabras[0].charAt(0) + (palabras.length > 1 ? segunda.charAt(0) : '')).toUpperCase();
}

function dibujarPersonas(perfil) {
  const contador = crear('span', 'pf-contador', `${perfil.personas.length} de ${perfil.max_personas}`);
  const caja = tarjeta('Personas autorizadas para recojo', 'Pueden retirar tus equipos del taller si tú no puedes ir', ICONOS.grupo, contador);

  const lista = crear('ul', 'pf-personas');
  perfil.personas.forEach((p, indice) => {
    const item = crear('li', 'pf-persona');
    item.append(crear('span', 'pf-persona-avatar tono' + (indice % 4), inicialesDePersona(p.nombre)));
    const textos = crear('div', 'pf-persona-textos');
    const titulo = crear('div', 'pf-persona-titulo');
    titulo.append(crear('b', '', p.nombre), crear('span', 'pf-ficha gris', p.parentesco));
    textos.append(titulo, crear('small', '', p.documento));
    const quitar = crear('button', 'pf-enlace-suave', 'Quitar con Juno');
    quitar.type = 'button';
    quitar.addEventListener('click', () => pedirAJuno(`Quiero quitar a ${p.nombre} de mis personas autorizadas`));
    item.append(textos, ficha('Vigente'), quitar);
    lista.append(item);
  });
  if (perfil.personas.length === 0) {
    lista.append(crear('li', 'pf-vacio', 'Nadie más puede recoger tus equipos. Si lo necesitas, autoriza a alguien de confianza.'));
  }
  caja.append(lista);

  if (perfil.personas.length < perfil.max_personas) {
    const nueva = crear('button', 'pf-autorizar');
    nueva.type = 'button';
    const mas = crear('span', 'pf-nueva-mas');
    mas.append(icono(ICONOS.mas));
    const textos = crear('span', 'pf-autorizar-textos');
    textos.append(crear('b', '', 'Autorizar nueva persona con Juno'), crear('small', '', 'Nombre, DNI y relación; confirmas con un código SMS'));
    nueva.append(mas, textos);
    nueva.addEventListener('click', () => pedirAJuno('Quiero autorizar a alguien para recoger'));
    caja.append(nueva);
  }
  caja.append(nota('Por seguridad, la persona autorizada deberá presentar su DNI físico al momento del recojo en tienda.', ICONOS.escudo));
  return caja;
}

// ---------- Tarjetas de la derecha ----------

function dibujarDatosPersonales(perfil) {
  const caja = tarjeta('Datos personales', 'Vienen de tu documento de identidad', ICONOS.persona, null, ICONOS.candado);
  const esEmpresa = perfil.variante === 'empresa';

  const nombre = fila(ICONOS.persona, esEmpresa ? 'Razón social' : 'Nombre completo', perfil.nombre);
  nombre.querySelector('.pf-fila-valor').classList.add('mayusculas');
  caja.append(nombre);

  // El documento sale oculto; «Mostrar» lo descubre y «Ocultar» lo vuelve a tapar.
  const mostrar = crear('button', 'pf-mostrar');
  mostrar.type = 'button';
  const textoMostrar = crear('span', '', 'Mostrar');
  mostrar.append(icono(ICONOS.ojo), textoMostrar);
  const filaDocumento = fila(ICONOS.documento, 'Documento', perfil.documento, mostrar);
  const valorDocumento = filaDocumento.querySelector('.pf-fila-valor');
  let visible = false;
  mostrar.addEventListener('click', () => {
    visible = !visible;
    valorDocumento.textContent = visible ? perfil.documento_completo : perfil.documento;
    textoMostrar.textContent = visible ? 'Ocultar' : 'Mostrar';
    mostrar.setAttribute('aria-pressed', String(visible));
  });
  caja.append(filaDocumento);

  if (perfil.nacimiento) caja.append(fila(ICONOS.pastel, 'Fecha de nacimiento', perfil.nacimiento));
  if (perfil.sexo) caja.append(fila(ICONOS.persona, 'Sexo', perfil.sexo));

  const pie = crear('div', 'pf-tarjeta-pie');
  pie.append(crear('span', '', 'Si algo no coincide, díselo a Juno.'), botonJuno('Corregir con Juno', 'Quiero corregir mis datos'));
  caja.append(pie);
  return caja;
}

function dibujarContacto(perfil) {
  const caja = tarjeta('Contacto', 'Con esto te avisamos cómo va tu equipo', ICONOS.celular);

  // El celular sale oculto salvo los 3 últimos dígitos: «9•• ••• 321».
  const celular = perfil.celular.replace(/^\+51\s*/, '');
  const valorCelular = crear('span', 'pf-valor-con-marca');
  valorCelular.append(document.createTextNode(celular));
  if (perfil.celular_verificado) valorCelular.append(conVisto('Verificado'));
  caja.append(fila(ICONOS.celular, 'Celular', valorCelular, botonJuno('Cambiar con Juno', 'Quiero cambiar mi celular')));

  if (perfil.correo) {
    caja.append(fila(ICONOS.sobre, 'Correo electrónico', perfil.correo, botonJuno('Cambiar con Juno', 'Quiero cambiar mi correo')));
  } else {
    caja.append(fila(ICONOS.sobre, 'Correo electrónico', 'Sin correo registrado', botonJuno('Agregar con Juno', 'Quiero agregar mi correo')));
  }

  caja.append(fila(ICONOS.alfiler, 'Dirección', perfil.direccion, botonJuno('Cambiar con Juno', 'Quiero cambiar mi dirección')));
  return caja;
}

function dibujarPrivacidad(perfil) {
  const caja = tarjeta('Privacidad y consentimientos', 'Lo que aceptaste al registrarte', ICONOS.escudo);

  caja.append(fila(ICONOS.libro, 'Términos y condiciones', 'Aceptados', ficha('Vigente')));
  caja.append(fila(ICONOS.escudo, 'Política de privacidad', 'Aceptada', ficha('Vigente')));
  caja.append(
    fila(
      ICONOS.celular,
      'Avisos por SMS de tus pedidos',
      perfil.celular_verificado ? 'Activados' : 'Pendientes de confirmar tu celular',
      perfil.celular_verificado ? ficha('Activo') : ficha('Pendiente', 'ambar'),
    ),
  );

  const acciones = crear('div', 'pf-privacidad-acciones');
  const ver = crear('a', 'pf-boton-sec');
  ver.href = '/static/legal/terminos.html';
  ver.target = '_blank';
  ver.rel = 'noopener';
  ver.append(icono(ICONOS.ojo), document.createTextNode('Ver términos'));

  const pedir = crear('button', 'pf-boton-sec');
  pedir.type = 'button';
  pedir.append(icono(ICONOS.libro), document.createTextNode('Pedir mis datos a Juno'));
  pedir.addEventListener('click', () => pedirAJuno('Quiero pedir mis datos'));

  const eliminar = crear('button', 'pf-boton-sec pf-peligro');
  eliminar.type = 'button';
  eliminar.append(icono(ICONOS.equis), document.createTextNode('Eliminar mi cuenta con Juno'));
  eliminar.addEventListener('click', () => pedirAJuno('Quiero eliminar mi cuenta'));

  acciones.append(ver, pedir, eliminar);
  caja.append(acciones);
  return caja;
}

// ---------- Avisos breves ----------

let temporizadorAviso = null;
function avisar(texto) {
  let aviso = document.getElementById('pf-aviso-breve');
  if (!aviso) {
    aviso = crear('div', 'pf-aviso-breve');
    aviso.id = 'pf-aviso-breve';
    aviso.setAttribute('role', 'status');
    document.body.append(aviso);
  }
  aviso.textContent = texto;
  aviso.classList.add('visible');
  clearTimeout(temporizadorAviso);
  temporizadorAviso = setTimeout(() => aviso.classList.remove('visible'), 3200);
}

// ---------- Estados de la página ----------

function mostrarSolo(cual) {
  cargando.hidden = cual !== 'cargando';
  visitante.hidden = cual !== 'visitante';
  error.hidden = cual !== 'error';
  contenido.hidden = cual !== 'contenido';
  barra.hidden = cual !== 'contenido';
}

function dibujarPerfil(perfil) {
  izquierda.replaceChildren(dibujarFilaCarne(perfil), dibujarDirecciones(perfil), dibujarPersonas(perfil));
  derecha.replaceChildren(dibujarDatosPersonales(perfil), dibujarContacto(perfil), dibujarPrivacidad(perfil));
}

async function cargar() {
  mostrarSolo('cargando');
  const token = leerToken();
  if (!token) {
    mostrarVisitante();
    mostrarSolo('visitante');
    return;
  }

  const [sesion, resultado] = await Promise.all([datosDeSesion(token), datosDePerfil(token)]);
  if (resultado.estado === 'ok') {
    mostrarCliente(sesion.nombre || (resultado.perfil.nombre || '').split(' ')[0]);
    dibujarPerfil(resultado.perfil);
    mostrarSolo('contenido');
  } else if (resultado.estado === 'visitante') {
    mostrarVisitante();
    mostrarSolo('visitante');
  } else {
    if (sesion.reconocido) mostrarCliente(sesion.nombre);
    else mostrarVisitante();
    mostrarSolo('error');
  }
}

// ---------- Menú y barra «Pídele algo a Juno» ----------

function iniciarSesion() {
  dejarDestino('/perfil'); // la ventana de ingreso termina en el lienzo; desde ahí se vuelve a esta página
  abrirIngreso({ lado: 'cliente' });
}

iniciarMenu({
  alElegir: (mensaje) => pedirAJuno(mensaje),
  alNueva: () => {
    dejarNuevaConversacion();
    location.assign('/chat');
  },
  alIngresar: iniciarSesion,
  alSalir: () => {
    borrarToken();
    borrarTema();
    location.assign('/');
  },
});

// En el menú, esta página queda marcada.
const enlacePerfil = document.getElementById('menu-perfil');
if (enlacePerfil) enlacePerfil.setAttribute('aria-current', 'page');

document.getElementById('pf-iniciar').addEventListener('click', iniciarSesion);
document.getElementById('pf-reintentar').addEventListener('click', cargar);

barra.addEventListener('submit', (evento) => {
  evento.preventDefault();
  const campo = document.getElementById('pf-barra-texto');
  const texto = campo.value.trim();
  if (texto === '') return;
  cerrarMenu({ devolverFoco: false });
  pedirAJuno(texto);
});

// En pantallas angostas el texto de ayuda de la barra se acorta para que no se corte.
function ajustarAyuda() {
  document.getElementById('pf-barra-texto').placeholder =
    window.innerWidth < 600
      ? 'Pídele algo a Juno...'
      : 'Pídele algo a Juno: cambiar mi celular, corregir un dato, pedir mis datos...';
}
ajustarAyuda();
window.addEventListener('resize', ajustarAyuda);

cargar();
