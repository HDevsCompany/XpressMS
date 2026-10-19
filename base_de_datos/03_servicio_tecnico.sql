-- 03_servicio_tecnico.sql
-- Esquema del servicio técnico: tipos de equipo, servicios y la relación entre ambos.
-- La base de datos solo guarda datos y protege su integridad (claves, checks, not null).
-- Los códigos de 9 caracteres los genera el backend (src/modulos/comun/codigos.rs).
-- Ejecutar UNA vez en el SQL Editor de Neon (base XpressMS), seleccionando todo el script.
-- Los servicios y tipos de equipo los creará el personal desde la plataforma (módulo de gestión, más adelante).
-- "Eliminar" nunca borra la fila: pone Estado = 'E'. Así las citas y órdenes futuras conservan su historial.

Create schema ServicioTecnico;

-- ---------------------------------------------------------------------------------------------
-- TiposEquipo: lo que la empresa atiende (Laptop, PC de escritorio, Componentes, Impresora...).
-- ---------------------------------------------------------------------------------------------
Create table ServicioTecnico.TiposEquipo
	(
	TiposEquipoCodigo char(9),
	TiposEquipoNombre varchar(60) not null,
	TiposEquipoEstado char(1) Default 'A',
	constraint TiposEquipoPK primary key (TiposEquipoCodigo),
	constraint TiposEquipoNombreUK unique (TiposEquipoNombre),
	constraint TiposEquipoEstadoCK check (TiposEquipoEstado = 'A' or TiposEquipoEstado = 'E')
	);

-- ---------------------------------------------------------------------------------------------
-- Servicios: lo que H&Devs ofrece y el agente informa.
-- PrecioBase es referencial: el costo final se confirma al revisar el equipo.
-- Si PrecioBase es nulo, el servicio es "a cotizar".
-- TiempoEstimado es texto libre para mostrar ("Mismo día", "2 a 3 días").
-- FechaActualizacion la actualiza el backend cada vez que alguien edita el servicio.
-- ---------------------------------------------------------------------------------------------
Create table ServicioTecnico.Servicios
	(
	ServiciosCodigo char(9),
	ServiciosNombre varchar(100) not null,
	ServiciosDescripcion varchar(500) not null,
	ServiciosPrecioBase numeric(10,2),
	ServiciosTiempoEstimado varchar(40),
	ServiciosFechaRegistro timestamptz not null Default now(),
	ServiciosFechaActualizacion timestamptz not null Default now(),
	ServiciosEstado char(1) Default 'A',
	constraint ServiciosPK primary key (ServiciosCodigo),
	constraint ServiciosNombreUK unique (ServiciosNombre),
	constraint ServiciosPrecioBaseCK check (ServiciosPrecioBase >= 0),
	constraint ServiciosEstadoCK check (ServiciosEstado = 'A' or ServiciosEstado = 'E')
	);

-- ---------------------------------------------------------------------------------------------
-- ServiciosEquipos: para qué tipos de equipo sirve cada servicio (relación de muchos a muchos).
-- Cada fila es una pareja "este servicio sirve para este equipo".
-- El agente la usa para preguntar "¿para qué equipo?" mostrando solo los equipos que corresponden.
-- ---------------------------------------------------------------------------------------------
Create table ServicioTecnico.ServiciosEquipos
	(
	ServiciosEquiposServicio char(9),
	ServiciosEquiposTipoEquipo char(9),
	constraint ServiciosEquiposPK primary key (ServiciosEquiposServicio, ServiciosEquiposTipoEquipo),
	constraint ServiciosEquiposServiciosFK foreign key (ServiciosEquiposServicio)
		references ServicioTecnico.Servicios (ServiciosCodigo),
	constraint ServiciosEquiposTiposEquipoFK foreign key (ServiciosEquiposTipoEquipo)
		references ServicioTecnico.TiposEquipo (TiposEquipoCodigo)
	);

-- Índice para la consulta inversa: "¿qué servicios tiene este tipo de equipo?".
Create index ServiciosEquiposTipoEquipoIX on ServicioTecnico.ServiciosEquipos (ServiciosEquiposTipoEquipo);
