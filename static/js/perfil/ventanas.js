// ventanas.js: las dos ventanas del pase de «Mi perfil y datos»: «Cambiar foto de perfil» y «Personalizar portada».
// Las dos solo cambian cómo se VE el pase en este navegador (la foto y la portada se guardan en este dispositivo,
// no viajan al servidor). Nada de la cuenta se modifica aquí; para todo lo demás está Juno.
// Todo texto se pone con textContent (nunca innerHTML).

import { crear } from '../dom.js';
import { aplicarPortada, construirCaras, copiaDeCara, imagenDePortadaPropia, inicialesDeCarne, PORTADA_PROPIA, PORTADAS } from './carne.js';

const ESPACIOS = 'http://www.w3.org/2000/svg';
const LADO = 360; // lado (px) de la foto recortada que se guarda
const MAXIMO_BYTES = 5 * 1024 * 1024;
const MINIMO_FOTO = 400; // la foto, cuadrada: mínimo 400 × 400 px
const MINIMO_PORTADA = 400; // la portada: mínimo 400 px por lado

function icono(...trazos) {
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

const TRAZOS = {
  subir: ['M12 16V5', 'M7 9l5-5 5 5', 'M5 20h14'],
  camara: ['M4 8h3l1.500-2h7L17 8h3v11H4z', 'M12 16.500a3.500 3.500 0 100-7 3.500 3.500 0 000 7z'],
  quitar: ['M5 7h14', 'M10 7V4h4v3', 'M7 7l1 13h8l1-13', 'M10 11v6M14 11v6'],
  equis: ['M6 6l12 12M18 6L6 18'],
  visto: ['M5.500 12.500l4 4 9-9'],
  imagen: ['M4 5h16v14H4z', 'M4 16l5-5 4 4 3-3 4 4', 'M9 9.500a1 1 0 100-2 1 1 0 000 2z'],
  info: ['M12 21a9 9 0 100-18 9 9 0 000 18z', 'M12 11v5', 'M12 7.800v.1'],
  descargar: ['M12 4v11', 'M7 11l5 5 5-5', 'M5 20h14'],
};

// El armazón común: fondo desenfocado, tarjeta blanca con título y botón de cerrar. Esc o tocar fuera la cierra.
function crearVentana(titulo, subtitulo, alCerrar = () => {}) {
  const velo = crear('div', 'pf-ventana-velo');
  const ventana = crear('div', 'pf-ventana');
  ventana.setAttribute('role', 'dialog');
  ventana.setAttribute('aria-modal', 'true');
  ventana.setAttribute('aria-label', titulo);

  const cab = crear('header', 'pf-ventana-cab');
  const textos = crear('div');
  textos.append(crear('h2', '', titulo));
  if (subtitulo) textos.append(crear('p', '', subtitulo));
  const cerrarBoton = crear('button', 'pf-ventana-cerrar');
  cerrarBoton.type = 'button';
  cerrarBoton.setAttribute('aria-label', 'Cerrar');
  cerrarBoton.append(icono(...TRAZOS.equis));
  cab.append(textos, cerrarBoton);
  ventana.append(cab);
  velo.append(ventana);

  const anterior = document.activeElement;
  function cerrar() {
    document.removeEventListener('keydown', alTeclear);
    alCerrar();
    velo.remove();
    if (anterior && typeof anterior.focus === 'function') anterior.focus({ preventScroll: true });
  }
  function alTeclear(evento) {
    if (evento.key === 'Escape') {
      // Si hay otra capa encima (la pantalla de tienda), esa se encarga.
      cerrar();
    }
  }
  cerrarBoton.addEventListener('click', cerrar);
  velo.addEventListener('mousedown', (evento) => {
    if (evento.target === velo) cerrar();
  });
  document.addEventListener('keydown', alTeclear);
  document.body.append(velo);
  cerrarBoton.focus({ preventScroll: true });
  return { ventana, cerrar };
}

function pieDeBotones(textoGuardar) {
  const pie = crear('div', 'pf-ventana-pie');
  const cancelar = crear('button', 'pf-boton-ventana', 'Cancelar');
  cancelar.type = 'button';
  const guardar = crear('button', 'pf-boton-ventana principal', textoGuardar);
  guardar.type = 'button';
  pie.append(cancelar, guardar);
  return { pie, cancelar, guardar };
}

function leerArchivo(archivo) {
  return new Promise((resolver, rechazar) => {
    const direccion = URL.createObjectURL(archivo);
    const imagen = new Image();
    imagen.onload = () => {
      URL.revokeObjectURL(direccion);
      resolver(imagen);
    };
    imagen.onerror = () => {
      URL.revokeObjectURL(direccion);
      rechazar(new Error('No se pudo leer la imagen'));
    };
    imagen.src = direccion;
  });
}

// ---------- Cambiar foto de perfil ----------

// "carne" es lo que devuelve crearCarne; "perfil" son los datos; "alGuardar(mensaje)" avisa el resultado; 
export function abrirVentanaFoto({ carne, perfil, alGuardar }) {
  let flujoCamara = null;
  const detenerCamara = () => {
    if (flujoCamara) flujoCamara.getTracks().forEach((pista) => pista.stop());
    flujoCamara = null;
  };
  const { ventana, cerrar } = crearVentana('Cambiar foto de perfil', 'Elige cómo quieres verte en tu pase.', detenerCamara);

  // Estado de la foto en edición.
  let imagen = null; // lo que se está recortando (null: sin foto nueva)
  let zoom = 1;
  let desplazarX = 0; // en píxeles del lienzo de recorte
  let desplazarY = 0;
  let quitar = false;
  let cambiado = false;

  // ----- Las tres opciones -----
  const entrada = document.createElement('input');
  entrada.type = 'file';
  entrada.accept = 'image/jpeg,image/png';
  entrada.hidden = true;

  const opciones = crear('div', 'pf-ventana-opciones');
  const crearOpcion = (trazos, texto) => {
    const boton = crear('button', 'pf-opcion');
    boton.type = 'button';
    boton.append(icono(...trazos), document.createTextNode(texto));
    return boton;
  };
  const botonSubir = crearOpcion(TRAZOS.subir, 'Subir desde mi equipo');
  const botonCamara = crearOpcion(TRAZOS.camara, 'Tomar una foto');
  const botonQuitar = crearOpcion(TRAZOS.quitar, 'Quitar foto (usar iniciales)');
  opciones.append(botonSubir, botonCamara, botonQuitar);

  // ----- Recorte circular con zoom -----
  const cuerpo = crear('div', 'pf-ventana-cuerpo');
  const ladoRecorte = crear('div');
  const recorte = crear('div', 'pf-recorte');
  const lienzo = document.createElement('canvas');
  lienzo.width = LADO;
  lienzo.height = LADO;
  const vacio = crear('span', 'pf-recorte-vacio', inicialesDeCarne(perfil.nombre));
  recorte.append(vacio, lienzo);

  const zona = crear('label', 'pf-zoom');
  const deslizador = document.createElement('input');
  deslizador.type = 'range';
  deslizador.min = '1';
  deslizador.max = '3';
  deslizador.step = '0.01';
  deslizador.value = '1';
  deslizador.setAttribute('aria-label', 'Zoom');
  zona.append(document.createTextNode('Zoom'), deslizador);

  const capturar = crear('button', 'pf-boton-ventana principal', 'Capturar');
  capturar.type = 'button';
  capturar.style.margin = '14px auto 0';
  capturar.style.display = 'none';
  // Las medidas, cortas, justo debajo del círculo.
  const medidaFoto = crear('p', 'pf-medida-corta', 'JPG o PNG · mín. 400 × 400 px · hasta 5 MB');
  ladoRecorte.append(recorte, medidaFoto, zona, capturar);

  // ----- Cómo se verá en el pase -----
  const ladoPrevia = crear('div', 'pf-previa-lado');
  const mini = crear('div', 'pf-mini-lateral');
  aplicarPortada(mini, carne.portadaActual());
  const circulo = crear('div', 'pf-previa-circulo');
  const inicialesMini = crear('span', '', inicialesDeCarne(perfil.nombre));
  const fotoMini = document.createElement('img');
  fotoMini.alt = '';
  circulo.append(inicialesMini, fotoMini);
  const rotuloMini = crear('small');
  rotuloMini.append(crear('span', '', 'Pase'), document.createElement('br'), crear('span', '', 'XpressMS'));
  mini.append(circulo, rotuloMini);
  ladoPrevia.append(crear('p', 'pf-previa-rotulo', 'Así se verá en tu pase'), mini);
  cuerpo.append(ladoRecorte, ladoPrevia);

  const error = crear('p', 'pf-ventana-error');
  error.hidden = true;
  const { pie, cancelar, guardar } = pieDeBotones('Guardar foto');
  ventana.append(opciones, cuerpo, error, pie, entrada);

  // ----- Dibujo del recorte -----
  function limitar() {
    if (!imagen) return;
    const base = Math.max(LADO / imagen.naturalWidth, LADO / imagen.naturalHeight);
    const escala = base * zoom;
    const maxX = Math.max(0, (imagen.naturalWidth * escala - LADO) / 2);
    const maxY = Math.max(0, (imagen.naturalHeight * escala - LADO) / 2);
    desplazarX = Math.min(maxX, Math.max(-maxX, desplazarX));
    desplazarY = Math.min(maxY, Math.max(-maxY, desplazarY));
  }

  function dibujar() {
    const c = lienzo.getContext('2d');
    c.clearRect(0, 0, LADO, LADO);
    if (imagen) {
      limitar();
      const base = Math.max(LADO / imagen.naturalWidth, LADO / imagen.naturalHeight);
      const escala = base * zoom;
      const ancho = imagen.naturalWidth * escala;
      const alto = imagen.naturalHeight * escala;
      c.drawImage(imagen, (LADO - ancho) / 2 + desplazarX, (LADO - alto) / 2 + desplazarY, ancho, alto);
      fotoMini.src = lienzo.toDataURL('image/jpeg', 0.85);
      fotoMini.hidden = false;
      inicialesMini.hidden = true;
      lienzo.hidden = false;
      vacio.hidden = true;
    } else {
      lienzo.hidden = true;
      vacio.hidden = false;
      fotoMini.hidden = true;
      inicialesMini.hidden = false;
    }
    zona.style.visibility = imagen ? 'visible' : 'hidden';
    guardar.disabled = !cambiado;
    botonQuitar.setAttribute('aria-pressed', String(quitar));
  }

  function mostrarError(texto) {
    error.textContent = texto || '';
    error.hidden = !texto;
  }

  function usarImagen(nueva) {
    detenerCamara();
    recorte.querySelectorAll('video').forEach((v) => v.remove());
    capturar.style.display = 'none';
    imagen = nueva;
    zoom = 1;
    desplazarX = 0;
    desplazarY = 0;
    deslizador.value = '1';
    quitar = false;
    cambiado = true;
    mostrarError('');
    dibujar();
  }

  // La foto que ya tiene el pase aparece de entrada para poder ajustarla.
  const guardada = carne.fotoActual();
  if (guardada) {
    const previa = new Image();
    previa.onload = () => {
      imagen = previa;
      dibujar();
    };
    previa.src = guardada;
  }
  dibujar();

  // ----- Eventos -----
  botonSubir.addEventListener('click', () => entrada.click());
  entrada.addEventListener('change', async () => {
    const archivo = entrada.files && entrada.files[0];
    entrada.value = '';
    if (!archivo) return;
    if (!/^image\/(jpeg|png)$/.test(archivo.type) || archivo.size > MAXIMO_BYTES) {
      mostrarError('Esa imagen no sirve: usa un archivo JPG o PNG de hasta 5 MB.');
      return;
    }
    try {
      const leida = await leerArchivo(archivo);
      if (Math.min(leida.naturalWidth, leida.naturalHeight) < MINIMO_FOTO) {
        mostrarError(`Esa foto es muy pequeña (${leida.naturalWidth} × ${leida.naturalHeight} px). Usa una de al menos ${MINIMO_FOTO} × ${MINIMO_FOTO} px.`);
        return;
      }
      usarImagen(leida);
    } catch {
      mostrarError('No pude leer esa imagen. Prueba con otra.');
    }
  });

  botonQuitar.addEventListener('click', () => {
    detenerCamara();
    recorte.querySelectorAll('video').forEach((v) => v.remove());
    capturar.style.display = 'none';
    imagen = null;
    quitar = true;
    cambiado = Boolean(guardada) || cambiado;
    mostrarError('');
    dibujar();
  });

  botonCamara.addEventListener('click', async () => {
    mostrarError('');
    if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
      mostrarError('Este navegador no deja usar la cámara. Sube una foto desde tu equipo.');
      return;
    }
    try {
      detenerCamara();
      flujoCamara = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'user' }, audio: false });
      const video = document.createElement('video');
      video.srcObject = flujoCamara;
      video.muted = true;
      video.setAttribute('playsinline', '');
      video.style.transform = 'scaleX(-1)'; // como un espejo, así es más natural encuadrarse
      await video.play();
      recorte.querySelectorAll('video').forEach((v) => v.remove());
      recorte.append(video);
      lienzo.hidden = true;
      vacio.hidden = true;
      zona.style.visibility = 'hidden';
      capturar.style.display = 'block';
    } catch {
      detenerCamara();
      mostrarError('No pude abrir la cámara. Revisa el permiso o sube una foto desde tu equipo.');
    }
  });

  capturar.addEventListener('click', () => {
    const video = recorte.querySelector('video');
    if (!video || !video.videoWidth) return;
    const foto = document.createElement('canvas');
    foto.width = video.videoWidth;
    foto.height = video.videoHeight;
    const c = foto.getContext('2d');
    c.translate(foto.width, 0);
    c.scale(-1, 1);
    c.drawImage(video, 0, 0);
    const nueva = new Image();
    nueva.onload = () => usarImagen(nueva);
    nueva.src = foto.toDataURL('image/jpeg', 0.92);
  });

  deslizador.addEventListener('input', () => {
    zoom = Number(deslizador.value);
    cambiado = true;
    dibujar();
  });

  // Arrastrar la foto dentro del círculo para encuadrarla.
  let arrastre = null;
  recorte.addEventListener('pointerdown', (evento) => {
    if (!imagen) return;
    arrastre = { x: evento.clientX, y: evento.clientY, dx: desplazarX, dy: desplazarY };
    recorte.setPointerCapture(evento.pointerId);
    recorte.style.cursor = 'grabbing';
  });
  recorte.addEventListener('pointermove', (evento) => {
    if (!arrastre) return;
    const razon = LADO / recorte.clientWidth;
    desplazarX = arrastre.dx + (evento.clientX - arrastre.x) * razon;
    desplazarY = arrastre.dy + (evento.clientY - arrastre.y) * razon;
    cambiado = true;
    dibujar();
  });
  const soltar = () => {
    arrastre = null;
    recorte.style.cursor = '';
  };
  recorte.addEventListener('pointerup', soltar);
  recorte.addEventListener('pointercancel', soltar);

  cancelar.addEventListener('click', cerrar);
  guardar.addEventListener('click', () => {
    let hecho;
    if (quitar || !imagen) {
      hecho = carne.cambiarFoto(null);
      alGuardar(hecho ? 'Quitamos tu foto: tu pase muestra tus iniciales.' : 'Quitamos tu foto, pero solo mientras esta página esté abierta.');
    } else {
      dibujar(); // deja el lienzo al día con el encuadre final
      hecho = carne.cambiarFoto(lienzo.toDataURL('image/jpeg', 0.88));
      alGuardar(hecho ? 'Foto actualizada en este dispositivo.' : 'Tu foto se ve ahora, pero este navegador no la pudo guardar.');
    }
    cerrar();
  });
}

// ---------- Personalizar portada ----------

// Reduce la imagen elegida a un panel vertical de 480 × 1008 px (JPEG) para que pese poco en el navegador.
function prepararPortada(imagen) {
  const lienzo = document.createElement('canvas');
  lienzo.width = 480;
  lienzo.height = 1008;
  const c = lienzo.getContext('2d');
  const escala = Math.max(480 / imagen.naturalWidth, 1008 / imagen.naturalHeight);
  const w = imagen.naturalWidth * escala;
  const h = imagen.naturalHeight * escala;
  c.drawImage(imagen, (480 - w) / 2, (1008 - h) / 2, w, h);
  return lienzo.toDataURL('image/jpeg', 0.82);
}

export function abrirVentanaPortada({ carne, perfil, alGuardar }) {
  const { ventana, cerrar } = crearVentana('Personalizar portada', 'Elige el color o la imagen de tu pase.');
  let elegida = carne.portadaActual();
  let imagenNueva = null; // la imagen que se acaba de subir (aún sin guardar)
  const imagenPropia = () => imagenNueva || imagenDePortadaPropia();

  const cuerpo = crear('div', 'pf-ventana-cuerpo');

  // ----- 3 × 2 estilos (y «Mi portada» si ya hay una imagen subida) -----
  const muestras = crear('div', 'pf-muestras');
  muestras.setAttribute('role', 'radiogroup');
  muestras.setAttribute('aria-label', 'Estilo de portada del pase');
  const crearMuestra = (id, nombre, fondo) => {
    const opcion = crear('button', 'pf-muestra');
    opcion.type = 'button';
    opcion.setAttribute('role', 'radio');
    opcion.setAttribute('aria-label', nombre);
    opcion.dataset.id = id;
    const color = crear('span', 'pf-muestra-color');
    color.style.setProperty('--fondo-muestra', fondo);
    const marca = crear('span', 'pf-muestra-marca');
    marca.append(icono(...TRAZOS.visto));
    color.append(marca);
    opcion.append(color, crear('span', 'pf-muestra-nombre', nombre));
    muestras.append(opcion);
    return opcion;
  };
  const opciones = PORTADAS.map((portada) => crearMuestra(portada.id, portada.nombre, portada.muestra));
  const muestraPropia = crearMuestra(PORTADA_PROPIA, 'Mi portada', 'none');
  opciones.push(muestraPropia);

  const entrada = document.createElement('input');
  entrada.type = 'file';
  entrada.accept = 'image/jpeg,image/png';
  entrada.hidden = true;
  const propia = crear('button', 'pf-muestra-propia');
  propia.type = 'button';
  const textosPropia = crear('span', 'pf-propia-textos');
  const tituloPropia = crear('b');
  textosPropia.append(tituloPropia, crear('small', '', 'JPG o PNG · 600 × 1260 px · vertical'));
  propia.append(icono(...TRAZOS.imagen), textosPropia);
  muestras.append(propia);

  // ----- Vista previa -----
  const ladoPrevia = crear('div', 'pf-previa-lado');
  const caras = construirCaras(perfil, carne.fotoActual());
  [caras.botonEditar, caras.botonVoltear].forEach((b) => b.remove());
  const previa = copiaDeCara(caras.frente, elegida, 'pf-copia-previa', imagenPropia());
  ladoPrevia.append(crear('p', 'pf-previa-rotulo', 'Vista previa'), previa);
  cuerpo.append(muestras, ladoPrevia);

  const error = crear('p', 'pf-ventana-error');
  error.hidden = true;

  function marcar() {
    const hayPropia = Boolean(imagenPropia());
    muestraPropia.hidden = !hayPropia;
    if (hayPropia) muestraPropia.querySelector('.pf-muestra-color').style.setProperty('--fondo-muestra', `url("${imagenPropia()}") center / cover`);
    tituloPropia.textContent = hayPropia ? 'Cambiar mi portada' : 'Subir mi portada';
    opciones.forEach((opcion) => opcion.setAttribute('aria-checked', String(opcion.dataset.id === elegida)));
    aplicarPortada(previa, elegida, imagenPropia());
  }
  opciones.forEach((opcion) => {
    opcion.addEventListener('click', () => {
      elegida = opcion.dataset.id;
      error.hidden = true;
      marcar();
    });
  });

  propia.addEventListener('click', () => entrada.click());
  entrada.addEventListener('change', async () => {
    const archivo = entrada.files && entrada.files[0];
    entrada.value = '';
    if (!archivo) return;
    const mostrar = (texto) => {
      error.textContent = texto;
      error.hidden = !texto;
    };
    if (!/^image\/(jpeg|png)$/.test(archivo.type) || archivo.size > MAXIMO_BYTES) {
      mostrar('Esa imagen no sirve: usa un archivo JPG o PNG de hasta 5 MB.');
      return;
    }
    try {
      const leida = await leerArchivo(archivo);
      if (Math.min(leida.naturalWidth, leida.naturalHeight) < MINIMO_PORTADA) {
        mostrar(`Esa imagen es muy pequeña (${leida.naturalWidth} × ${leida.naturalHeight} px). Usa una de al menos ${MINIMO_PORTADA} px por lado.`);
        return;
      }
      imagenNueva = prepararPortada(leida);
      elegida = PORTADA_PROPIA;
      mostrar('');
      marcar();
    } catch {
      mostrar('No pude leer esa imagen. Prueba con otra.');
    }
  });
  marcar();

  const { pie, cancelar, guardar } = pieDeBotones('Guardar portada');
  cancelar.addEventListener('click', cerrar);
  guardar.addEventListener('click', () => {
    const hecho = elegida === PORTADA_PROPIA ? carne.elegirPortada(PORTADA_PROPIA, imagenNueva) : carne.elegirPortada(elegida);
    alGuardar(hecho ? 'Portada actualizada.' : 'Tu portada se ve ahora, pero este navegador no la pudo guardar.');
    cerrar();
  });
  ventana.append(cuerpo, error, pie, entrada);
}

// ---------- Descargar el pase: vista previa de lo que se va a bajar ----------

export function abrirVentanaDescarga({ carne, perfil }) {
  const { ventana, cerrar } = crearVentana(
    'Descargar el pase',
    'Se descarga con la medida real de tarjeta. El archivo trae anverso y reverso, así que se trata como dato sensible.',
  );
  ventana.classList.add('pf-ventana-ancha');
  const portada = carne.portadaActual();
  const foto = carne.fotoActual();
  const dosCaras = (clase) => {
    const caras = construirCaras(perfil, foto);
    [caras.botonEditar, caras.botonVoltear].forEach((b) => b.remove());
    return [copiaDeCara(caras.frente, portada, clase), copiaDeCara(caras.dorso, portada, clase)];
  };

  const rejilla = crear('div', 'pf-descarga-rejilla');
  const tarjeta = (titulo, detalle, vista, textoBoton, hacer) => {
    const caja = crear('section', 'pf-descarga-card');
    const boton = crear('button', 'pf-boton-ventana principal pf-descarga-boton');
    boton.type = 'button';
    boton.append(icono(...TRAZOS.descargar), document.createTextNode(textoBoton));
    boton.addEventListener('click', hacer);
    caja.append(crear('h3', '', titulo), crear('p', 'pf-descarga-detalle', detalle), vista, boton);
    return caja;
  };

  // Imagen PNG: anverso y reverso uno debajo del otro.
  const vistaPng = crear('div', 'pf-descarga-vista');
  const [frentePng, dorsoPng] = dosCaras('pf-copia-previa');
  vistaPng.append(frentePng, dorsoPng);

  // Documento PDF: una hoja A4 con las dos caras.
  const vistaPdf = crear('div', 'pf-descarga-vista pf-descarga-vista-pdf');
  const hoja = crear('div', 'pf-pdf-hoja');
  const [frentePdf, dorsoPdf] = dosCaras('pf-copia-previa');
  const contenido = crear('div', 'pf-pdf-contenido');
  contenido.append(
    crear('span', 'pf-pdf-titulo', 'Pase XpressMS · tamaño real'),
    frentePdf,
    dorsoPdf,
    crear('span', 'pf-pdf-nota', 'Recorta por el borde. Medida 85,60 × 53,98 mm (como una tarjeta bancaria).'),
  );
  hoja.append(contenido);
  vistaPdf.append(hoja);

  rejilla.append(
    tarjeta('Imagen (PNG)', '3 imágenes: anverso, reverso y ambos juntos · 1012 × 638 px por lado · 300 ppp', vistaPng, 'Descargar PNG', async () => {
      cerrar();
      await carne.descargarPng();
    }),
    tarjeta('Documento (PDF)', 'Una hoja A4 con anverso y reverso a tamaño real · para imprimir y plastificar', vistaPdf, 'Descargar PDF', () => {
      cerrar();
      carne.descargarPdf();
    }),
  );
  ventana.append(rejilla);
}
