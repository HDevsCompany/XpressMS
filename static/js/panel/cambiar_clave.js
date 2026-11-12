// cambiar_clave.js: el formulario para cambiar la clave. Lo usa la página de inicio del panel,
// tanto cuando la clave es temporal (obligatorio) como cuando la persona quiere cambiarla por gusto.

import { crear } from '../dom.js';
import { cambiarClave } from './api.js';

// Crea el formulario. "alTerminar" se ejecuta cuando la clave se cambió con éxito.
export function crearFormularioClave(alTerminar) {
  const formulario = crear('form', 'formulario');
  formulario.autocomplete = 'off';

  const campo = (etiqueta, nombre, autocompletado, ayuda = '') => {
    const contenedor = crear('label', 'campo');
    contenedor.append(crear('span', '', etiqueta));
    const entrada = crear('input');
    entrada.type = 'password';
    entrada.name = nombre;
    entrada.required = true;
    entrada.autocomplete = autocompletado;
    contenedor.append(entrada);
    if (ayuda) contenedor.append(crear('small', '', ayuda));
    formulario.append(contenedor);
    return entrada;
  };

  const actual = campo('Clave actual', 'actual', 'current-password');
  const nueva = campo('Clave nueva', 'nueva', 'new-password', 'Mínimo 10 caracteres, con al menos una letra y un número.');
  const repetida = campo('Repite la clave nueva', 'repetida', 'new-password');

  const mensaje = crear('p', 'mensaje-error');
  mensaje.setAttribute('role', 'alert');
  mensaje.hidden = true;
  formulario.append(mensaje);

  const boton = crear('button', 'boton-principal', 'Guardar clave nueva');
  boton.type = 'submit';
  formulario.append(boton);

  function mostrar(texto, esExito = false) {
    mensaje.textContent = texto;
    mensaje.className = esExito ? 'mensaje-exito' : 'mensaje-error';
    mensaje.hidden = false;
  }

  formulario.addEventListener('submit', async (evento) => {
    evento.preventDefault();
    mensaje.hidden = true;
    if (nueva.value !== repetida.value) {
      mostrar('Las dos claves nuevas no coinciden.');
      return;
    }
    boton.disabled = true;
    try {
      const resultado = await cambiarClave(actual.value, nueva.value);
      if (resultado.ok) {
        formulario.reset();
        mostrar('Tu clave se cambió correctamente.', true);
        alTerminar();
        return;
      }
      mostrar(resultado.datos?.mensaje || 'No se pudo cambiar la clave.');
    } catch {
      mostrar('No hay conexión con el servidor. Inténtalo de nuevo.');
    }
    boton.disabled = false;
  });

  return formulario;
}
