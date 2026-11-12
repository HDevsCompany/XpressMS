// lista.js: la lista del personal con sus botones de editar, restablecer clave y eliminar.

import { crear } from '../../dom.js';
import { eliminarPersonal, restablecerClavePersonal } from '../api.js';
import { confirmar } from '../dialogo.js';
import { mostrarClaveTemporal } from './clave_temporal.js';
import { abrirFormularioPersonal } from './formulario.js';
import { aTitulo, nombreDocumento, nombreRol } from './roles.js';

function nombreCompleto(persona) {
  return aTitulo([persona.nombres, persona.paterno, persona.materno].filter(Boolean).join(' '));
}

function filaPersona(persona, esUnoMismo, mostrarError, recargar) {
  const fila = crear('article', 'fila');
  const texto = crear('div', 'fila-texto');

  const titulo = crear('h3', 'fila-titulo', nombreCompleto(persona));
  if (esUnoMismo) titulo.append(crear('span', 'tu', 'tú'));
  texto.append(titulo);
  texto.append(
    crear('p', 'fila-descripcion', `${nombreDocumento(persona.tipo_documento)} ${persona.numero_documento} · usuario ${persona.usuario}`),
  );

  const etiquetas = crear('div', 'etiquetas');
  for (const codigo of persona.roles) etiquetas.append(crear('span', 'etiqueta equipo', nombreRol(codigo)));
  if (persona.debe_cambiar_clave) etiquetas.append(crear('span', 'etiqueta alerta', 'Clave temporal'));
  if (persona.bloqueado) etiquetas.append(crear('span', 'etiqueta peligro', 'Bloqueado'));
  etiquetas.append(
    crear('span', 'etiqueta', persona.ultimo_acceso ? 'Último acceso: ' + persona.ultimo_acceso : 'Aún no ingresó'),
  );
  texto.append(etiquetas);

  const acciones = crear('div', 'fila-acciones');
  const editar = crear('button', 'boton-secundario', 'Editar');
  editar.type = 'button';
  editar.addEventListener('click', () => abrirFormularioPersonal(persona, esUnoMismo, recargar));
  acciones.append(editar);

  // Sobre la propia cuenta no se restablece la clave ni se elimina (el servidor también lo impide).
  if (!esUnoMismo) {
    const restablecer = crear('button', 'boton-secundario', 'Restablecer clave');
    restablecer.type = 'button';
    restablecer.addEventListener('click', async () => {
      const acepto = await confirmar(
        'Restablecer clave',
        `Se creará una clave temporal nueva para ${nombreCompleto(persona)}. La clave actual dejará de servir y se cerrarán sus sesiones abiertas.`,
        'Restablecer',
      );
      if (!acepto) return;
      try {
        const resultado = await restablecerClavePersonal(persona.codigo);
        if (resultado.ok) {
          recargar();
          mostrarClaveTemporal('Clave restablecida', resultado.datos);
          return;
        }
        mostrarError(resultado.datos?.mensaje || 'No se pudo restablecer la clave.');
      } catch {
        mostrarError('No hay conexión con el servidor. Inténtalo de nuevo.');
      }
      recargar();
    });

    const eliminar = crear('button', 'boton-peligro', 'Eliminar');
    eliminar.type = 'button';
    eliminar.addEventListener('click', async () => {
      const acepto = await confirmar(
        'Eliminar persona',
        `${nombreCompleto(persona)} dejará de poder ingresar a la plataforma y se cerrarán sus sesiones. Su historial se conserva.`,
        'Eliminar',
      );
      if (!acepto) return;
      try {
        const resultado = await eliminarPersonal(persona.codigo);
        if (!resultado.ok) mostrarError(resultado.datos?.mensaje || 'No se pudo eliminar a la persona.');
      } catch {
        mostrarError('No hay conexión con el servidor. Inténtalo de nuevo.');
      }
      recargar();
    });
    acciones.append(restablecer, eliminar);
  }

  fila.append(texto, acciones);
  return fila;
}

// Dibuja la lista completa dentro de "contenedor". "codigoPropio" es el código del usuario en sesión.
export function dibujarPersonal(contenedor, personal, codigoPropio, mostrarError, recargar) {
  contenedor.replaceChildren();

  const barra = crear('div', 'lista-barra');
  const titulo = crear('h2', '', 'Personal');
  titulo.append(crear('small', '', personal.length === 1 ? '1 activo' : personal.length + ' activos'));
  const nuevo = crear('button', 'boton-principal', 'Nueva persona');
  nuevo.type = 'button';
  nuevo.addEventListener('click', () => abrirFormularioPersonal(null, false, recargar));
  barra.append(titulo, nuevo);
  contenedor.append(barra);

  for (const persona of personal) {
    contenedor.append(filaPersona(persona, persona.codigo === codigoPropio, mostrarError, recargar));
  }
}
