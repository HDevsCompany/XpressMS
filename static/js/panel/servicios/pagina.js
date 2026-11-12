// pagina.js: la página /panel/servicios. Junta las piezas: la lista de servicios, Juno (el chat que arma, edita y elimina), la ficha en
// construcción y la pestaña de tipos de equipo. La interfaz muestra, Juno ejecuta: aquí nada se guarda sin pasar por Juno.

import { listarEquipos, listarServicios, quienSoy, quitarImagenServicio, salir, subirImagenServicio } from '../api.js';
import { crear } from '../../dom.js';
import { crearBorrador } from './borrador.js';
import { elegirImagen } from './imagen.js';
import { conResaltado, crearJuno } from './juno_panel.js';
import { dibujarEquipos } from './lista_equipos.js';
import { dibujarServicios, marcarFilas, resaltarFila } from './lista_servicios.js';

const vistaServicios = document.getElementById('vista-servicios');
const vistaEquipos = document.getElementById('vista-equipos');
const pestanaServicios = document.getElementById('pestana-servicios');
const pestanaEquipos = document.getElementById('pestana-equipos');
const avisoError = document.getElementById('aviso-error');
const zonaLista = document.getElementById('zona-lista');
const zonaBorrador = crear('div', 'pj-zona-bor'); // la ficha en construcción; se coloca entre el título y la lista
const panelJuno = document.getElementById('juno');
const esMovil = () => matchMedia('(max-width: 900px)').matches;

let imagenNueva = null; // { blob, url }: la imagen que se eligió y se subirá cuando Juno confirme
let borradorActual = null;

function mostrarError(texto) {
  avisoError.textContent = texto;
  avisoError.hidden = false;
}

// Un aviso breve arriba (por ejemplo, «Cambio de bisagras ya lo ven tus clientes»).
function avisar(contenido, esError = false) {
  document.querySelectorAll('.pj-toast').forEach((t) => t.remove());
  const aviso = crear('div', 'pj-toast' + (esError ? ' error' : ''));
  aviso.setAttribute('role', 'status');
  if (typeof contenido === 'string') conResaltado(contenido, aviso);
  else aviso.append(contenido);
  document.body.append(aviso);
  setTimeout(() => aviso.remove(), 7000);
}

function soltarImagen() {
  if (imagenNueva) URL.revokeObjectURL(imagenNueva.url);
  imagenNueva = null;
}

// ---------- Pestañas ----------

function elegirPestana(nombre) {
  const enServicios = nombre === 'servicios';
  vistaServicios.hidden = !enServicios;
  vistaEquipos.hidden = enServicios;
  pestanaServicios.setAttribute('aria-selected', String(enServicios));
  pestanaEquipos.setAttribute('aria-selected', String(!enServicios));
}
pestanaServicios.addEventListener('click', () => elegirPestana('servicios'));
pestanaEquipos.addEventListener('click', () => elegirPestana('equipos'));

// ---------- Juno y la ficha ----------

function abrirJuno() {
  if (!esMovil()) return;
  panelJuno.classList.add('abierto');
}
function cerrarJuno() {
  panelJuno.classList.remove('abierto');
}
document.getElementById('juno-fab').addEventListener('click', abrirJuno);
document.getElementById('juno-cerrar').addEventListener('click', cerrarJuno);

const juno = crearJuno(panelJuno, { alTurno, alAbrir: abrirJuno });

const ficha = crearBorrador(zonaBorrador, {
  alAccion: (accion, texto) => juno.enviar({ texto, accion }),
  alAviso: (activo) =>
    juno.enviar({ texto: activo ? 'Avisar que el precio puede cambiar' : 'Quitar el aviso de precio', accion: activo ? 'aviso:1' : 'aviso:0' }),
  alImagen: async () => {
    try {
      const elegida = await elegirImagen();
      if (!elegida) return;
      soltarImagen();
      imagenNueva = elegida;
      juno.enviar({ texto: 'Agregar una imagen', accion: 'imagen_nueva' });
    } catch (error) {
      avisar(error.message || 'No se pudo usar esa imagen.', true);
    }
  },
  alQuitarImagen: () => juno.enviar({ texto: 'Quitar la imagen', accion: 'imagen_quitar' }),
  alRestaurarImagen: () => juno.enviar({ texto: 'Dejar la imagen como estaba', accion: 'imagen_restaurar' }),
});

function dibujarFicha(borrador, opciones = {}) {
  borradorActual = borrador;
  if (!borrador || borrador.imagen !== 'nueva') soltarImagen();
  ficha.dibujar(borrador, { ...opciones, imagenNueva: imagenNueva ? imagenNueva.url : null });
  marcarFilas(zonaLista, borrador);
}

// Cada respuesta de Juno: se dibuja la ficha y, si ya se creó, editó o eliminó algo, se avisa y se recarga la lista.
async function alTurno(salida) {
  const hecho = salida.hecho;
  if (!hecho) {
    dibujarFicha(salida.borrador, { foco: salida.foco });
    return;
  }
  const imagen = hecho.imagen === 'nueva' ? imagenNueva : null;
  const imagenLista = imagen ? { ...imagen } : null;
  imagenNueva = null; // se suelta después de subirla
  borradorActual = null;
  ficha.dibujar(null, { reiniciar: true });
  marcarFilas(zonaLista, null);

  let fallaImagen = false;
  if (imagenLista) {
    try {
      const subida = await subirImagenServicio(hecho.codigo, imagenLista.blob);
      fallaImagen = !subida.ok;
    } catch {
      fallaImagen = true;
    }
    URL.revokeObjectURL(imagenLista.url);
  }
  await recargar();

  if (hecho.tipo === 'eliminado') {
    avisar('**' + hecho.nombre + '** ya no se muestra a los clientes');
  } else if (fallaImagen) {
    avisar('Se guardó **' + hecho.nombre + '**, pero la imagen no se pudo subir. Edítalo y vuelve a intentarlo.', true);
    resaltarFila(zonaLista, hecho.codigo, hecho.tipo === 'creado' ? 'NUEVO' : 'EDITADO');
  } else {
    const aviso = document.createDocumentFragment();
    aviso.append(crear('b', 'rs', hecho.nombre), hecho.tipo === 'creado' ? ' ya lo ven tus clientes ' : ' se actualizó ');
    const enlace = crear('a', '', 'Ver en /servicios');
    enlace.href = '/servicios';
    enlace.target = '_blank';
    enlace.rel = 'noopener';
    aviso.append(enlace);
    avisar(aviso);
    resaltarFila(zonaLista, hecho.codigo, hecho.tipo === 'creado' ? 'NUEVO' : 'EDITADO');
  }
}

// ---------- Datos ----------

let ultimosEquipos = [];

// Trae las dos listas del servidor y las dibuja. Se llama al abrir y después de cada cambio.
async function recargar() {
  try {
    const [equipos, servicios] = await Promise.all([listarEquipos(), listarServicios()]);
    if (!equipos.ok || !servicios.ok) {
      mostrarError('No se pudieron cargar los datos. Recarga la página.');
      return;
    }
    ultimosEquipos = equipos.datos;
    dibujarServicios(zonaLista, servicios.datos, {
      zonaBorrador,
      crear: () => {
        abrirJuno();
        if (juno.estado().modo) {
          avisar('Ya tienes una ficha en marcha: termínala o descártala primero.', true);
          return;
        }
        juno.enviar({ texto: 'Agregar un servicio' });
      },
      editar: (servicio) => juno.enviar({ texto: 'Editar ' + servicio.nombre, accion: 'editar:' + servicio.codigo }),
      eliminar: (servicio) => juno.enviar({ texto: 'Eliminar ' + servicio.nombre, accion: 'eliminar:' + servicio.codigo }),
    });
    dibujarEquipos(vistaEquipos, equipos.datos, mostrarError, recargar);
    marcarFilas(zonaLista, borradorActual);
  } catch {
    mostrarError('No hay conexión con el servidor. Revisa tu internet y recarga la página.');
  }
}

async function iniciar() {
  const yo = await quienSoy();
  if (!yo.ok) {
    window.location.href = '/ingresar';
    return;
  }
  document.getElementById('cabecera-usuario').textContent = yo.datos.nombre + ' · ' + yo.datos.usuario;
  await recargar();
  const inicio = juno.iniciar();
  dibujarFicha(inicio.borrador, { sinAnimar: true });
  // La imagen elegida antes de recargar la página ya no existe: Juno la quita de la ficha.
  if (inicio.borrador && inicio.borrador.imagen === 'nueva') {
    juno.enviar({ texto: 'Quitar la imagen', accion: 'imagen_quitar', mostrar: false });
  }
}

document.getElementById('boton-salir').addEventListener('click', async () => {
  try {
    await salir();
  } finally {
    window.location.href = '/ingresar';
  }
});

iniciar();
