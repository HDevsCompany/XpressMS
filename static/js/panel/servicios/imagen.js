// imagen.js (panel/servicios): elegir la imagen de un servicio y reducirla en el navegador antes de enviarla.
// Así nadie sube fotos de 8 MB: se deja a lo mucho 1200 px de ancho o alto, en JPG, y por debajo del límite del servidor (1.5 MB).

const LIMITE_BYTES = 1.5 * 1024 * 1024;
const ORIGEN_MAXIMO = 15 * 1024 * 1024; // lo que se acepta leer antes de reducir
const TIPOS = ['image/jpeg', 'image/png', 'image/webp'];

function cargar(archivo) {
  return new Promise((resolver, rechazar) => {
    const direccion = URL.createObjectURL(archivo);
    const imagen = new Image();
    imagen.onload = () => {
      URL.revokeObjectURL(direccion);
      resolver(imagen);
    };
    imagen.onerror = () => {
      URL.revokeObjectURL(direccion);
      rechazar(new Error('No se pudo leer esa imagen. Prueba con otra (JPG, PNG o WEBP).'));
    };
    imagen.src = direccion;
  });
}

function aBlob(lienzo, calidad) {
  return new Promise((resolver) => lienzo.toBlob(resolver, 'image/jpeg', calidad));
}

// Reduce el archivo elegido. Devuelve { blob, url }: "url" sirve para mostrarla enseguida (hay que soltarla con URL.revokeObjectURL).
export async function prepararImagen(archivo) {
  if (!TIPOS.includes(archivo.type)) throw new Error('La imagen debe ser JPG, PNG o WEBP.');
  if (archivo.size > ORIGEN_MAXIMO) throw new Error('Esa imagen pesa demasiado. Elige una de menos de 15 MB.');
  const imagen = await cargar(archivo);

  let lado = 1200;
  for (let intento = 0; intento < 4; intento++) {
    const escala = Math.min(1, lado / Math.max(imagen.naturalWidth, imagen.naturalHeight));
    const lienzo = document.createElement('canvas');
    lienzo.width = Math.max(1, Math.round(imagen.naturalWidth * escala));
    lienzo.height = Math.max(1, Math.round(imagen.naturalHeight * escala));
    const contexto = lienzo.getContext('2d');
    contexto.fillStyle = '#ffffff'; // los PNG con fondo transparente quedan sobre blanco
    contexto.fillRect(0, 0, lienzo.width, lienzo.height);
    contexto.drawImage(imagen, 0, 0, lienzo.width, lienzo.height);
    for (const calidad of [0.85, 0.7, 0.55]) {
      const blob = await aBlob(lienzo, calidad);
      if (blob && blob.size <= LIMITE_BYTES) return { blob, url: URL.createObjectURL(blob) };
    }
    lado = Math.round(lado * 0.75);
  }
  throw new Error('No pude reducir esa imagen lo suficiente. Prueba con otra.');
}

// Abre el selector de archivos del equipo. Devuelve { blob, url }, o null si la persona cerró el selector sin elegir.
export function elegirImagen() {
  return new Promise((resolver, rechazar) => {
    const entrada = document.createElement('input');
    entrada.type = 'file';
    entrada.accept = 'image/jpeg,image/png,image/webp';
    entrada.addEventListener('change', async () => {
      const archivo = entrada.files && entrada.files[0];
      if (!archivo) {
        resolver(null);
        return;
      }
      try {
        resolver(await prepararImagen(archivo));
      } catch (error) {
        rechazar(error);
      }
    });
    entrada.addEventListener('cancel', () => resolver(null));
    entrada.click();
  });
}
