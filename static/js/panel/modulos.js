// modulos.js: los apartados del panel y qué roles pueden verlos.
// Para sumar un apartado nuevo basta agregar una línea aquí. El menú solo es comodidad:
// quien de verdad permite o niega el acceso es el backend en cada ruta.

export const MODULOS = [
  {
    titulo: 'Servicios y equipos',
    descripcion: 'Crea, edita y elimina los servicios que el asistente muestra a los clientes y los tipos de equipo que atiendes.',
    ruta: '/panel/servicios',
    roles: ['ADM'],
  },
  {
    titulo: 'Órdenes de servicio',
    descripcion: 'Mira las citas y órdenes de los clientes, asigna técnicos y avanza cada orden hasta la entrega.',
    ruta: '/panel/ordenes',
    roles: ['ADM', 'TEC'],
  },
  {
    titulo: 'Productos y categorías',
    descripcion: 'Administra el catálogo que ve el cliente: categorías, productos, precios y unidades disponibles.',
    ruta: '/panel/productos',
    roles: ['ADM', 'VEN'],
  },
  {
    titulo: 'Apartados',
    descripcion: 'Mira los productos que los clientes apartaron, confirma la entrega cuando recogen y pagan, o cancela el apartado.',
    ruta: '/panel/apartados',
    roles: ['ADM', 'VEN'],
  },
  {
    titulo: 'Personal y usuarios',
    descripcion: 'Da de alta a técnicos y vendedores, asigna sus roles, restablece claves y da de baja cuentas.',
    ruta: '/panel/personal',
    roles: ['ADM'],
  },
];
