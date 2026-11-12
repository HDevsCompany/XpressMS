// clave_temporal.js: la ventana que muestra la clave temporal UNA sola vez (al dar de alta o restablecer).
// El servidor no la guarda en ningún lado: si se cierra sin anotarla, solo se puede restablecer otra.

import { crear } from '../../dom.js';
import { abrirDialogo, cerrarDialogo } from '../dialogo.js';

// "titulo": qué pasó ("Cuenta creada"). "datos": { usuario, clave_temporal }.
export function mostrarClaveTemporal(titulo, datos) {
  const cuerpo = crear('div', 'dialogo-cuerpo');
  cuerpo.append(crear('h2', '', titulo));
  cuerpo.append(
    crear(
      'p',
      '',
      'Entrégale estos datos a la persona. Al ingresar por primera vez tendrá que cambiar la clave por una propia.',
    ),
  );

  const caja = crear('div', 'clave-temporal');
  const fila = (etiqueta, valor) => {
    const linea = crear('div', 'clave-linea');
    linea.append(crear('span', '', etiqueta), crear('code', '', valor));
    return linea;
  };
  caja.append(fila('Usuario', datos.usuario), fila('Clave temporal', datos.clave_temporal));
  cuerpo.append(caja);
  cuerpo.append(crear('p', 'aviso-unico', 'Anótala o cópiala ahora: no se vuelve a mostrar.'));

  const botones = crear('div', 'dialogo-botones');
  const copiar = crear('button', 'boton-secundario', 'Copiar clave');
  copiar.type = 'button';
  copiar.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(datos.clave_temporal);
      copiar.textContent = 'Copiada';
    } catch {
      copiar.textContent = 'Selecciónala y cópiala a mano';
    }
  });
  const cerrar = crear('button', 'boton-principal', 'Listo');
  cerrar.type = 'button';
  cerrar.addEventListener('click', cerrarDialogo);
  botones.append(copiar, cerrar);
  cuerpo.append(botones);

  abrirDialogo(cuerpo);
}
