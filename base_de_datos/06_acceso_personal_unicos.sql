-- 06_acceso_personal_unicos.sql
-- Cambia la regla de repetidos en Empleados y Usuarios (esquema Acceso).
-- Antes: el documento y el nombre de usuario eran únicos entre TODOS los registros, incluso los eliminados
-- (Estado = 'E'), así que no se podía volver a dar de alta a una persona que ya había sido dada de baja.
-- Ahora: son únicos solo entre los ACTIVOS. Los eliminados conservan sus datos para el historial.
-- Ejecutar UNA vez en el SQL Editor de Neon (después de 04_acceso_personal.sql), seleccionando todo el script.

Alter table Acceso.Empleados drop constraint EmpleadosDocumentoUK;
Alter table Acceso.Usuarios drop constraint UsuariosNombreUsuarioUK;

Create unique index EmpleadosDocumentoActivoUX
	on Acceso.Empleados (EmpleadosTipoDocumento, EmpleadosNumeroDocumento) where EmpleadosEstado = 'A';
Create unique index UsuariosNombreUsuarioActivoUX
	on Acceso.Usuarios (UsuariosNombreUsuario) where UsuariosEstado = 'A';
