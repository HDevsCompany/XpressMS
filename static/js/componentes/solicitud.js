// solicitud.js (componentes): «Tu solicitud» en /servicios: el cajón con los 5 pasos, el selector de día y hora, y la confirmación.
// Solo MUESTRA lo que Juno ya hizo (el servidor manda el estado completo en cada respuesta). Los botones no ejecutan nada por su cuenta:
// le piden a Juno que lo haga con "ordenar(texto)" («Confirmar cita», «Cancelar solicitud», «mar 10/11», «15:00»…).

import { crear } from '../dom.js';
import { dibujoDeProducto } from './catalogo_arte.js';
import { varianteDe } from './catalogo_comun.js';
import { arteDeServicio, fondoDe, iconoSvg } from './servicios.js';

const FOTO_JUNO = '/static/img/juno.png';
const EN_CELULAR = '(max-width: 820px)';
const NS_SVG = 'http://www.w3.org/2000/svg';

const ETIQUETAS_PASOS = ['Servicio', 'Equipo', 'Problema', 'Día y hora', 'Confirmar'];

// Lo que Juno le dice a la persona en la franja del cajón, según en qué va.
const AYUDA = {
  servicio: 'Dile a Juno qué servicio necesitas.',
  equipo: 'Cuéntale a Juno qué equipo es.',
  detalle: 'Cuéntale a Juno qué le pasa a tu equipo.',
  dia: 'Elige el día y la hora de tu cita.',
  hora: 'Elige el día y la hora de tu cita.',
  confirmar: 'Revisa los datos y confirma tu cita.',
  registro: 'Juno necesita saber quién eres para registrar tu cita.',
};

function svgTrazos(trazos, clase = '') {
  const svg = document.createElementNS(NS_SVG, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('aria-hidden', 'true');
  if (clase) svg.setAttribute('class', clase);
  for (const trazo of trazos) {
    const camino = document.createElementNS(NS_SVG, 'path');
    camino.setAttribute('d', trazo);
    svg.appendChild(camino);
  }
  return svg;
}

function fotoJuno() {
  const imagen = crear('img');
  imagen.src = FOTO_JUNO;
  imagen.alt = '';
  return imagen;
}

function esCelular() {
  return window.matchMedia(EN_CELULAR).matches;
}

function tituloDe(estado) {
  return estado.modo === 'diagnostico' ? 'Tu cita de diagnóstico' : 'Tu solicitud';
}

// "ordenar(texto)": le pide algo a Juno. "abrirJuno()" / "minimizarJuno()": su ventana. "alTerminar()": la solicitud terminó o se dejó.
export function iniciarSolicitud({ ordenar, abrirJuno, minimizarJuno, alTerminar }) {
  let estado = null; // lo último que mandó Juno
  let cajon = null; // el cajón «Tu solicitud» (elemento) o null
  let velo = null; // el velo detrás del selector o de la confirmación
  let selector = null; // el selector de día y hora, o la confirmación
  let chip = null; // el botón que reabre el cajón
  let cajonOculto = false; // la persona lo minimizó
  let selectorDescartado = false; // la persona cerró el selector sin elegir
  let horaElegida = null;
  let ocupado = false;
  let faseAnterior = '';

  // ---------- Pedirle cosas a Juno ----------

  async function pedir(texto) {
    if (ocupado) return;
    ocupado = true;
    selector?.classList.add('ocupado');
    try {
      await ordenar(texto);
    } finally {
      ocupado = false;
      selector?.classList.remove('ocupado');
    }
  }

  // ---------- Cerrar cosas ----------

  function quitarConSalida(elemento) {
    if (!elemento) return;
    elemento.classList.add('saliendo');
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) elemento.remove();
    else setTimeout(() => elemento.remove(), 160);
  }

  function cerrarSelector() {
    quitarConSalida(selector);
    quitarConSalida(velo);
    selector = null;
    velo = null;
    document.removeEventListener('keydown', alTeclado);
  }

  function cerrarCajon() {
    quitarConSalida(cajon);
    cajon = null;
    document.body.classList.remove('sv-con-cajon');
  }

  function quitarChip() {
    chip?.remove();
    chip = null;
  }

  function cerrarTodo() {
    cerrarSelector();
    cerrarCajon();
    quitarChip();
    estado = null;
    cajonOculto = false;
    selectorDescartado = false;
    horaElegida = null;
    faseAnterior = '';
  }

  function alTeclado(evento) {
    if (evento.key === 'Escape' && selector) {
      evento.preventDefault();
      descartarSelector();
    }
  }

  // La persona cierra el selector (o la confirmación) sin seguir.
  function descartarSelector() {
    if (estado?.fase === 'listo') {
      terminar();
      return;
    }
    selectorDescartado = true;
    cerrarSelector();
    dibujar();
  }

  // La confirmación se cerró: ya no hay solicitud en curso.
  function terminar() {
    cerrarTodo();
    alTerminar?.();
  }

  // ---------- El cajón «Tu solicitud» ----------

  function paso(numero, estadoPaso, titulo, valor, extra, adicional) {
    const fila = crear('div', `sv-st ${estadoPaso}`);
    const circulo = crear('span', 'n', estadoPaso === 'ok' ? '✓' : String(numero));
    const texto = crear('div');
    texto.append(crear('small', '', titulo.toUpperCase()), crear('b', '', valor));
    if (extra) texto.append(crear('div', 'v', extra));
    if (adicional) texto.append(adicional);
    fila.append(circulo, texto);
    return fila;
  }

  function lineaDePasos() {
    const e = estado;
    const actual = e.paso;
    const diagnostico = e.modo === 'diagnostico';
    const marca = (n) => (n < actual ? 'ok' : n === actual ? 'on' : 'pend');
    const filas = [];

    filas.push(
      paso(
        1,
        marca(1),
        diagnostico ? 'Tipo de cita' : 'Servicio',
        e.servicio || 'Elige un servicio con Juno',
        e.servicio_linea,
      ),
    );
    filas.push(paso(2, marca(2), 'Equipo', e.equipo || 'Pendiente', null));
    filas.push(
      paso(
        3,
        marca(3),
        diagnostico ? 'Qué notas en el equipo' : 'Qué le pasa',
        e.detalle || 'Pendiente',
        null,
      ),
    );

    let cuando = 'Pendiente';
    let cambiar = null;
    if (e.cita) {
      cuando = e.cita;
      cambiar = crear('button', 'ed', 'Cambiar');
      cambiar.type = 'button';
      cambiar.addEventListener('click', () => pedir('Cambiar fecha'));
    } else if (e.dia) {
      cuando = `${e.dia} · elige la hora`;
    }
    filas.push(paso(4, marca(4), 'Día y hora', cuando, e.fase === 'dia' || e.fase === 'hora' ? e.horario : null, cambiar));

    filas.push(
      paso(
        5,
        marca(5),
        'Confirmar',
        e.fase === 'confirmar' ? 'Revisa y confirma tu cita' : 'Al final',
        null,
      ),
    );
    return filas;
  }

  function dibujarCajon() {
    const e = estado;
    const nuevo = crear('aside', 'sv-sol');
    nuevo.setAttribute('role', 'complementary');
    nuevo.setAttribute('aria-label', tituloDe(e));

    const cabecera = crear('div', 'sv-sol-cab');
    const titulo = crear('h2', '', tituloDe(e));
    titulo.append(crear('small', '', `paso ${e.paso} de 5`));
    const minimizar = crear('button', 'sv-x');
    minimizar.type = 'button';
    minimizar.setAttribute('aria-label', 'Minimizar');
    minimizar.append(iconoSvg('cerrar'));
    minimizar.addEventListener('click', () => {
      cajonOculto = true;
      dibujar();
    });
    cabecera.append(titulo, minimizar);

    const ayuda = crear('div', 'sv-sol-jn');
    ayuda.append(fotoJuno(), crear('span', '', AYUDA[e.fase] || AYUDA.servicio));

    const lineas = crear('div', 'sv-sol-tl');
    lineas.append(...lineaDePasos());

    const pie = crear('div', 'sv-sol-pie');
    const principal = crear('button', 'sv-bn p');
    principal.type = 'button';
    if (e.fase === 'dia' || e.fase === 'hora') {
      principal.append(fotoJuno(), 'Elegir día y hora');
      principal.addEventListener('click', () => {
        selectorDescartado = false;
        dibujar();
      });
    } else {
      principal.append(fotoJuno(), 'Confirmar cita');
      principal.disabled = e.fase !== 'confirmar';
      principal.addEventListener('click', () => pedir('Confirmar cita'));
    }
    const cancelar = crear('button', 'sv-sol-cancelar', 'Cancelar solicitud');
    cancelar.type = 'button';
    cancelar.addEventListener('click', () => pedir('Cancelar solicitud'));
    pie.append(principal, cancelar);

    nuevo.append(cabecera, ayuda, lineas, pie);
    if (cajon) {
      nuevo.style.animation = 'none'; // no vuelve a entrar con cada respuesta de Juno
      cajon.replaceWith(nuevo);
    } else document.body.append(nuevo);
    cajon = nuevo;
    if (!esCelular()) document.body.classList.add('sv-con-cajon');
  }

  function dibujarChip() {
    const e = estado;
    const nuevo = crear('button', 'sv-chip');
    nuevo.type = 'button';
    nuevo.append(crear('i', '', String(e.paso)), `${tituloDe(e)} · paso ${e.paso} de 5`);
    nuevo.addEventListener('click', () => {
      cajonOculto = false;
      if (esCelular()) minimizarJuno();
      dibujar();
    });
    chip?.remove();
    chip = nuevo;
    document.body.append(nuevo);
  }

  // ---------- El selector de día y hora ----------

  function resumenDerecha() {
    const e = estado;
    const lado = crear('div', 'r');
    lado.append(crear('span', 'rot', e.modo === 'diagnostico' ? 'TU CITA DE DIAGNÓSTICO' : 'TU SOLICITUD'));

    const servicio = crear('div', 'sv-rs');
    const tope = crear('div', 'tp');
    const imagen = crear('span', 'im');
    imagen.style.background = fondoDe(e.servicio ? e.servicio.length : 0);
    imagen.append(dibujoDeProducto(arteDeServicio(e.servicio || ''), varianteDe(e.servicio || '')));
    const texto = crear('div');
    texto.append(crear('small', '', e.modo === 'diagnostico' ? 'TIPO DE CITA' : 'SERVICIO'), crear('b', '', e.servicio || '—'));
    if (e.servicio_linea) texto.append(crear('small', '', e.servicio_linea));
    tope.append(imagen, texto);
    servicio.append(tope);
    lado.append(servicio);

    if (e.equipo) {
      const equipo = crear('div', 'sv-rs');
      equipo.append(crear('small', '', 'EQUIPO'), crear('b', '', e.equipo));
      lado.append(equipo);
    }
    if (e.detalle) {
      const detalle = crear('div', 'sv-rs');
      detalle.append(crear('small', '', 'QUÉ LE PASA'), crear('b', '', e.detalle));
      lado.append(detalle);
    }
    const cuando = crear('div', 'sv-rs cita');
    const elegida = e.dia && horaElegida ? `${e.dia} · ${horaElegida}` : e.dia || 'Elige un día';
    cuando.append(crear('small', '', 'DÍA Y HORA'), crear('b', '', elegida));
    lado.append(cuando);
    return lado;
  }

  function barraDePasos(actual) {
    const barra = crear('div', 'sv-pas');
    ETIQUETAS_PASOS.forEach((nombre, i) => {
      const numero = i + 1;
      const item = crear('span', numero < actual ? 'ok' : numero === actual ? 'on' : '');
      item.append(crear('i', '', numero < actual ? '✓' : String(numero)), nombre);
      barra.append(item);
      if (i < ETIQUETAS_PASOS.length - 1) barra.append(crear('hr'));
    });
    return barra;
  }

  function dibujarSelector() {
    const e = estado;
    const nuevo = crear('section', 'sv-ck');
    nuevo.setAttribute('role', 'dialog');
    nuevo.setAttribute('aria-modal', 'true');
    nuevo.setAttribute('aria-label', 'Elige el día y la hora');

    const izquierda = crear('div', 'l');
    const cerrar = crear('button', 'sv-x');
    cerrar.type = 'button';
    cerrar.setAttribute('aria-label', 'Cerrar');
    cerrar.append(iconoSvg('cerrar'));
    cerrar.addEventListener('click', descartarSelector);

    izquierda.append(cerrar, barraDePasos(4));
    izquierda.append(crear('h2', '', 'Elige el día y la hora'));
    izquierda.append(
      crear(
        'p',
        'sub',
        e.modo === 'diagnostico'
          ? 'Lleva tu equipo al local en ese horario y lo revisamos.'
          : 'Lleva tu equipo al local en ese horario para que lo atiendan.',
      ),
    );

    izquierda.append(crear('div', 'sv-lb', 'DÍA'));
    const dias = crear('div', 'sv-dias');
    for (const dia of e.dias) {
      const boton = crear('button', dia.elegido ? 'sv-dia on' : 'sv-dia');
      boton.type = 'button';
      boton.append(crear('small', '', dia.semana), crear('b', '', String(dia.numero)), crear('span', '', dia.mes));
      boton.addEventListener('click', () => {
        if (dia.elegido) return;
        horaElegida = null;
        pedir(dia.texto);
      });
      dias.append(boton);
    }
    izquierda.append(dias);

    izquierda.append(crear('div', 'sv-lb', 'HORA'));
    if (e.fase === 'hora' && e.horas.length > 0) {
      if (!e.horas.some((h) => h.texto === horaElegida && h.cupos > 0)) horaElegida = null;
      const horas = crear('div', 'sv-horas');
      for (const hora of e.horas) {
        const lleno = hora.cupos === 0;
        const clases = ['sv-hora'];
        if (lleno) clases.push('lleno');
        else if (hora.cupos === 1) clases.push('poco');
        if (hora.texto === horaElegida) clases.push('on');
        const boton = crear('button', clases.join(' '));
        boton.type = 'button';
        boton.disabled = lleno;
        boton.append(hora.texto, crear('small', '', lleno ? 'Lleno' : hora.cupos === 1 ? '1 cupo' : `${hora.cupos} cupos`));
        boton.addEventListener('click', () => {
          horaElegida = hora.texto;
          dibujarSelectorNuevo();
        });
        horas.append(boton);
      }
      izquierda.append(horas);
    } else {
      horaElegida = null;
      izquierda.append(crear('p', 'sv-vacio', 'Elige un día para ver las horas disponibles.'));
    }

    if (e.horario) {
      const nota = crear('p', 'sv-hor');
      nota.append(svgTrazos(['M12 3.5a8.5 8.5 0 100 17 8.5 8.5 0 000-17z', 'M12 7.5V12l3 2']), e.horario);
      izquierda.append(nota);
    }

    const botones = crear('div', 'sv-bsx');
    const volver = crear('button', 'sv-bn s', 'Cerrar');
    volver.type = 'button';
    volver.addEventListener('click', descartarSelector);
    const seguir = crear('button', 'sv-bn p');
    seguir.type = 'button';
    seguir.disabled = !horaElegida;
    seguir.append(fotoJuno(), 'Continuar con Juno');
    seguir.addEventListener('click', () => {
      if (horaElegida) pedir(horaElegida);
    });
    botones.append(volver, seguir);
    izquierda.append(botones);

    nuevo.append(izquierda, resumenDerecha());
    return nuevo;
  }

  // Cambia de hora dentro del selector sin hablar con Juno todavía: solo se vuelve a dibujar.
  function dibujarSelectorNuevo() {
    if (!selector || !estado) return;
    const arriba = selector.querySelector('.l')?.scrollTop || 0;
    const nuevo = dibujarSelector();
    nuevo.style.animation = 'none';
    selector.replaceWith(nuevo);
    selector = nuevo;
    const lado = nuevo.querySelector('.l');
    if (lado) lado.scrollTop = arriba;
  }

  function abrirVelo(alCerrar) {
    if (velo) return;
    velo = crear('div', 'sv-velo');
    velo.addEventListener('click', alCerrar);
    document.body.append(velo);
    document.addEventListener('keydown', alTeclado);
  }

  function mostrarSelector() {
    if (!selector) {
      minimizarJuno(); // la ventana de Juno no tapa el selector
      abrirVelo(descartarSelector);
      selector = dibujarSelector();
      document.body.append(selector);
      selector.querySelector('.sv-hora.on, .sv-dia.on, .sv-dia')?.focus({ preventScroll: true });
    } else {
      const arriba = selector.querySelector('.l')?.scrollTop || 0;
      const nuevo = dibujarSelector();
      nuevo.style.animation = 'none';
      selector.replaceWith(nuevo);
      selector = nuevo;
      const lado = nuevo.querySelector('.l');
      if (lado) lado.scrollTop = arriba;
    }
  }

  // ---------- La confirmación ----------

  function dibujarListo() {
    const e = estado;
    const nuevo = crear('section', 'sv-ck');
    nuevo.setAttribute('role', 'dialog');
    nuevo.setAttribute('aria-modal', 'true');
    nuevo.setAttribute('aria-label', 'Tu cita quedó registrada');

    const izquierda = crear('div', 'l');
    const cerrar = crear('button', 'sv-x');
    cerrar.type = 'button';
    cerrar.setAttribute('aria-label', 'Cerrar');
    cerrar.append(iconoSvg('cerrar'));
    cerrar.addEventListener('click', terminar);

    const marca = crear('div', 'sv-okc');
    marca.append(svgTrazos(['M5 12.5l4.5 4.5L19 7.5']));
    const titulo = crear('h2', '', e.modo === 'diagnostico' ? '¡Tu cita de diagnóstico quedó registrada!' : '¡Tu solicitud quedó registrada!');
    titulo.style.textAlign = 'center';
    const sub = crear('p', 'sub', e.cita ? `Te esperamos el ${e.cita}.` : 'Te esperamos en el local.');
    sub.style.textAlign = 'center';

    const codigo = crear('div', 'sv-cod');
    codigo.append(crear('small', '', 'CÓDIGO DE TU ORDEN'), crear('b', '', e.orden?.codigo || ''));

    izquierda.append(cerrar, marca, titulo, sub, codigo);

    const hitos = e.orden?.seguimiento || [];
    if (hitos.length > 0) {
      const seguimiento = crear('div', 'sv-seg');
      seguimiento.style.gridTemplateColumns = `repeat(${hitos.length}, 1fr)`;
      for (const hito of hitos) {
        const clase = hito.estado === 'hecho' ? 'h' : hito.estado === 'actual' ? 'a' : '';
        seguimiento.append(crear('div', clase, hito.nombre));
      }
      izquierda.append(seguimiento);
    }

    const globo = crear('div', 'sv-jb');
    globo.append(
      fotoJuno(),
      crear('div', '', `Guarda tu código: con él puedes preguntarme cómo va tu equipo cuando quieras.`),
    );
    izquierda.append(globo);

    const botones = crear('div', 'sv-bsx');
    const seguirViendo = crear('button', 'sv-bn s', 'Seguir viendo servicios');
    seguirViendo.type = 'button';
    seguirViendo.addEventListener('click', terminar);
    const verSeguimiento = crear('button', 'sv-bn p');
    verSeguimiento.type = 'button';
    verSeguimiento.append(fotoJuno(), 'Ver el seguimiento');
    verSeguimiento.addEventListener('click', () => {
      terminar();
      abrirJuno();
      ordenar('Ver mi orden');
    });
    botones.append(seguirViendo, verSeguimiento);
    izquierda.append(botones);

    const lado = crear('div', 'r');
    lado.append(crear('span', 'rot', 'RESUMEN'));
    const filas = [
      [e.modo === 'diagnostico' ? 'TIPO DE CITA' : 'SERVICIO', e.servicio],
      ['EQUIPO', e.equipo],
      ['QUÉ LE PASA', e.detalle],
    ];
    for (const [rotulo, valor] of filas) {
      if (!valor) continue;
      const fila = crear('div', 'sv-rs');
      fila.append(crear('small', '', rotulo), crear('b', '', valor));
      lado.append(fila);
    }
    if (e.cita) {
      const cuando = crear('div', 'sv-rs cita');
      cuando.append(crear('small', '', 'DÍA Y HORA'), crear('b', '', e.cita));
      lado.append(cuando);
    }

    nuevo.append(izquierda, lado);
    return nuevo;
  }

  function mostrarListo() {
    cerrarCajon();
    quitarChip();
    cerrarSelector();
    minimizarJuno();
    abrirVelo(terminar);
    selector = dibujarListo();
    document.body.append(selector);
    selector.querySelector('.sv-bn.p')?.focus({ preventScroll: true });
  }

  // ---------- Poner todo al día ----------

  function dibujar() {
    if (!estado) return;
    const e = estado;
    const enSelector = (e.fase === 'dia' || e.fase === 'hora') && !selectorDescartado;

    if (enSelector) mostrarSelector();
    else if (selector) cerrarSelector();

    // El cajón: en computador está abierto salvo que la persona lo minimice; en celular solo se abre al confirmar o cuando lo pide.
    const celular = esCelular();
    const abrirCajon = !enSelector && !cajonOculto && (!celular || e.fase === 'confirmar');
    if (abrirCajon) {
      if (celular) minimizarJuno();
      dibujarCajon();
      quitarChip();
    } else {
      cerrarCajon();
      if (enSelector) quitarChip();
      else dibujarChip();
    }
  }

  function actualizar(nuevoEstado) {
    if (!nuevoEstado || nuevoEstado.tipo !== 'solicitud') return;
    if (nuevoEstado.fase === 'fin') {
      const habia = estado !== null;
      cerrarTodo();
      if (habia) alTerminar?.();
      return;
    }
    if (nuevoEstado.fase === 'listo') {
      estado = nuevoEstado;
      faseAnterior = 'listo';
      mostrarListo();
      return;
    }
    const salioDelSelector = (faseAnterior === 'dia' || faseAnterior === 'hora') && nuevoEstado.fase !== 'dia' && nuevoEstado.fase !== 'hora';
    if (nuevoEstado.fase !== 'dia' && nuevoEstado.fase !== 'hora') selectorDescartado = false;
    if (faseAnterior === 'listo' || faseAnterior === '') cajonOculto = false;
    estado = nuevoEstado;
    faseAnterior = nuevoEstado.fase;
    dibujar();
    // Al elegir la hora, Juno vuelve a mostrarse (junto al cajón en computador) para que se vea su pregunta y la persona le responda.
    // En celular, al confirmar la hoja «Tu solicitud» ya trae el botón, así que Juno no se abre encima.
    if (salioDelSelector && !(esCelular() && nuevoEstado.fase === 'confirmar')) abrirJuno();
  }

  return { actualizar, cerrarTodo, hayAlgoAbierto: () => selector !== null || cajon !== null };
}
