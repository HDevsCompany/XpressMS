// dom.js: pequeña ayuda para crear elementos de la página sin repetir código.

// Crea un elemento HTML. Ejemplo: crear('div', 'burbuja agente', 'Hola')
// -> <div class="burbuja agente">Hola</div>
// Usa textContent (no innerHTML) para que ningún texto pueda inyectar HTML malicioso.
export function crear(etiqueta, clases = '', texto = '') {
  const elemento = document.createElement(etiqueta);
  if (clases) elemento.className = clases;
  if (texto) elemento.textContent = texto;
  return elemento;
}

// Espera una cantidad de milisegundos (se usa con "await pausa(500)").
export function pausa(milisegundos) {
  return new Promise((resolver) => setTimeout(resolver, milisegundos));
}
