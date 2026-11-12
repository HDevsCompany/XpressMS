-- 05_servicio_tecnico_nombres.sql
-- Cambia la regla de nombres repetidos en Servicios y TiposEquipo (esquema ServicioTecnico).
-- Antes: el nombre era único entre TODOS los registros, incluso los eliminados (Estado = 'E'),
-- así que no se podía volver a crear un servicio que alguien había eliminado.
-- Ahora: el nombre es único solo entre los ACTIVOS y sin importar mayúsculas ("Laptop" = "laptop").
-- Los eliminados conservan su nombre para el historial.
-- Ejecutar UNA vez en el SQL Editor de Neon (después de 03_servicio_tecnico.sql), seleccionando todo el script.

Alter table ServicioTecnico.TiposEquipo drop constraint TiposEquipoNombreUK;
Alter table ServicioTecnico.Servicios drop constraint ServiciosNombreUK;

Create unique index TiposEquipoNombreActivoUX
	on ServicioTecnico.TiposEquipo (lower(TiposEquipoNombre)) where TiposEquipoEstado = 'A';
Create unique index ServiciosNombreActivoUX
	on ServicioTecnico.Servicios (lower(ServiciosNombre)) where ServiciosEstado = 'A';
