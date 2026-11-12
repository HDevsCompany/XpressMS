// Módulo "servicio_tecnico": servicios que ofrece la empresa y los tipos de equipo que atiende.
pub mod agenda;               // reglas de la agenda: días, horas y cupos de las citas
pub mod estados;              // estados de una orden y a cuáles puede pasar
pub mod gestion;              // reglas de crear, editar y eliminar (lo usa el panel del Administrador)
pub mod modelo;
pub mod ordenes;              // reglas de las órdenes en el panel: quién ve qué, asignar técnico, cambiar estado
pub mod repositorio;          // SQL de lectura (lo usa el agente)
pub mod repositorio_gestion;  // SQL de escritura y listas del panel
pub mod repositorio_ordenes;  // SQL de las órdenes: agendar, consultar, historial
pub mod repositorio_ordenes_panel; // SQL de las órdenes para el panel del personal
pub mod rutas;                // /panel/api/servicios y /panel/api/equipos
pub mod rutas_ordenes;        // /panel/api/ordenes y /panel/api/tecnicos
pub mod validaciones;         // revisa nombre, descripción, precio y equipos
