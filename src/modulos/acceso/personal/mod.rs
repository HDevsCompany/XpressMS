// Submódulo "personal": cómo entra el equipo de H&Devs a la plataforma (empleados, usuarios, roles y sesiones).
// Los clientes NO pasan por aquí: ellos se identifican con su código en el agente.
pub mod autenticacion; // la lógica: entrar, cambiar clave, restablecer clave
pub mod claves;        // Argon2 (guardar claves), huella SHA-256 del token y clave temporal
pub mod comandos;      // comandos de terminal: crear-admin y restablecer-clave
pub mod cookies;       // leer y armar la cookie de sesión
pub mod gestion;       // reglas de alta, edición, clave y baja del personal (Administrador)
pub mod modelo;        // estructuras: usuario actual, filas de la base
pub mod repositorio;   // SQL de ingreso, sesiones y alta de personal
pub mod repositorio_gestion; // SQL de la lista, edición y baja del personal
pub mod roles;         // códigos de rol y comprobación de permisos
pub mod rutas;         // las "puertas": /ingresar y /acceso/personal/...
pub mod rutas_gestion; // las "puertas" del Administrador: /panel/api/personal
pub mod sesion;        // reconoce quién hace cada petición (extractor)
pub mod validaciones;  // revisa documento, nombres, usuario y roles del personal
