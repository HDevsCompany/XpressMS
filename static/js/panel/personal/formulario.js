// formulario.js: la ventana para dar de alta o editar a una persona del personal.

import { crear } from '../../dom.js';
import { actualizarPersonal, crearPersonal } from '../api.js';
import { abrirDialogo, cerrarDialogo, crearBotones } from '../dialogo.js';
import { mostrarClaveTemporal } from './clave_temporal.js';
import { DOCUMENTOS, ROLES, aTitulo, nombreDocumento } from './roles.js';

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

function fila(...campos) {
  const contenedor = crear('div', 'fila-doble');
  contenedor.append(...campos);
  return contenedor;
}

// "persona" es null para dar de alta, o una fila de la lista para editar.
// "esUnoMismo" es true si la persona editada es quien está usando el panel (no puede quitarse el rol de Administrador).
export function abrirFormularioPersonal(persona, esUnoMismo, alGuardar) {
  const editando = persona !== null;
  const cuerpo = crear('div', 'dialogo-cuerpo');
  cuerpo.append(crear('h2', '', editando ? 'Editar persona' : 'Nueva persona'));
  const formulario = crear('form', 'formulario');

  // Documento y usuario: se eligen al dar de alta y después ya no se cambian.
  let tipoDocumento = null;
  let numeroDocumento = null;
  let usuario = null;
  if (editando) {
    const fijos = crear('p', 'datos-fijos');
    fijos.textContent = `${nombreDocumento(persona.tipo_documento)} ${persona.numero_documento} · usuario ${persona.usuario}`;
    formulario.append(fijos);
  } else {
    const campoTipo = crear('label', 'campo');
    campoTipo.append(crear('span', '', 'Documento'));
    tipoDocumento = crear('select');
    for (const doc of DOCUMENTOS) {
      const opcion = crear('option', '', doc.nombre);
      opcion.value = doc.codigo;
      tipoDocumento.append(opcion);
    }
    campoTipo.append(tipoDocumento);
    numeroDocumento = campoTexto('Número', '', { maxLength: 12, required: true, inputMode: 'numeric', placeholder: '12345678' });
    formulario.append(fila(campoTipo, numeroDocumento.campo));
  }

  const paterno = campoTexto('Apellido paterno', editando ? aTitulo(persona.paterno) : '', { maxLength: 100, required: true });
  const materno = campoTexto('Apellido materno', editando ? aTitulo(persona.materno) : '', { maxLength: 100 });
  const nombres = campoTexto('Nombres', editando ? aTitulo(persona.nombres) : '', { maxLength: 100, required: true });
  const telefono = campoTexto('Celular (opcional)', persona?.telefono ?? '', { maxLength: 25, inputMode: 'tel', placeholder: '987654321' });
  const correo = campoTexto('Correo (opcional)', persona?.correo ?? '', { maxLength: 150, inputMode: 'email', placeholder: 'nombre@empresa.com' });
  formulario.append(fila(paterno.campo, materno.campo), nombres.campo, fila(telefono.campo, correo.campo));

  if (!editando) {
    usuario = campoTexto('Nombre de usuario', '', { maxLength: 30, required: true, autocapitalize: 'none', spellcheck: false, placeholder: 'ana.rojas' });
    usuario.campo.append(crear('small', '', 'Con este nombre entrará a la plataforma. No se puede cambiar después.'));
    formulario.append(usuario.campo);
  }

  // Roles: al menos uno.
  const grupo = crear('fieldset', 'grupo-equipos');
  grupo.append(crear('legend', '', 'Roles'));
  const marcados = new Set(persona?.roles ?? []);
  const casillas = ROLES.map((rol) => {
    const opcion = crear('label', 'opcion-equipo');
    const casilla = crear('input');
    casilla.type = 'checkbox';
    casilla.value = rol.codigo;
    casilla.checked = marcados.has(rol.codigo);
    // Quien edita su propia cuenta no puede quitarse el rol de Administrador.
    if (esUnoMismo && rol.codigo === 'ADM') casilla.disabled = true;
    const texto = crear('span');
    texto.append(crear('strong', '', rol.nombre), document.createTextNode(' · ' + rol.descripcion));
    opcion.append(casilla, texto);
    grupo.append(opcion);
    return casilla;
  });
  formulario.append(grupo);

  const mensaje = crear('p', 'mensaje-error');
  mensaje.setAttribute('role', 'alert');
  mensaje.hidden = true;
  const { botones, accion } = crearBotones(editando ? 'Guardar' : 'Crear cuenta');
  formulario.append(mensaje, botones);

  formulario.addEventListener('submit', async (evento) => {
    evento.preventDefault();
    mensaje.hidden = true;
    accion.disabled = true;
    const comunes = {
      paterno: paterno.entrada.value,
      materno: materno.entrada.value,
      nombres: nombres.entrada.value,
      telefono: telefono.entrada.value,
      correo: correo.entrada.value,
      roles: casillas.filter((c) => c.checked).map((c) => c.value),
    };
    try {
      const resultado = editando
        ? await actualizarPersonal(persona.codigo, comunes)
        : await crearPersonal({
            ...comunes,
            tipo_documento: tipoDocumento.value,
            numero_documento: numeroDocumento.entrada.value,
            usuario: usuario.entrada.value,
          });
      if (resultado.ok) {
        cerrarDialogo();
        alGuardar();
        // Al dar de alta, el servidor devuelve la clave temporal: se muestra una sola vez.
        if (!editando && resultado.datos) mostrarClaveTemporal('Cuenta creada', resultado.datos);
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
  (editando ? paterno : numeroDocumento).entrada.focus();
}
