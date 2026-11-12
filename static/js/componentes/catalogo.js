// catalogo.js: la vitrina pública de productos.
// Esta pantalla SOLO MUESTRA: no tiene buscador, filtros ni selectores. Quien busca y filtra es Juno (el servidor decide
// qué productos van y este archivo los dibuja). Cada tarjeta tiene dos botones: «Añadir al carrito», que no ejecuta nada
// (le pide a Juno que lo haga, igual que si la persona lo hubiera escrito), y «Más info», que abre la ficha del producto.

import { crear } from '../dom.js';
import { dibujoDeProducto } from './catalogo_arte.js';
import { existencias, iconoDe, iconoStock, partirPrecio, tonoDe, varianteDe } from './catalogo_comun.js';
import { abrirFicha } from './ficha.js';

const FOTO_JUNO = '/static/img/juno.png';

// Cierra los rótulos que se abrieron con un toque (en celular no existe «pasar el cursor»).
function cerrarAvisos(excepto) {
  document.querySelectorAll('.lz-prod-stock.abierto').forEach((otro) => {
    if (otro !== excepto) {
      otro.classList.remove('abierto');
      otro.setAttribute('aria-expanded', 'false');
    }
  });
}
document.addEventListener('click', () => cerrarAvisos());

// Arma el rótulo de existencias: solo el ícono (con el número de unidades en «pocas unidades») y, al pasar el cursor,
// enfocarlo con el teclado o tocarlo, se despliega el texto.
function armarRotulo(rotulo, stock, unidades) {
  const texto = crear('span', 'lz-prod-texto', stock.texto);
  rotulo.append(iconoStock(stock.clase));
  if (stock.clase === 'poco') {
    const numero = crear('b', 'lz-prod-num', String(unidades));
    numero.setAttribute('aria-hidden', 'true');
    rotulo.append(numero);
  }
  rotulo.append(texto);
  rotulo.tabIndex = 0;
  rotulo.setAttribute('role', 'button');
  rotulo.setAttribute('aria-expanded', 'false');
  let temporizador = 0;
  rotulo.addEventListener('click', (evento) => {
    evento.stopPropagation(); // que no lo cierre el clic general
    cerrarAvisos(rotulo);
    const abrir = !rotulo.classList.contains('abierto');
    rotulo.classList.toggle('abierto', abrir);
    rotulo.setAttribute('aria-expanded', String(abrir));
    clearTimeout(temporizador);
    if (abrir) temporizador = setTimeout(() => cerrarAvisos(), 4000);
  });
}

const CHECK = 'M5 12.5l4.5 4.5L19 7.5';

// El botón principal de la tarjeta: «Añadir al carrito» o, si ya está en el carrito, «En tu carrito» (que abre el carrito).
function pintarBoton(boton, cantidad) {
  boton.replaceChildren();
  boton.classList.toggle('en', cantidad > 0);
  boton.dataset.modo = cantidad > 0 ? 'ver' : 'anadir';
  boton.dataset.cantidad = String(cantidad);
  if (cantidad > 0) {
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('viewBox', '0 0 24 24');
    svg.setAttribute('aria-hidden', 'true');
    const trazo = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    trazo.setAttribute('d', CHECK);
    svg.appendChild(trazo);
    boton.append(svg, cantidad > 1 ? `En tu carrito · ${cantidad}` : 'En tu carrito');
  } else {
    const juno = crear('img');
    juno.src = FOTO_JUNO;
    juno.alt = '';
    boton.append(juno, 'Añadir al carrito');
  }
}

// Pone cada botón «Añadir al carrito» en su estado según lo que hay en el carrito (cantidades por código de producto).
export function marcarEnCarrito(cantidades) {
  document.querySelectorAll('.lz-prod[data-id]').forEach((tarjetaProducto) => {
    const boton = tarjetaProducto.querySelector('.lz-prod-boton[data-modo]');
    if (!boton) return;
    const cantidad = cantidades.get(tarjetaProducto.dataset.id) || 0;
    if (boton.dataset.cantidad !== String(cantidad)) pintarBoton(boton, cantidad);
  });
}

const ICONO_INFO = ['M12 3.5a8.5 8.5 0 100 17 8.5 8.5 0 000-17z', 'M12 11v5', 'M12 8h.01'];

function tarjeta(producto, acciones) {
  const [fuerte, medio, claro] = tonoDe(producto.categoria);
  const agotado = producto.unidades <= 0;
  const articulo = crear('article', agotado ? 'lz-prod agotado' : 'lz-prod');
  articulo.dataset.id = producto.id;

  // La foto lleva la categoría, las existencias, el dibujo del producto y el precio en una pestaña abajo a la izquierda.
  const foto = crear('div', 'lz-prod-foto');
  foto.style.background = `linear-gradient(160deg, ${fuerte}, ${medio} 60%, ${claro})`;

  const stock = existencias(producto.unidades);
  const rotulo = crear('span', `lz-prod-stock ${stock.clase}`);
  armarRotulo(rotulo, stock, producto.unidades);

  const precio = partirPrecio(producto.precio);
  const lineaPrecio = crear('p', 'lz-prod-precio');
  lineaPrecio.append(crear('small', '', 'S/'), precio.entero);
  if (precio.decimales) lineaPrecio.append(crear('small', 'dec', precio.decimales));

  foto.append(
    crear('span', 'lz-prod-cat', producto.categoria),
    rotulo,
    dibujoDeProducto(iconoDe(producto), varianteDe(producto.nombre)),
    lineaPrecio,
  );
  // Tocar la foto abre la ficha (igual que «Más info»); el rótulo de existencias tiene su propio toque.
  foto.addEventListener('click', (evento) => {
    if (!evento.target.closest('.lz-prod-stock')) abrirFicha(producto, acciones, articulo);
  });

  const boton = crear('button', 'lz-prod-boton');
  boton.type = 'button';
  if (!agotado) {
    pintarBoton(boton, acciones.cantidadEnCarrito(producto.id));
    // No añade nada: le pide a Juno que lo haga (o abre el carrito, si ya está dentro).
    boton.addEventListener('click', () => {
      if (boton.dataset.modo === 'ver') {
        acciones.verCarrito();
        return;
      }
      document.querySelectorAll('.lz-prod.elegido').forEach((otra) => otra.classList.remove('elegido'));
      articulo.classList.add('elegido');
      acciones.anadir(producto);
    });
  } else {
    boton.classList.add('suave');
    boton.textContent = 'Agotado por ahora';
    boton.disabled = true;
  }

  const mas = crear('button', 'lz-prod-mas');
  mas.type = 'button';
  const info = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  info.setAttribute('viewBox', '0 0 24 24');
  info.setAttribute('aria-hidden', 'true');
  for (const trazo of ICONO_INFO) {
    const camino = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    camino.setAttribute('d', trazo);
    info.appendChild(camino);
  }
  mas.append(info, 'Más info');
  mas.setAttribute('aria-label', `Más información de ${producto.nombre}`);
  mas.addEventListener('click', () => abrirFicha(producto, acciones, articulo));

  const botones = crear('div', 'lz-prod-botones');
  botones.append(boton, mas);

  const cuerpo = crear('div', 'lz-prod-cuerpo');
  cuerpo.append(crear('h3', '', producto.nombre), crear('p', 'lz-prod-detalle', producto.descripcion), botones);
  articulo.append(foto, cuerpo);
  return articulo;
}

// La franja de información: solo lectura. Dice qué filtró Juno y cuántos productos se ven.
function describir(datos) {
  const visibles = datos.productos.length;
  const cantidad = visibles === 1 ? '1 producto' : `${visibles} productos`;
  if (datos.filtros.length === 0) {
    const fragmento = document.createDocumentFragment();
    fragmento.append('Mostrando ', crear('b', '', 'todo el catálogo'), ` · ${cantidad}`);
    return fragmento;
  }
  const fragmento = document.createDocumentFragment();
  fragmento.append('Juno filtró: ', crear('span', 'lz-filtro', datos.filtros.join(' · ')), ` · ${visibles} de ${datos.total} productos`);
  return fragmento;
}

let numeroDeCambio = 0; // para que, si Juno cambia la vitrina dos veces seguidas, gane siempre la última

// Dibuja la vitrina con lo que envió el servidor: { filtros, total, productos }.
// Cuando Juno la cambia (un filtro, «Ver todo»), la cuadrícula anterior se desvanece y la nueva entra tarjeta por tarjeta.
// "acciones" (lo arma tienda.js): anadir(producto) y preguntar(producto) le piden algo a Juno; verCarrito() abre el carrito;
// cantidadEnCarrito(id) dice cuántas unidades de ese producto hay ya en el carrito.
export function mostrarCatalogo(datos, acciones) {
  const grilla = document.getElementById('catalogo-grid');
  const estado = document.getElementById('catalogo-estado');
  const miCambio = ++numeroDeCambio;

  estado.classList.remove('cambio');
  void estado.offsetWidth; // reinicia el destello de la franja
  estado.classList.add('cambio');
  document.getElementById('catalogo-estado-texto').replaceChildren(describir(datos));

  const dibujar = () => {
    if (miCambio !== numeroDeCambio) return;
    const tarjetas = datos.productos.map((producto, posicion) => {
      const nueva = tarjeta(producto, acciones);
      nueva.style.animationDelay = `${Math.min(posicion, 11) * 35}ms`; // entran en cascada
      return nueva;
    });
    grilla.replaceChildren(...tarjetas);
    grilla.classList.remove('saliendo', 'nueva');
    void grilla.offsetWidth; // reinicia la animación de entrada
    grilla.classList.add('nueva');
    document.getElementById('catalogo').scrollTo({ top: 0, behavior: 'smooth' });
  };

  const sinMovimiento = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (grilla.children.length === 0 || sinMovimiento) {
    dibujar();
  } else {
    grilla.classList.add('saliendo');
    setTimeout(dibujar, 170);
  }
}
