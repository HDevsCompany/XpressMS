// lista_productos.js: la pestaña "Productos": la lista con sus botones de crear, editar y eliminar.
// Un filtro por categoría ayuda cuando el catálogo crece.

import { crear } from '../../dom.js';
import { eliminarProducto } from '../api.js';
import { confirmar } from '../dialogo.js';
import { abrirFormularioProducto } from './formulario_producto.js';

// Qué etiqueta lleva el stock: sin unidades se avisa, con pocas se advierte.
function etiquetaStock(producto) {
  if (producto.stock === 0) return crear('span', 'etiqueta peligro', 'Agotado');
  const texto = producto.stock === 1 ? '1 unidad' : producto.stock + ' unidades';
  return crear('span', producto.stock <= 3 ? 'etiqueta alerta' : 'etiqueta equipo', texto);
}

function filaProducto(producto, categorias, mostrarError, recargar) {
  const fila = crear('article', 'fila');
  const texto = crear('div', 'fila-texto');
  texto.append(crear('h3', 'fila-titulo', producto.nombre), crear('p', 'fila-descripcion', producto.descripcion));

  const etiquetas = crear('div', 'etiquetas');
  etiquetas.append(crear('span', 'etiqueta precio', 'S/ ' + producto.precio));
  etiquetas.append(etiquetaStock(producto));
  etiquetas.append(crear('span', 'etiqueta', producto.categoria_nombre));
  texto.append(etiquetas);

  const acciones = crear('div', 'fila-acciones');
  const editar = crear('button', 'boton-secundario', 'Editar');
  editar.type = 'button';
  editar.addEventListener('click', () => abrirFormularioProducto(producto, categorias, recargar));
  const eliminar = crear('button', 'boton-peligro', 'Eliminar');
  eliminar.type = 'button';
  eliminar.addEventListener('click', async () => {
    const acepto = await confirmar(
      'Eliminar producto',
      '«' + producto.nombre + '» dejará de mostrarse a los clientes en el asistente.',
      'Eliminar',
    );
    if (!acepto) return;
    try {
      const resultado = await eliminarProducto(producto.codigo);
      if (!resultado.ok) mostrarError(resultado.datos?.mensaje || 'No se pudo eliminar el producto.');
    } catch {
      mostrarError('No hay conexión con el servidor. Inténtalo de nuevo.');
    }
    recargar();
  });
  acciones.append(editar, eliminar);

  fila.append(texto, acciones);
  return fila;
}

// La categoría elegida en el filtro ('' = todas). Se recuerda para que no se pierda al recargar la lista.
let filtroActual = '';

// Dibuja la pestaña completa dentro de "contenedor".
export function dibujarProductos(contenedor, productos, categorias, mostrarError, recargar) {
  contenedor.replaceChildren();
  let filtro = filtroActual;
  // Si la categoría filtrada ya no existe (se eliminó), se vuelve a mostrar todo.
  if (filtro && !categorias.some((c) => c.codigo === filtro)) filtro = '';
  const visibles = filtro ? productos.filter((p) => p.categoria_codigo === filtro) : productos;

  const barra = crear('div', 'lista-barra');
  const titulo = crear('h2', '', 'Productos');
  titulo.append(crear('small', '', visibles.length === 1 ? '1 producto' : visibles.length + ' productos'));
  const nuevo = crear('button', 'boton-principal', 'Nuevo producto');
  nuevo.type = 'button';
  nuevo.addEventListener('click', () => abrirFormularioProducto(null, categorias, recargar));
  barra.append(titulo, nuevo);
  contenedor.append(barra);

  if (categorias.length === 0) {
    contenedor.append(crear('p', 'vacio', 'Primero crea al menos una categoría en la pestaña «Categorías».'));
    return;
  }

  // Filtro por categoría (solo si hay productos que filtrar).
  if (productos.length > 0) {
    const caja = crear('label', 'filtro-categoria');
    caja.append(crear('span', '', 'Categoría'));
    const selector = crear('select');
    const todas = crear('option', '', 'Todas');
    todas.value = '';
    selector.append(todas);
    for (const categoria of categorias) {
      const opcion = crear('option', '', categoria.nombre);
      opcion.value = categoria.codigo;
      selector.append(opcion);
    }
    selector.value = filtro;
    selector.addEventListener('change', () => {
      filtroActual = selector.value;
      dibujarProductos(contenedor, productos, categorias, mostrarError, recargar);
    });
    caja.append(selector);
    contenedor.append(caja);
  }

  if (visibles.length === 0) {
    const texto = productos.length === 0
      ? 'Todavía no hay productos. Crea el primero con «Nuevo producto».'
      : 'No hay productos en esta categoría.';
    contenedor.append(crear('p', 'vacio', texto));
    return;
  }
  for (const producto of visibles) {
    contenedor.append(filaProducto(producto, categorias, mostrarError, recargar));
  }
}
