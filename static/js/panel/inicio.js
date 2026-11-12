// inicio.js: la página /panel. Pregunta quién está en sesión y dibuja el contenido según sus roles.

import { crear } from '../dom.js';
import { quienSoy, salir } from './api.js';
import { crearFormularioClave } from './cambiar_clave.js';
import { MODULOS } from './modulos.js';

// Nombre legible de cada rol (los códigos vienen de la tabla Acceso.Roles).
const ROLES = {
  ADM: 'Administrador',
  TEC: 'Técnico',
  VEN: 'Vendedor',
};

const contenido = document.getElementById('contenido');
const cabeceraUsuario = document.getElementById('cabecera-usuario');

// Si la sesión venció o se cerró en otro lado, volvemos a la página de ingreso.
function irAIngresar() {
  window.location.href = '/ingresar';
}

// Bloque con el formulario de cambio de clave.
function bloqueClave(titulo, descripcion, clases, alTerminar) {
  const bloque = crear('section', 'panel-bloque ' + clases);
  bloque.append(crear('h2', '', titulo), crear('p', '', descripcion), crearFormularioClave(alTerminar));
  return bloque;
}

function dibujar(usuario) {
  cabeceraUsuario.textContent = usuario.nombre + ' · ' + usuario.usuario;
  contenido.replaceChildren();

  // Con clave temporal solo se puede cambiar la clave: nada más se muestra hasta hacerlo.
  if (usuario.debe_cambiar_clave) {
    contenido.append(
      bloqueClave(
        'Cambia tu clave temporal',
        'Por seguridad, antes de continuar elige una clave propia. La clave temporal que recibiste dejará de servir.',
        'aviso',
        () => setTimeout(cargar, 800), // al terminar, recargamos los datos para mostrar el panel completo
      ),
    );
    return;
  }

  const saludo = crear('section', 'panel-bloque');
  saludo.append(crear('h2', '', 'Hola, ' + usuario.nombre.split(' ')[0]));
  saludo.append(crear('p', '', 'Estos son tus roles en la plataforma:'));
  const roles = crear('div', 'roles');
  for (const codigo of usuario.roles) {
    roles.append(crear('span', 'rol', ROLES[codigo] || codigo));
  }
  saludo.append(roles);
  contenido.append(saludo);

  // Los apartados a los que puede entrar según sus roles (el backend vuelve a comprobarlo en cada uno).
  const disponibles = MODULOS.filter((modulo) => modulo.roles.some((rol) => usuario.roles.includes(rol)));
  if (disponibles.length > 0) {
    const apartados = crear('section', 'panel-bloque');
    apartados.append(crear('h2', '', 'Apartados'));
    for (const modulo of disponibles) {
      const enlace = crear('a', 'apartado');
      enlace.href = modulo.ruta;
      enlace.append(crear('strong', '', modulo.titulo), crear('span', '', modulo.descripcion));
      apartados.append(enlace);
    }
    contenido.append(apartados);
  }

  contenido.append(
    bloqueClave('Cambiar mi clave', 'Te recomendamos cambiarla si crees que alguien más la conoce.', '', () => {}),
  );
}

async function cargar() {
  try {
    const resultado = await quienSoy();
    if (!resultado.ok) return irAIngresar();
    dibujar(resultado.datos);
  } catch {
    contenido.replaceChildren(crear('p', 'mensaje-error', 'No hay conexión con el servidor. Recarga la página.'));
  }
}

document.getElementById('boton-salir').addEventListener('click', async () => {
  try {
    await salir();
  } finally {
    irAIngresar();
  }
});

cargar();
