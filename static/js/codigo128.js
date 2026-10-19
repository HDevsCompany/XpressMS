// codigo128.js: convierte un texto en un código de barras estándar "Code 128" (subconjunto B),
// el que leen todos los lectores de barras y las cámaras de celular.
// Aquí SOLO se calculan las barras; dibujarlas es trabajo de componentes/tarjeta_datos.js.

// Los 107 patrones del estándar. Cada patrón son 6 números: ancho de barra, espacio, barra, espacio, barra, espacio.
// (Posición 103-105 = símbolos de inicio A/B/C; posición 106 = símbolo de fin, que lleva 7 números.)
const PATRONES = [
  '212222', '222122', '222221', '121223', '121322', '131222', '122213', '122312', '132212', '221213',
  '221312', '231212', '112232', '122132', '122231', '113222', '123122', '123221', '223211', '221132',
  '221231', '213212', '223112', '312131', '311222', '321122', '321221', '312212', '322112', '322211',
  '212123', '212321', '232121', '111323', '131123', '131321', '112313', '132113', '132311', '211313',
  '231113', '231311', '112133', '112331', '132131', '113123', '113321', '133121', '313121', '211331',
  '231131', '213113', '213311', '213131', '311123', '311321', '331121', '312113', '312311', '332111',
  '314111', '221411', '431111', '111224', '111422', '121124', '121421', '141122', '141221', '112214',
  '112412', '122114', '122411', '142112', '142211', '241211', '221114', '413111', '241112', '134111',
  '111242', '121142', '121241', '114212', '124112', '124211', '411212', '421112', '421211', '212141',
  '214121', '412121', '111143', '111341', '131141', '114113', '114311', '411113', '411311', '113141',
  '114131', '311141', '411131', '211412', '211214', '211232', '2331112',
];

const INICIO_B = 104;
const FIN = 106;

// Devuelve la lista de anchos alternados [barra, espacio, barra, espacio, ...] (en "módulos")
// o null si el texto tiene caracteres que el subconjunto B no puede representar.
export function anchosCode128(texto) {
  const valores = [INICIO_B];
  for (const letra of texto) {
    const codigo = letra.charCodeAt(0);
    if (codigo < 32 || codigo > 126) return null; // solo caracteres ASCII imprimibles
    valores.push(codigo - 32);
  }

  // Dígito de control: (inicio + cada valor × su posición) módulo 103. Permite al lector detectar errores de lectura.
  let suma = valores[0];
  for (let i = 1; i < valores.length; i++) suma += valores[i] * i;
  valores.push(suma % 103, FIN);

  const anchos = [];
  valores.forEach((valor) => {
    for (const digito of PATRONES[valor]) anchos.push(Number(digito));
  });
  return anchos;
}
