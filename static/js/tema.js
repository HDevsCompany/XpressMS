// tema.js: decide el color del chat (ver static/css/temas.css) ANTES de que se dibuje la página, para que no parpadee.
// Es un script común (no un módulo) y va en el <head> de chat.html.
//   - Si la dirección trae ?motivo=registro, el tema es "verde"; con ?motivo=mirar, "bronce". Se guarda en el navegador.
//   - Si no trae motivo, se usa el tema guardado (así el color se mantiene al recargar).
//   - Un cliente reconocido no tiene tema (azul): static/js/acceso/cliente.js lo borra al identificarse.
// La clave «xpressms_tema» es la misma que usa static/js/almacen.js.
(function () {
  // Dentro de la burbuja de Juno (página principal) el chat siempre va en azul y no toca el tema guardado.
  if (window.self !== window.top) return;
  var CLAVE = 'xpressms_tema';
  var TEMAS = { registro: 'verde', mirar: 'bronce' };
  var tema = null;
  try {
    var motivo = new URLSearchParams(location.search).get('motivo');
    if (motivo && TEMAS[motivo]) {
      tema = TEMAS[motivo];
      localStorage.setItem(CLAVE, tema);
    } else {
      tema = localStorage.getItem(CLAVE);
    }
  } catch (error) {
    /* sin almacenamiento: se usa el tema de la dirección (si lo hay) o el azul */
  }
  if (tema === 'verde' || tema === 'bronce') document.documentElement.dataset.tema = tema;
})();
