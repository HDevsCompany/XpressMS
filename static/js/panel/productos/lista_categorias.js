// lista_categorias.js: la pestaña "Categorías": Laptops, Periféricos, Almacenamiento...

import { crear } from '../../dom.js';
import { eliminarCategoria } from '../api.js';
import { confirmar } from '../dialogo.js';
import { abrirFormularioCategoria } from './formulario_categoria.js';

function textoUso(categoria) {
  if (categoria.productos === 0) return 'Sin productos';
  return categoria.productos === 1 ? '1 producto' : categoria.productos + ' productos';
}

function filaCategoria(categoria, mostrarError, recargar) {
  const fila = crear('article', 'fila');
  const texto = crear('div', 'fila-texto');
  texto.append(crear('h3', 'fila-titulo', categoria.nombre));
  const etiquetas = crear('div', 'etiquetas');
  etiquetas.append(crear('span', 'etiqueta', textoUso(categoria)));
  texto.append(etiquetas);

  const acciones = crear('div', 'fila-acciones');
  const renombrar = crear('button', 'boton-secundario', 'Renombrar');
  renombrar.type = 'button';
  renombrar.addEventListener('click', () => abrirFormularioCategoria(categoria, recargar));
  const eliminar = crear('button', 'boton-peligro', 'Eliminar');
  eliminar.type = 'button';
  eliminar.addEventListener('click', async () => {
    const aviso = '«' + categoria.nombre + '» dejará de mostrarse a los clientes.';
    if (!(await confirmar('Eliminar categoría', aviso, 'Eliminar'))) return;
    try {
      const resultado = await eliminarCategoria(categoria.codigo);
      if (!resultado.ok) mostrarError(resultado.datos?.mensaje || 'No se pudo eliminar la categoría.');
    } catch {
      mostrarError('No hay conexión con el servidor. Inténtalo de nuevo.');
    }
    recargar();
  });
  acciones.append(renombrar, eliminar);

  fila.append(texto, acciones);
  return fila;
}

export function dibujarCategorias(contenedor, categorias, mostrarError, recargar) {
  contenedor.replaceChildren();

  const barra = crear('div', 'lista-barra');
  const titulo = crear('h2', '', 'Categorías');
  titulo.append(crear('small', '', categorias.length === 1 ? '1 activa' : categorias.length + ' activas'));
  const nueva = crear('button', 'boton-principal', 'Nueva categoría');
  nueva.type = 'button';
  nueva.addEventListener('click', () => abrirFormularioCategoria(null, recargar));
  barra.append(titulo, nueva);
  contenedor.append(barra);

  if (categorias.length === 0) {
    contenedor.append(crear('p', 'vacio', 'Todavía no hay categorías. Crea la primera con «Nueva categoría».'));
    return;
  }
  for (const categoria of categorias) {
    contenedor.append(filaCategoria(categoria, mostrarError, recargar));
  }
}
