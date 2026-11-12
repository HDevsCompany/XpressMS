// formulario_servicio.js: la ventana para crear o editar un servicio.

import { crear } from '../../dom.js';
import { actualizarServicio, crearServicio } from '../api.js';
import { abrirDialogo, cerrarDialogo, crearBotones } from '../dialogo.js';

// Un campo de una línea con su etiqueta. Devuelve { campo, entrada }.
function campoTexto(etiqueta, valor, opciones = {}) {
  const campo = crear('label', 'campo');
  campo.append(crear('span', '', etiqueta));
  const entrada = crear('input');
  entrada.type = 'text';
  entrada.value = valor;
  Object.assign(entrada, opciones);
  campo.append(entrada);
  return { campo, entrada };
}

// "servicio" es null para crear, o un servicio de la lista para editar.
// "equipos" son todos los tipos de equipo disponibles ({ codigo, nombre }).
export function abrirFormularioServicio(servicio, equipos, alGuardar) {
  const cuerpo = crear('div', 'dialogo-cuerpo');
  cuerpo.append(crear('h2', '', servicio ? 'Editar servicio' : 'Nuevo servicio'));
  const formulario = crear('form', 'formulario');

  const nombre = campoTexto('Nombre', servicio?.nombre ?? '', { maxLength: 100, required: true, placeholder: 'Mantenimiento preventivo' });

  const campoDescripcion = crear('label', 'campo');
  campoDescripcion.append(crear('span', '', 'Descripción (lo que verá el cliente)'));
  const descripcion = crear('textarea');
  descripcion.maxLength = 500;
  descripcion.required = true;
  descripcion.value = servicio?.descripcion ?? '';
  campoDescripcion.append(descripcion);

  const precio = campoTexto('Precio base (S/)', servicio?.precio_base ?? '', { placeholder: '60.00', inputMode: 'decimal' });
  precio.campo.append(crear('small', '', 'Déjalo vacío si el servicio se cotiza.'));
  const tiempo = campoTexto('Tiempo estimado', servicio?.tiempo_estimado ?? '', { maxLength: 40, placeholder: 'Mismo día, 2 a 3 días…' });
  const fila = crear('div', 'fila-doble');
  fila.append(precio.campo, tiempo.campo);

  // Casillas de los tipos de equipo para los que sirve el servicio.
  const grupo = crear('fieldset', 'grupo-equipos');
  grupo.append(crear('legend', '', 'Sirve para'));
  const marcados = new Set((servicio?.equipos ?? []).map((e) => e.codigo));
  const casillas = equipos.map((equipo) => {
    const opcion = crear('label', 'opcion-equipo');
    const casilla = crear('input');
    casilla.type = 'checkbox';
    casilla.value = equipo.codigo;
    casilla.checked = marcados.has(equipo.codigo);
    opcion.append(casilla, crear('span', '', equipo.nombre));
    grupo.append(opcion);
    return casilla;
  });
  if (equipos.length === 0) {
    grupo.append(crear('small', '', 'Primero crea al menos un tipo de equipo en la pestaña «Tipos de equipo».'));
  }

  const mensaje = crear('p', 'mensaje-error');
  mensaje.setAttribute('role', 'alert');
  mensaje.hidden = true;

  const { botones, accion } = crearBotones('Guardar');
  accion.disabled = equipos.length === 0;
  formulario.append(nombre.campo, campoDescripcion, fila, grupo, mensaje, botones);

  formulario.addEventListener('submit', async (evento) => {
    evento.preventDefault();
    mensaje.hidden = true;
    accion.disabled = true;
    const datos = {
      nombre: nombre.entrada.value,
      descripcion: descripcion.value,
      precio_base: precio.entrada.value.trim() || null,
      tiempo_estimado: tiempo.entrada.value.trim() || null,
      equipos: casillas.filter((c) => c.checked).map((c) => c.value),
    };
    try {
      const resultado = servicio
        ? await actualizarServicio(servicio.codigo, datos)
        : await crearServicio(datos);
      if (resultado.ok) {
        cerrarDialogo();
        alGuardar();
        return;
      }
      mensaje.textContent = resultado.datos?.mensaje || 'No se pudo guardar.';
    } catch {
      mensaje.textContent = 'No hay conexión con el servidor. Inténtalo de nuevo.';
    }
    mensaje.hidden = false;
    accion.disabled = false;
  });

  cuerpo.append(formulario);
  abrirDialogo(cuerpo);
  nombre.entrada.focus();
}
