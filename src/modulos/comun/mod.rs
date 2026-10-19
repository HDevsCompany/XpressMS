// Módulo "comun": piezas que usan todos los demás módulos (acceso, servicio_tecnico, agente).
// Aquí vive la lógica compartida; la base de datos solo guarda los datos y protege su integridad.
pub mod codigos;       // genera los códigos de 9 caracteres y los tokens
pub mod formato;       // presenta fechas, sexo y direcciones como texto legible
pub mod modelo;        // estructuras de las tablas del esquema Comun
pub mod repositorio;   // consultas SQL del esquema Comun
pub mod validaciones;  // revisa documento, teléfono, correo y nombres
