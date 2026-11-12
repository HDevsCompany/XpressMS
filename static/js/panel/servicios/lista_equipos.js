// lista_equipos.js: la pestaña "Tipos de equipo": Laptop, PC de escritorio, Impresora...

import { crear } from '../../dom.js';
import { eliminarEquipo } from '../api.js';
import { confirmar } from '../dialogo.js';
import { abrirFormularioEquipo } from './formulario_equipo.js';

function textoUso(equipo) {
  if (equipo.servicios === 0) return 'Sin servicios';
  return equipo.servicios === 1 ? '1 servicio' : equipo.servicios + ' servicios';
}

function filaEquipo(equipo, mostrarError, recargar) {
  const fila = crear('article', 'fila');
  const texto = crear('div', 'fila-texto');
  texto.append(crear('h3', 'fila-titulo', equipo.nombre));
  const etiquetas = crear('div', 'etiquetas');
  etiquetas.append(crear('span', 'etiqueta', textoUso(equipo)));
  texto.append(etiquetas);

  const acciones = crear('div', 'fila-acciones');
  const renombrar = crear('button', 'boton-secundario', 'Renombrar');
  renombrar.type = 'button';
  renombrar.addEventListener('click', () => abrirFormularioEquipo(equipo, recargar));
  const eliminar = crear('button', 'boton-peligro', 'Eliminar');
  eliminar.type = 'button';
  eliminar.addEventListener('click', async () => {
    const aviso = equipo.servicios > 0
      ? '«' + equipo.nombre + '» dejará de ofrecerse en ' + textoUso(equipo).toLowerCase() + '. Los servicios no se eliminan.'
      : '«' + equipo.nombre + '» dejará de mostrarse a los clientes.';
    if (!(await confirmar('Eliminar tipo de equipo', aviso, 'Eliminar'))) return;
    try {
      const resultado = await eliminarEquipo(equipo.codigo);
      if (!resultado.ok) mostrarError(resultado.datos?.mensaje || 'No se pudo eliminar el tipo de equipo.');
    } catch {
      mostrarError('No hay conexión con el servidor. Inténtalo de nuevo.');
    }
    recargar();
  });
  acciones.append(renombrar, eliminar);

  fila.append(texto, acciones);
  return fila;
}

export function dibujarEquipos(contenedor, equipos, mostrarError, recargar) {
  contenedor.replaceChildren();

  const barra = crear('div', 'lista-barra');
  const titulo = crear('h2', '', 'Tipos de equipo');
  titulo.append(crear('small', '', equipos.length === 1 ? '1 activo' : equipos.length + ' activos'));
  const nuevo = crear('button', 'boton-principal', 'Nuevo tipo de equipo');
  nuevo.type = 'button';
  nuevo.addEventListener('click', () => abrirFormularioEquipo(null, recargar));
  barra.append(titulo, nuevo);
  contenedor.append(barra);

  if (equipos.length === 0) {
    contenedor.append(crear('p', 'vacio', 'Todavía no hay tipos de equipo. Crea el primero con «Nuevo tipo de equipo».'));
    return;
  }
  for (const equipo of equipos) {
    contenedor.append(filaEquipo(equipo, mostrarError, recargar));
  }
}
