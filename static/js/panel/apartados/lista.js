// lista.js (apartados): la lista de apartados de la pestaña activa. Los pendientes tienen los botones
// "Entregado" (el cliente recogió y pagó en tienda) y "Cancelar" (el stock vuelve al catálogo).

import { crear } from '../../dom.js';
import { cancelarApartado, entregarApartado } from '../api.js';
import { confirmar } from '../dialogo.js';
import { PENDIENTE, claseEstado, nombrePersona, textoCita } from './formato.js';

function textoUnidades(cantidad) {
  return cantidad === 1 ? '1 unidad' : cantidad + ' unidades';
}

// Hace la acción en el servidor, avisa si falló y vuelve a cargar la lista (haya salido bien o mal).
async function ejecutar(accion, apartado, textoFallo, contexto) {
  try {
    const resultado = await accion(apartado.codigo);
    if (!resultado.ok) contexto.mostrarError(resultado.datos?.mensaje || textoFallo);
  } catch {
    contexto.mostrarError('No hay conexión con el servidor. Inténtalo de nuevo.');
  }
  contexto.recargar();
}

function filaApartado(apartado, contexto) {
  const pendiente = apartado.estado_codigo === PENDIENTE;
  const fila = crear('article', 'fila fila-orden');

  const texto = crear('div', 'fila-texto');
  texto.append(crear('h3', 'fila-titulo', apartado.producto));
  texto.append(
    crear('p', 'fila-descripcion', nombrePersona(apartado.cliente) + ' · ' + apartado.telefono),
  );

  const etiquetas = crear('div', 'etiquetas');
  etiquetas.append(crear('span', claseEstado(apartado), apartado.estado_nombre));
  etiquetas.append(crear('span', 'etiqueta', textoUnidades(apartado.cantidad)));
  etiquetas.append(crear('span', 'etiqueta precio', 'Total S/ ' + apartado.total));
  if (pendiente) etiquetas.append(crear('span', 'etiqueta alerta', 'Vence ' + textoCita(apartado.vence)));
  // Los productos de un mismo pedido (carrito) comparten el código de retiro que se le dio al cliente.
  if (apartado.pedido) etiquetas.append(crear('span', 'etiqueta alerta', 'Código de retiro ' + apartado.pedido));
  etiquetas.append(crear('span', 'etiqueta codigo', apartado.codigo));
  texto.append(etiquetas);
  fila.append(texto);

  if (pendiente) {
    const acciones = crear('div', 'fila-acciones');
    const entregar = crear('button', 'boton-principal boton-corto', 'Entregado');
    entregar.type = 'button';
    entregar.addEventListener('click', async () => {
      const acepto = await confirmar(
        'Confirmar entrega',
        nombrePersona(apartado.cliente) + ' recogió «' + apartado.producto + '» (' + textoUnidades(apartado.cantidad) +
          ') y pagó S/ ' + apartado.total + ' en tienda.',
        'Sí, entregado',
      );
      if (acepto) await ejecutar(entregarApartado, apartado, 'No se pudo confirmar la entrega.', contexto);
    });
    const cancelar = crear('button', 'boton-peligro', 'Cancelar');
    cancelar.type = 'button';
    cancelar.addEventListener('click', async () => {
      const acepto = await confirmar(
        'Cancelar apartado',
        'Se cancelará el apartado de «' + apartado.producto + '» y ' + textoUnidades(apartado.cantidad) +
          ' volverá a estar disponible para otros clientes.',
        'Cancelar apartado',
      );
      if (acepto) await ejecutar(cancelarApartado, apartado, 'No se pudo cancelar el apartado.', contexto);
    });
    acciones.append(entregar, cancelar);
    fila.append(acciones);
  }
  return fila;
}

// Dibuja los apartados ya filtrados dentro de "contenedor". "contexto" lleva { mostrarError, recargar }.
export function dibujarApartados(contenedor, apartados, mensajeVacio, contexto) {
  contenedor.replaceChildren();

  const barra = crear('div', 'lista-barra');
  const titulo = crear('h2', '', 'Apartados');
  titulo.append(crear('small', '', apartados.length === 1 ? '1 apartado' : apartados.length + ' apartados'));
  barra.append(titulo);
  contenedor.append(barra);

  if (apartados.length === 0) {
    contenedor.append(crear('p', 'vacio', mensajeVacio));
    return;
  }
  for (const apartado of apartados) contenedor.append(filaApartado(apartado, contexto));
}
