// sesion.js: en la página principal (/), si este navegador ya tiene una conversación con un cliente reconocido,
// lo lleva directo al chat (no tiene que ingresar otra vez). En /ingresar no hace nada: allí se puede entrar
// al portal de equipo aunque antes se haya identificado como cliente.
// Para volver a ver la tarjeta de ingreso, en el chat se pulsa «Nueva conversación» (olvida la identificación).

import { sesionReconocida } from '../api.js';
import { leerToken } from '../almacen.js';

async function revisar() {
  if (location.pathname !== '/') return;
  const token = leerToken();
  if (!token) return;
  if (await sesionReconocida(token)) location.replace('/chat');
}

revisar();
