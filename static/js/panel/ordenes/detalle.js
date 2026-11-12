// detalle.js (órdenes): la ventana con todo lo de una orden. Desde aquí el Administrador asigna el técnico
// y el personal cambia el estado. Las reglas (qué estados siguen, quién puede) las decide el servidor:
// esta ventana solo muestra los botones que el servidor dice que corresponden.

import { crear } from '../../dom.js';
import { asignarTecnico, cambiarEstadoOrden, detalleOrden, listarTecnicos } from '../api.js';
import { abrirDialogo, cerrarDialogo, confirmar } from '../dialogo.js';
import { nombrePersona, textoCita } from './formato.js';

const CANCELADA = 'CAN';

// Un dato de la ventana: etiqueta pequeña arriba y el valor debajo.
function dato(etiqueta, valor) {
  const bloque = crear('div', 'orden-dato-panel');
  bloque.append(crear('span', '', etiqueta), crear('strong', '', valor));
  return bloque;
}

function mensajeDe(resultado, porDefecto) {
  return resultado.datos?.mensaje || porDefecto;
}

// Abre el detalle de la orden "codigo". "contexto" = { esAdmin, mostrarError, recargar }.
export async function abrirDetalleOrden(codigo, contexto) {
  let resultado;
  try {
    resultado = await detalleOrden(codigo);
  } catch {
    contexto.mostrarError('No hay conexión con el servidor. Inténtalo de nuevo.');
    return;
  }
  if (!resultado.ok) {
    contexto.mostrarError(mensajeDe(resultado, 'No se pudo abrir la orden.'));
    contexto.recargar();
    return;
  }

  const { orden, historial, siguientes } = resultado.datos;
  const terminada = siguientes.length === 0;
  const cuerpo = crear('div', 'dialogo-cuerpo');

  // --- Encabezado: servicio, estado y código ---
  const encabezado = crear('div', 'orden-encabezado');
  encabezado.append(crear('h2', '', orden.servicio));
  const etiquetas = crear('div', 'etiquetas');
  etiquetas.append(crear('span', 'etiqueta equipo', orden.estado_nombre));
  etiquetas.append(crear('span', 'etiqueta codigo', orden.codigo));
  encabezado.append(etiquetas);
  cuerpo.append(encabezado);

  // --- Datos de la orden ---
  const datos = crear('div', 'orden-datos-panel');
  datos.append(dato('Cliente', nombrePersona(orden.cliente)), dato('Celular', orden.telefono));
  datos.append(dato('Cita', textoCita(orden.fecha_cita)));
  if (orden.equipo) datos.append(dato('Equipo', orden.equipo));
  cuerpo.append(datos);
  const problema = crear('div', 'orden-problema');
  problema.append(crear('span', '', 'Problema que cuenta el cliente'), crear('p', '', orden.problema));
  cuerpo.append(problema);

  const mensaje = crear('p', 'mensaje-error');
  mensaje.setAttribute('role', 'alert');
  mensaje.hidden = true;
  const avisar = (texto) => {
    mensaje.textContent = texto;
    mensaje.hidden = false;
  };

  // --- Técnico: el Administrador lo asigna; el Técnico solo lo ve ---
  const bloqueTecnico = crear('section', 'orden-seccion');
  bloqueTecnico.append(crear('h3', '', 'Técnico'));
  if (contexto.esAdmin && !terminada) {
    await dibujarAsignacion(bloqueTecnico, orden, contexto, avisar);
  } else {
    bloqueTecnico.append(
      crear('p', '', orden.tecnico_nombre ? nombrePersona(orden.tecnico_nombre) : 'Sin técnico asignado'),
    );
  }
  cuerpo.append(bloqueTecnico);

  // --- Cambiar el estado ---
  if (!terminada) {
    cuerpo.append(dibujarCambioDeEstado(orden, siguientes, contexto, avisar));
  }
  cuerpo.append(mensaje);

  // --- Historial ---
  const bloqueHistorial = crear('section', 'orden-seccion');
  bloqueHistorial.append(crear('h3', '', 'Historial'));
  const lista = crear('ol', 'orden-historial');
  for (const registro of historial) {
    const item = crear('li', '');
    const linea = crear('div', 'historial-linea');
    linea.append(crear('strong', '', registro.estado), crear('span', '', registro.fecha));
    item.append(linea);
    const quien = registro.usuario ? 'Por ' + nombrePersona(registro.usuario) : 'Por el cliente';
    item.append(crear('small', '', quien));
    if (registro.nota) item.append(crear('p', 'historial-nota', registro.nota));
    lista.append(item);
  }
  bloqueHistorial.append(lista);
  cuerpo.append(bloqueHistorial);

  // --- Pie ---
  const pie = crear('div', 'dialogo-botones');
  const cerrar = crear('button', 'boton-secundario', 'Cerrar');
  cerrar.type = 'button';
  cerrar.addEventListener('click', cerrarDialogo);
  pie.append(cerrar);
  cuerpo.append(pie);

  abrirDialogo(cuerpo);
}

// El selector de técnico con su botón (solo Administrador).
async function dibujarAsignacion(bloque, orden, contexto, avisar) {
  let tecnicos = [];
  try {
    const resultado = await listarTecnicos();
    if (resultado.ok) tecnicos = resultado.datos;
  } catch {
    // sin la lista, el selector queda con las opciones que ya conocemos
  }

  const fila = crear('div', 'orden-asignar');
  const selector = crear('select');
  selector.setAttribute('aria-label', 'Técnico de la orden');
  const vacio = crear('option', '', 'Sin técnico');
  vacio.value = '';
  selector.append(vacio);
  for (const tecnico of tecnicos) {
    const opcion = crear('option', '', nombrePersona(tecnico.nombre));
    opcion.value = tecnico.codigo;
    selector.append(opcion);
  }
  // Si el técnico actual ya no está disponible (por ejemplo, fue dado de baja), se muestra igual.
  if (orden.tecnico_codigo && !tecnicos.some((t) => t.codigo === orden.tecnico_codigo)) {
    const actual = crear('option', '', nombrePersona(orden.tecnico_nombre || orden.tecnico_codigo) + ' (no disponible)');
    actual.value = orden.tecnico_codigo;
    selector.append(actual);
  }
  selector.value = orden.tecnico_codigo || '';

  const guardar = crear('button', 'boton-principal', 'Guardar técnico');
  guardar.type = 'button';
  guardar.addEventListener('click', async () => {
    guardar.disabled = true;
    try {
      const resultado = await asignarTecnico(orden.codigo, selector.value || null);
      if (resultado.ok) {
        cerrarDialogo();
        contexto.recargar();
        return;
      }
      avisar(mensajeDe(resultado, 'No se pudo asignar el técnico.'));
    } catch {
      avisar('No hay conexión con el servidor. Inténtalo de nuevo.');
    }
    guardar.disabled = false;
  });

  fila.append(selector, guardar);
  bloque.append(fila);
  if (tecnicos.length === 0) {
    bloque.append(
      crear('p', 'orden-ayuda', 'No hay técnicos disponibles. Da de alta a alguien con el rol Técnico en «Personal y usuarios».'),
    );
  }
}

// La nota opcional y un botón por cada estado al que la orden puede pasar.
function dibujarCambioDeEstado(orden, siguientes, contexto, avisar) {
  const bloque = crear('section', 'orden-seccion');
  bloque.append(crear('h3', '', 'Cambiar estado'));

  const campo = crear('label', 'campo');
  campo.append(crear('span', '', 'Nota (opcional)'));
  const nota = crear('textarea');
  nota.rows = 2;
  nota.maxLength = 300;
  nota.placeholder = 'Por ejemplo: se cambió la pasta térmica, falta confirmar el repuesto…';
  campo.append(nota);
  bloque.append(campo);

  const botones = crear('div', 'orden-estados');
  const todos = [];
  for (const estado of siguientes) {
    const esCancelar = estado.codigo === CANCELADA;
    const boton = crear('button', esCancelar ? 'boton-peligro' : 'boton-principal', esCancelar ? 'Cancelar orden' : 'Pasar a «' + estado.nombre + '»');
    boton.type = 'button';
    boton.addEventListener('click', async () => {
      if (esCancelar) {
        const acepto = await confirmar(
          'Cancelar orden',
          'La orden ' + orden.codigo + ' quedará cancelada y ya no podrá cambiarse.',
          'Cancelar orden',
        );
        if (!acepto) {
          abrirDetalleOrden(orden.codigo, contexto); // confirmar() cerró la ventana: se vuelve a abrir
          return;
        }
      }
      todos.forEach((b) => (b.disabled = true));
      try {
        const resultado = await cambiarEstadoOrden(orden.codigo, estado.codigo, nota.value);
        if (resultado.ok) {
          cerrarDialogo();
          contexto.recargar();
          return;
        }
        const texto = mensajeDe(resultado, 'No se pudo cambiar el estado.');
        if (esCancelar) {
          // La ventana de confirmación ya cerró esta ventana: el aviso va en la página.
          contexto.mostrarError(texto);
          contexto.recargar();
          return;
        }
        avisar(texto);
        // Si la orden cambió mientras se miraba (409), la lista de fondo se actualiza.
        if (resultado.estado === 409) contexto.recargar();
      } catch {
        const texto = 'No hay conexión con el servidor. Inténtalo de nuevo.';
        if (esCancelar) {
          contexto.mostrarError(texto);
          return;
        }
        avisar(texto);
      }
      todos.forEach((b) => (b.disabled = false));
    });
    todos.push(boton);
    botones.append(boton);
  }
  bloque.append(botones);
  return bloque;
}
