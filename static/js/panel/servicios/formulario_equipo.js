// formulario_equipo.js: la ventana para crear o renombrar un tipo de equipo.

import { crear } from '../../dom.js';
import { crearEquipo, renombrarEquipo } from '../api.js';
import { abrirDialogo, cerrarDialogo, crearBotones } from '../dialogo.js';

// "equipo" es null para crear uno nuevo, o { codigo, nombre } para renombrar.
// "alGuardar" se ejecuta cuando el servidor aceptó el cambio (la pantalla recarga las listas).
export function abrirFormularioEquipo(equipo, alGuardar) {
  const cuerpo = crear('div', 'dialogo-cuerpo');
  cuerpo.append(crear('h2', '', equipo ? 'Renombrar tipo de equipo' : 'Nuevo tipo de equipo'));

  const formulario = crear('form', 'formulario');
  const campo = crear('label', 'campo');
  campo.append(crear('span', '', 'Nombre'));
  const entrada = crear('input');
  entrada.type = 'text';
  entrada.maxLength = 60;
  entrada.required = true;
  entrada.placeholder = 'Laptop, PC de escritorio, Impresora…';
  entrada.value = equipo ? equipo.nombre : '';
  campo.append(entrada);

  const mensaje = crear('p', 'mensaje-error');
  mensaje.setAttribute('role', 'alert');
  mensaje.hidden = true;

  const { botones, accion } = crearBotones('Guardar');
  formulario.append(campo, mensaje, botones);

  formulario.addEventListener('submit', async (evento) => {
    evento.preventDefault();
    mensaje.hidden = true;
    accion.disabled = true;
    try {
      const resultado = equipo
        ? await renombrarEquipo(equipo.codigo, entrada.value)
        : await crearEquipo(entrada.value);
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
  entrada.focus();
}
