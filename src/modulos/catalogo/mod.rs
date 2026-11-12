// Módulo "catalogo": los productos que vende H&Devs y los apartados que hacen los clientes.
// El cliente aparta desde el asistente; el Vendedor (o el Administrador) administra todo desde el panel.
pub mod apartados;           // reglas de los apartados: cuántos días duran, estados, entregar y cancelar
pub mod gestion;             // reglas de crear, editar y eliminar categorías y productos
pub mod modelo;
pub mod repositorio;         // SQL de lectura y de apartados (lo usan el agente y el panel)
pub mod repositorio_gestion; // SQL de altas, cambios y bajas de categorías y productos
pub mod rutas;               // /panel/api/categorias, /panel/api/productos y /panel/api/apartados
pub mod validaciones;        // revisa nombre, descripción, precio y stock
