// Módulo "acceso": quién entra al sistema (clientes y personal).
pub mod codigo_otp; // código de 6 dígitos: generarlo, guardar su huella y compararlo (sin base de datos)
pub mod consulta_documento; // consulta de datos por DNI/RUC (hoy simulada; luego Factiliza)
pub mod modelo;
pub mod personal; // empleados, usuarios, roles y sesiones del equipo de H&Devs
pub mod repositorio;
pub mod repositorio_verificacion; // consultas SQL de los códigos de verificación del celular
pub mod rutas;
pub mod verificacion_telefono; // pedir y comprobar el código que llega por SMS
