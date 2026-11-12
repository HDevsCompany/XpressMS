// roles.js: nombres legibles de los roles y de los documentos (los códigos vienen de la base de datos).
// Para sumar un rol nuevo en la base, se agrega también aquí.

export const ROLES = [
  { codigo: 'ADM', nombre: 'Administrador', descripcion: 'Gestiona servicios, equipos y personal.' },
  { codigo: 'TEC', nombre: 'Técnico', descripcion: 'Atiende las órdenes de servicio.' },
  { codigo: 'VEN', nombre: 'Vendedor', descripcion: 'Gestiona el catálogo y las compras.' },
];

export function nombreRol(codigo) {
  return ROLES.find((rol) => rol.codigo === codigo)?.nombre ?? codigo;
}

export const DOCUMENTOS = [
  { codigo: '1', nombre: 'DNI' },
  { codigo: '4', nombre: 'Carnet de extranjería' },
  { codigo: '7', nombre: 'Pasaporte' },
];

export function nombreDocumento(codigo) {
  return DOCUMENTOS.find((doc) => doc.codigo === codigo)?.nombre ?? 'Documento';
}

// Los nombres se guardan en mayúsculas ("PEDRO JUAN"); para mostrarlos se pasan a "Pedro Juan".
export function aTitulo(texto) {
  return (texto || '')
    .toLowerCase()
    .replace(/(^|[\s'.-])(\p{L})/gu, (_, separador, letra) => separador + letra.toUpperCase());
}
