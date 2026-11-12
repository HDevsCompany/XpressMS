// borrador.js (panel/servicios): el bloque «Progreso de creación de ficha». Solo MUESTRA lo que Juno armó:
// el anillo de avance, los cinco pasos conectados, la vista previa (Tarjeta | Ficha), las tarjetas de datos, la imagen y el aviso de precio.
// Todos sus botones le piden algo a Juno (no guardan nada por su cuenta).
//
// Colores con significado: verde = avanzar, amarillo = cambiar, rojo = quitar o retroceder.

import { crear } from '../../dom.js';
import { fichaEnLinea, mostrarServicios } from '../../componentes/servicios.js';
import { icono } from './iconos.js';

const PASOS = [
  ['nombre', 'Nombre'],
  ['descripcion', 'Descripción'],
  ['precio', 'Precio base'],
  ['tiempo', 'Tiempo'],
  ['equipos', 'Equipos'],
];
const HECHOS = ['ok', 'jn', 'cambio'];

const AYUDA = {
  nombre: 'Dime cómo se llama',
  descripcion: 'La redacto cuando tenga el nombre',
  precio: '¿Tiene precio base o se cotiza?',
  tiempo: 'Dime cuánto tarda, o que va sin tiempo fijo',
  equipos: 'Dime para qué equipos sirve',
};

function iconoDeNodo(estado, indice) {
  const i = crear('i');
  if (estado === 'ok' || estado === 'jn') i.append(icono('check', 3));
  else if (estado === 'cambio') i.append(icono('lapiz', 2.6));
  else if (estado === 'quitado') i.append(icono('equis', 3));
  else i.textContent = String(indice + 1);
  i.querySelector('svg')?.setAttribute('width', '12');
  i.querySelector('svg')?.setAttribute('height', '12');
  return i;
}

function textoFaltan(n) {
  return n === 1 ? 'falta 1 dato' : 'faltan ' + n + ' datos';
}

export function crearBorrador(contenedor, acciones) {
  let vista = 'tarjeta';
  let previo = null; // { estados: {clave: estado}, pct }
  let ultimo = null; // lo último que se dibujó, para poder redibujar al cambiar de vista

  // ---------- Los cinco pasos conectados ----------

  function pasos(estados, anteriores, tono) {
    const cx = crear('div', 'cx');
    const nodos = [];
    const lineas = [];
    PASOS.forEach(([clave, etiqueta], i) => {
      if (i > 0) {
        const linea = crear('div', 'cx-l');
        const relleno = crear('u');
        linea.append(relleno);
        cx.append(linea);
        lineas.push(relleno);
      }
      const nodo = crear('div', 'cx-n ' + (anteriores[clave] || 'falta'));
      nodo.append(iconoDeNodo(anteriores[clave] || 'falta', i), crear('span', '', etiqueta));
      cx.append(nodo);
      nodos.push(nodo);
    });

    const llenar = (i, estado) => {
      if (i === 0) return;
      const relleno = lineas[i - 1];
      relleno.className = estado === 'cambio' ? 'am' : '';
      relleno.style.width = HECHOS.includes(estado) ? '100%' : '0%';
    };
    PASOS.forEach(([clave], i) => llenar(i, anteriores[clave] || 'falta'));

    // Un cuadro después de dibujar, se pasa a lo nuevo: así el relleno y los colores se animan.
    requestAnimationFrame(() =>
      requestAnimationFrame(() => {
        PASOS.forEach(([clave], i) => {
          const antes = anteriores[clave] || 'falta';
          const ahora = estados[clave] || 'falta';
          if (antes === ahora) return;
          const nodo = nodos[i];
          const retrocede = HECHOS.includes(antes) && !HECHOS.includes(ahora);
          const avanza = !HECHOS.includes(antes) && HECHOS.includes(ahora);
          nodo.replaceChildren(iconoDeNodo(retrocede ? 'quitado' : ahora, i), crear('span', '', PASOS[i][1]));
          nodo.className = 'cx-n ' + (retrocede ? 'quitado' : ahora) + (avanza ? ' pop' : '');
          if (retrocede && i > 0) {
            // Retroceso: la línea se vacía en rojo, el paso se sacude y luego queda como pendiente.
            const relleno = lineas[i - 1];
            relleno.className = 'rojo';
            relleno.style.width = '0%';
            if (ahora === 'falta') {
              setTimeout(() => {
                if (!nodo.isConnected) return;
                nodo.className = 'cx-n falta';
                nodo.replaceChildren(iconoDeNodo('falta', i), crear('span', '', PASOS[i][1]));
                relleno.className = '';
              }, 1500);
            }
          } else {
            llenar(i, ahora);
          }
        });
        // En edición los pasos cambiados se pintan en amarillo.
        if (tono === 'am') lineas.forEach((l, k) => estados[PASOS[k + 1][0]] === 'cambio' && (l.className = 'am'));
      }),
    );
    return cx;
  }

  // ---------- Qué ve el cliente ----------

  function servicioDe(b, imagenUrl) {
    return {
      nombre: b.nombre || 'Nombre del servicio',
      descripcion: b.descripcion || 'Aquí aparecerá la descripción del servicio.',
      precio: b.precio,
      tiempo: b.tiempo,
      equipos: b.equipos,
      aviso_precio: b.aviso_precio,
      imagen: imagenUrl,
    };
  }

  function imagenVisible(b, nueva) {
    if (b.imagen === 'nueva') return nueva || null;
    if (b.imagen === 'actual') return b.imagen_actual || null;
    return null;
  }

  function vistaPrevia(b, nueva) {
    const servicio = servicioDe(b, imagenVisible(b, nueva));
    if (vista === 'ficha') {
      const cuerpo = crear('div', 'pj-bor-cuerpo fch');
      const caja = crear('div');
      cuerpo.append(caja);
      fichaEnLinea(caja, servicio);
      // Lo que cambió se marca con «antes …» sobre su caja.
      setTimeout(() => {
        const cajas = caja.querySelectorAll('.sv-cajas > div');
        let k = 0;
        for (const clave of ['precio', 'tiempo']) {
          const dato = b.datos.find((d) => d.clave === clave);
          const presente = clave === 'precio' || b.tiempo;
          if (!presente) continue;
          const celda = cajas[k++];
          if (celda && dato && dato.estado === 'cambio' && dato.antes) {
            celda.style.position = 'relative';
            celda.style.boxShadow = '0 0 0 2px #f59e0b';
            const globo = crear('span', 'globo-cambio');
            globo.append('antes ', crear('s', '', dato.antes));
            celda.append(globo);
          }
        }
      }, 30);
      return { cuerpo, completo: true };
    }
    const lz = crear('div', 'lz');
    const cont = crear('div');
    lz.append(cont);
    mostrarServicios(cont, [servicio], { solicitar() {}, preguntar() {}, diagnostico() {} });
    return { cuerpo: lz, completo: false };
  }

  // ---------- Tarjetas de datos, imagen y aviso ----------

  function tarjetaDato(d, foco) {
    const caja = crear('div', 'pj-dato ' + d.estado + (foco.includes(d.clave) ? ' foco' : ''));
    const marca = crear('i');
    if (d.estado === 'falta') marca.textContent = '?';
    else if (d.estado === 'ok' || d.estado === 'jn') marca.append(icono('check', 3));
    else if (d.estado === 'cambio') marca.append(icono('lapiz', 2.6));
    else marca.append(icono('equis', 3));
    marca.querySelector('svg')?.setAttribute('width', '13');
    marca.querySelector('svg')?.setAttribute('height', '13');

    const texto = crear('div');
    const titulo = d.clave === 'descripcion' ? 'DESCRIPCIÓN PARA EL CLIENTE' : d.titulo.toUpperCase();
    const valor = crear('b', d.clave === 'descripcion' ? 'recorte' : '');
    if (d.estado === 'falta') {
      valor.textContent = 'Falta';
    } else if (d.antes) {
      valor.append(crear('s', '', d.antes), ' → ' + d.valor);
    } else {
      valor.textContent = d.valor;
    }
    texto.append(crear('small', '', titulo), valor);
    const sub = d.estado === 'falta' ? AYUDA[d.clave] : d.sub;
    if (sub) texto.append(crear('em', '', sub));
    caja.append(marca, texto);
    return caja;
  }

  function tarjetaImagen(b, nueva) {
    const estado = b.imagen;
    const clase = estado === 'nueva' ? (b.modo === 'editar' ? 'cambio' : 'con') : estado === 'actual' ? 'con' : estado === 'quitar' ? 'quitar' : '';
    const caja = crear('div', 'pj-img ' + clase);
    const marca = crear('i');
    marca.append(icono(estado === 'quitar' ? 'equis' : 'imagen', 2.4));
    const texto = crear('div');
    texto.append(crear('small', '', 'IMAGEN · OPCIONAL'));

    const fila = crear('div', 'pj-img-fila');
    const boton = (clase_, nombre, etiqueta, accion) => {
      const x = crear('button', 'pj-mini-btn ' + clase_);
      x.type = 'button';
      x.append(icono(nombre, 2.4), etiqueta);
      x.addEventListener('click', accion);
      return x;
    };
    if (estado === 'nueva' || estado === 'actual') {
      const url = imagenVisible(b, nueva);
      if (url) {
        const miniatura = crear('img');
        miniatura.src = url;
        miniatura.alt = '';
        fila.append(miniatura);
      }
      texto.append(
        crear('b', '', estado === 'nueva' ? 'Imagen nueva' : 'Imagen actual'),
        crear('em', '', estado === 'nueva' ? 'Se sube cuando confirmes' : 'La que ven tus clientes ahora'),
      );
      fila.append(boton('a', 'lapiz', 'Cambiar', acciones.alImagen), boton('r', 'equis', 'Quitar', acciones.alQuitarImagen));
    } else if (estado === 'quitar') {
      texto.append(crear('b', '', 'Se quitará la imagen'), crear('em', '', 'Volverá el dibujo de siempre al guardar'));
      fila.append(boton('', 'check', 'Deshacer', acciones.alRestaurarImagen));
    } else {
      texto.append(crear('b', '', 'Sin imagen'), crear('em', '', 'Si no pones una, se muestra el dibujo de siempre'));
      fila.append(boton('', 'mas', 'Agregar imagen', acciones.alImagen));
    }
    texto.append(fila);
    caja.append(marca, texto);
    return caja;
  }

  function interruptor(b) {
    const sinPrecio = b.precio === null || b.precio === undefined;
    const activo = b.aviso_precio && !sinPrecio;
    const caja = crear('div', 'pj-sw' + (b.aviso_cambio ? ' cambio' : '') + (sinPrecio ? ' off' : ''));
    const texto = crear('div');
    texto.append(crear('small', '', 'AVISO AL CLIENTE'), crear('b', '', 'El precio puede cambiar'));
    const detalle = crear('em');
    if (sinPrecio) {
      detalle.textContent = 'Disponible cuando la ficha tenga precio base';
    } else {
      detalle.append('Muestra ', crear('b', 'rs', 'El costo final se confirma después de revisar tu equipo'));
    }
    texto.append(detalle);
    const boton = crear('button', 'sw' + (activo ? ' on' : ''));
    boton.type = 'button';
    boton.disabled = sinPrecio;
    boton.setAttribute('role', 'switch');
    boton.setAttribute('aria-checked', String(activo));
    boton.setAttribute('aria-label', 'El precio puede cambiar');
    boton.append(crear('i'));
    boton.addEventListener('click', () => acciones.alAviso(!b.aviso_precio));
    caja.append(texto, boton);
    return caja;
  }

  // ---------- Encabezado y pie ----------

  function cabecera(b, estadosActuales, anillo) {
    const cab = crear('div', 'pj-bor-cab');
    const nCambios = b
      ? b.datos.filter((d) => d.estado === 'cambio' || d.estado === 'quitado').length +
        (b.aviso_cambio ? 1 : 0) +
        (b.modo === 'editar' && (b.imagen === 'nueva' || b.imagen === 'quitar') ? 1 : 0)
      : 0;
    const modo = b ? b.modo : '';
    const total = b ? b.total : 5;
    const hechos = b ? b.completos : 0;
    const pct = Math.round((hechos / total) * 100);
    const lleno = hechos === total;

    const aro = crear('div', 'anillo' + (modo === 'editar' ? ' am' : modo === 'eliminar' ? ' ro' : lleno ? ' lleno' : ''));
    aro.style.setProperty('--p', String(anillo.desde));
    const centro = crear('b');
    if (modo === 'eliminar') centro.append(icono('equis', 3));
    else if (modo === 'editar') centro.append(icono('lapiz', 2.6));
    else if (lleno) centro.append(icono('check', 3));
    else centro.textContent = hechos + '/' + total;
    centro.querySelector('svg')?.setAttribute('width', '20');
    centro.querySelector('svg')?.setAttribute('height', '20');
    aro.append(centro);
    requestAnimationFrame(() => requestAnimationFrame(() => aro.style.setProperty('--p', String(modo === 'eliminar' ? 100 : pct))));

    const tt = crear('div', 'tt');
    const titulo = crear('b');
    const sello = (clase, texto) => crear('span', 'pj-sello ' + clase, texto);
    if (modo === 'editar') titulo.append('Editando ficha', sello('am', 'EDITANDO'));
    else if (modo === 'eliminar') titulo.append('Eliminar servicio', sello('ro', 'SE ELIMINARÁ'));
    else titulo.append('Progreso de creación de ficha', modo ? sello('', 'NO PUBLICADO') : '');
    const detalle = crear('span');
    if (modo === 'editar') {
      if (nCambios === 0) detalle.textContent = 'Sin cambios todavía';
      else detalle.append(crear('em', '', nCambios === 1 ? '1 cambio' : nCambios + ' cambios'), ' sin guardar');
    } else if (modo === 'eliminar') {
      detalle.textContent = 'Los clientes dejarán de verlo';
    } else if (lleno) {
      detalle.append(crear('em', '', 'Ficha completa'), ' · lista para crear');
    } else {
      detalle.append(crear('em', '', 'Ficha al ' + pct + '%'), ' · ' + textoFaltan(total - hechos));
    }
    tt.append(titulo, detalle);
    cab.append(aro, tt);

    if (modo) {
      const vistas = crear('div', 'vistas');
      for (const [id, texto] of [['tarjeta', 'Tarjeta'], ['ficha', 'Ficha']]) {
        const boton = crear('button', vista === id ? 'on' : '', texto);
        boton.type = 'button';
        boton.addEventListener('click', () => {
          if (vista === id) return;
          vista = id;
          if (ultimo) dibujar(ultimo.b, { ...ultimo.opciones, sinAnimar: true });
        });
        vistas.append(boton);
      }
      cab.append(vistas);
    }
    return cab;
  }

  function pie(b) {
    const caja = crear('div', 'pj-bor-pie');
    const nota = crear('span', 'nota');
    nota.append(icono('info', 2));
    const boton = (clase, texto, accion, ico) => {
      const x = crear('button', clase);
      x.type = 'button';
      if (ico) x.append(icono(ico, 3));
      x.append(texto);
      x.addEventListener('click', () => acciones.alAccion(accion, texto));
      return x;
    };
    if (b.modo === 'editar') {
      const faltan = b.datos.filter((d) => d.estado === 'falta').length;
      nota.append(
        faltan > 0 ? 'Falta ' + faltan + (faltan === 1 ? ' dato' : ' datos') + ' para poder guardar.' : b.hay_cambios ? 'Lo amarillo es lo que cambiaste.' : 'Pídele a Juno lo que quieras cambiar.',
      );
      const guardar = boton('pj-ok', 'Guardar cambio', 'guardar', 'check');
      guardar.disabled = !b.hay_cambios || faltan > 0;
      caja.append(nota, boton('pj-txt', 'Descartar', 'descartar'), boton('pj-ghost', 'Cambiar algo', 'cambiar'), guardar);
    } else {
      const faltan = b.datos.filter((d) => d.estado === 'falta').length;
      nota.append(b.listo ? 'Hasta que confirmes, los clientes no lo ven.' : (faltan === 1 ? 'Falta 1 dato' : 'Faltan ' + faltan + ' datos') + ' para poder crearlo.');
      const crearBoton = boton('pj-ok', 'Crear servicio', 'crear', 'check');
      crearBoton.disabled = !b.listo;
      caja.append(nota, boton('pj-txt', 'Descartar', 'descartar'), boton('pj-ghost', 'Cambiar algo', 'cambiar'), crearBoton);
    }
    return caja;
  }

  // ---------- Dibujar ----------

  // "b": el borrador que mandó Juno (o null si no hay ficha). Opciones: { foco, imagenNueva (url local), sinAnimar }.
  function dibujar(b, opciones = {}) {
    ultimo = { b, opciones };
    const foco = opciones.foco || [];
    const estadosNuevos = {};
    if (b) for (const d of b.datos) estadosNuevos[d.clave] = d.estado;
    if (opciones.reiniciar) previo = null;
    // Se anima de lo anterior a lo nuevo. Al empezar a editar o eliminar un servicio no hay nada que animar: ya viene completo.
    const cambioDeModo = b && b.modo !== 'crear' && (!previo || previo.modo !== b.modo);
    const sinAnimacion = opciones.sinAnimar || cambioDeModo;
    const anteriores = sinAnimacion ? estadosNuevos : previo ? previo.estados : {};
    const desdePct = sinAnimacion
      ? b
        ? Math.round((b.completos / b.total) * 100)
        : 0
      : previo
        ? previo.pct
        : 0;

    const modo = b ? b.modo : '';
    const bloque = crear('section', 'pj-bor' + (b ? '' : ' vacio') + (modo === 'editar' ? ' editando' : modo === 'eliminar' ? ' eliminando' : ''));
    bloque.append(cabecera(b, estadosNuevos, { desde: desdePct }));

    if (modo !== 'eliminar') bloque.append(pasos(estadosNuevos, anteriores, modo === 'editar' ? 'am' : ''));

    if (b) {
      const { cuerpo, completo } = vistaPrevia(b, opciones.imagenNueva);
      if (completo) {
        bloque.append(cuerpo);
      } else {
        const rejilla = crear('div', 'pj-bor-cuerpo');
        const datos = crear('div', 'pj-datos');
        if (modo !== 'eliminar') {
          for (const d of b.datos) datos.append(tarjetaDato(d, foco));
          datos.append(tarjetaImagen(b, opciones.imagenNueva), interruptor(b));
        } else {
          datos.append(crear('p', 'vacio', 'Esta es la ficha que se va a eliminar. Confírmalo con Juno.'));
        }
        rejilla.append(cuerpo, datos);
        bloque.append(rejilla);
      }
      if (modo !== 'eliminar') {
        if (completo) {
          // En la vista «Ficha» las tarjetas de datos no caben: se muestran debajo, en dos columnas.
          const resto = crear('div', 'pj-datos');
          resto.style.cssText = 'display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));margin-top:14px';
          for (const d of b.datos) resto.append(tarjetaDato(d, foco));
          resto.append(tarjetaImagen(b, opciones.imagenNueva), interruptor(b));
          bloque.append(resto);
        }
        bloque.append(pie(b));
      }
    }

    contenedor.replaceChildren(bloque);
    previo = { estados: estadosNuevos, pct: b ? Math.round((b.completos / b.total) * 100) : 0, modo: b ? b.modo : '' };
  }

  return { dibujar };
}
