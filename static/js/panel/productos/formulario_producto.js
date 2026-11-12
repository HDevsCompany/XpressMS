// formulario_producto.js: la ventana para crear o editar un producto.
// El stock es la cantidad que hay en tienda (no se suma ni se resta: se escribe el número final).

import { crear } from '../../dom.js';
import { actualizarProducto, crearProducto } from '../api.js';
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

// "producto" es null para crear, o un producto de la lista para editar.
// "categorias" son todas las categorías activas ({ codigo, nombre }).
export function abrirFormularioProducto(producto, categorias, alGuardar) {
  const cuerpo = crear('div', 'dialogo-cuerpo');
  cuerpo.append(crear('h2', '', producto ? 'Editar producto' : 'Nuevo producto'));
  const formulario = crear('form', 'formulario');

  const nombre = campoTexto('Nombre', producto?.nombre ?? '', {
    maxLength: 120, required: true, placeholder: 'Mouse inalámbrico Logitech M170',
  });

  const campoCategoria = crear('label', 'campo');
  campoCategoria.append(crear('span', '', 'Categoría'));
  const categoria = crear('select');
  categoria.required = true;
  for (const c of categorias) {
    const opcion = crear('option', '', c.nombre);
    opcion.value = c.codigo;
    categoria.append(opcion);
  }
  if (producto) categoria.value = producto.categoria_codigo;
  campoCategoria.append(categoria);

  const campoDescripcion = crear('label', 'campo');
  campoDescripcion.append(crear('span', '', 'Descripción (lo que verá el cliente)'));
  const descripcion = crear('textarea');
  descripcion.maxLength = 500;
  descripcion.required = true;
  descripcion.value = producto?.descripcion ?? '';
  campoDescripcion.append(descripcion);

  const precio = campoTexto('Precio (S/)', producto?.precio ?? '', {
    required: true, placeholder: '45.00', inputMode: 'decimal',
  });
  const stock = campoTexto('Unidades disponibles', producto ? String(producto.stock) : '', {
    required: true, placeholder: '10', inputMode: 'numeric',
  });
  const fila = crear('div', 'fila-doble');
  fila.append(precio.campo, stock.campo);

  const mensaje = crear('p', 'mensaje-error');
  mensaje.setAttribute('role', 'alert');
  mensaje.hidden = true;

  const { botones, accion } = crearBotones('Guardar');
  formulario.append(nombre.campo, campoCategoria, campoDescripcion, fila, mensaje, botones);

  formulario.addEventListener('submit', async (evento) => {
    evento.preventDefault();
    mensaje.hidden = true;
    const unidades = stock.entrada.value.trim();
    if (!/^\d{1,6}$/.test(unidades)) {
      mensaje.textContent = 'Las unidades deben ser un número entero, por ejemplo 10.';
      mensaje.hidden = false;
      return;
    }
    accion.disabled = true;
    const datos = {
      categoria: categoria.value,
      nombre: nombre.entrada.value,
      descripcion: descripcion.value,
      precio: precio.entrada.value,
      stock: Number(unidades),
    };
    try {
      const resultado = producto
        ? await actualizarProducto(producto.codigo, datos)
        : await crearProducto(datos);
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
