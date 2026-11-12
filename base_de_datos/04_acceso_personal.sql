-- 04_acceso_personal.sql
-- Acceso del personal de H&Devs: empleados, roles, usuarios y sesiones (esquema Acceso).
-- Los CLIENTES no son usuarios: se identifican por su código en el agente. Aquí solo entra el personal.
-- La base de datos solo guarda datos y protege su integridad (claves, checks, not null).
-- Las reglas (claves seguras, bloqueo por intentos, permisos por rol) viven en el backend:
-- src/modulos/acceso/personal/.
-- Los códigos de 9 caracteres los genera el backend (src/modulos/comun/codigos.rs).
-- Ejecutar UNA vez en el SQL Editor de Neon (base XpressMS), seleccionando todo el script.
-- "Eliminar" nunca borra la fila: pone Estado = 'E'.
-- El primer administrador se crea con:  cargo run -- crear-admin

-- ---------------------------------------------------------------------------------------------
-- Empleados: la PERSONA que trabaja en la empresa (quién es).
-- Un empleado puede existir sin usuario (por ejemplo, alguien que aún no tiene acceso al sistema).
-- ---------------------------------------------------------------------------------------------
Create table Acceso.Empleados
	(
	EmpleadosCodigo char(9),
	EmpleadosTipoDocumento char(1) not null,
	EmpleadosNumeroDocumento varchar(12) not null,
	EmpleadosPaterno varchar(100) not null,
	EmpleadosMaterno varchar(100),
	EmpleadosNombres varchar(100) not null,
	EmpleadosTelefono varchar(25),
	EmpleadosCorreo varchar(150),
	EmpleadosFechaRegistro timestamptz not null Default now(),
	EmpleadosEstado char(1) Default 'A',
	constraint EmpleadosPK primary key (EmpleadosCodigo),
	constraint EmpleadosDocumentoUK unique (EmpleadosTipoDocumento, EmpleadosNumeroDocumento),
	constraint EmpleadosTiposDocumentoFK foreign key (EmpleadosTipoDocumento)
		references Comun.TiposDocumento (TiposDocumentoCodigo),
	constraint EmpleadosEstadoCK check (EmpleadosEstado = 'A' or EmpleadosEstado = 'E')
	);

-- ---------------------------------------------------------------------------------------------
-- Roles: qué puede hacer una persona dentro del sistema (catálogo fijo de la empresa).
--   ADM = Administrador: gestiona servicios, equipos, empleados y usuarios.
--   TEC = Técnico: atiende las órdenes de servicio y cambia su estado.
--   VEN = Vendedor: gestiona el catálogo de productos y las compras.
-- Los permisos concretos de cada rol los decide el backend; aquí solo existe el catálogo.
-- ---------------------------------------------------------------------------------------------
Create table Acceso.Roles
	(
	RolesCodigo char(3),
	RolesNombre varchar(40) not null,
	RolesDescripcion varchar(200) not null,
	RolesEstado char(1) Default 'A',
	constraint RolesPK primary key (RolesCodigo),
	constraint RolesNombreUK unique (RolesNombre),
	constraint RolesEstadoCK check (RolesEstado = 'A' or RolesEstado = 'E')
	);

Insert into Acceso.Roles (RolesCodigo, RolesNombre, RolesDescripcion)
	values ('ADM', 'Administrador', 'Gestiona servicios, tipos de equipo, empleados y usuarios'),
		('TEC', 'Técnico', 'Atiende las órdenes de servicio y actualiza su estado'),
		('VEN', 'Vendedor', 'Gestiona el catálogo de productos y las compras');

-- ---------------------------------------------------------------------------------------------
-- Usuarios: la CUENTA con la que un empleado entra a la plataforma (cómo entra).
-- Un empleado tiene como máximo una cuenta (UsuariosEmpleadoUK).
-- NombreUsuario va siempre en minúsculas (lo garantiza el check y el backend).
-- ClaveHash guarda el resultado de Argon2: la clave real nunca se guarda.
-- DebeCambiarClave = true cuando la clave es temporal (cuenta nueva o clave restablecida).
-- Tras 5 intentos fallidos, el backend llena BloqueadoHasta y la cuenta espera 15 minutos.
-- ---------------------------------------------------------------------------------------------
Create table Acceso.Usuarios
	(
	UsuariosCodigo char(9),
	UsuariosEmpleado char(9) not null,
	UsuariosNombreUsuario varchar(30) not null,
	UsuariosClaveHash varchar(255) not null,
	UsuariosDebeCambiarClave boolean not null Default true,
	UsuariosIntentosFallidos smallint not null Default 0,
	UsuariosBloqueadoHasta timestamptz,
	UsuariosUltimoAcceso timestamptz,
	UsuariosFechaRegistro timestamptz not null Default now(),
	UsuariosEstado char(1) Default 'A',
	constraint UsuariosPK primary key (UsuariosCodigo),
	constraint UsuariosEmpleadoUK unique (UsuariosEmpleado),
	constraint UsuariosNombreUsuarioUK unique (UsuariosNombreUsuario),
	constraint UsuariosEmpleadosFK foreign key (UsuariosEmpleado)
		references Acceso.Empleados (EmpleadosCodigo),
	constraint UsuariosNombreUsuarioCK check (UsuariosNombreUsuario = lower(UsuariosNombreUsuario)),
	constraint UsuariosIntentosFallidosCK check (UsuariosIntentosFallidos >= 0),
	constraint UsuariosEstadoCK check (UsuariosEstado = 'A' or UsuariosEstado = 'E')
	);

-- ---------------------------------------------------------------------------------------------
-- UsuariosRoles: qué roles tiene cada usuario (muchos a muchos).
-- Una persona puede ser Técnico y Vendedor a la vez: son dos filas.
-- ---------------------------------------------------------------------------------------------
Create table Acceso.UsuariosRoles
	(
	UsuariosRolesUsuario char(9),
	UsuariosRolesRol char(3),
	UsuariosRolesFechaAsignacion timestamptz not null Default now(),
	constraint UsuariosRolesPK primary key (UsuariosRolesUsuario, UsuariosRolesRol),
	constraint UsuariosRolesUsuariosFK foreign key (UsuariosRolesUsuario)
		references Acceso.Usuarios (UsuariosCodigo),
	constraint UsuariosRolesRolesFK foreign key (UsuariosRolesRol)
		references Acceso.Roles (RolesCodigo)
	);

-- Índice para la consulta inversa: "¿qué usuarios tienen este rol?".
Create index UsuariosRolesRolIX on Acceso.UsuariosRoles (UsuariosRolesRol);

-- ---------------------------------------------------------------------------------------------
-- Sesiones: cada ingreso al panel. El navegador guarda un token secreto en una cookie HttpOnly;
-- aquí solo se guarda su "huella" SHA-256 (64 caracteres hexadecimales), nunca el token.
-- Así, si alguien viera esta tabla, no podría usar ninguna sesión.
-- Cerrar sesión pone Estado = 'E'. Las sesiones vencidas las limpia el backend.
-- ---------------------------------------------------------------------------------------------
Create table Acceso.Sesiones
	(
	SesionesCodigo char(9),
	SesionesUsuario char(9) not null,
	SesionesTokenHash char(64) not null,
	SesionesFechaCreacion timestamptz not null Default now(),
	SesionesFechaExpiracion timestamptz not null,
	SesionesEstado char(1) Default 'A',
	constraint SesionesPK primary key (SesionesCodigo),
	constraint SesionesTokenHashUK unique (SesionesTokenHash),
	constraint SesionesUsuariosFK foreign key (SesionesUsuario)
		references Acceso.Usuarios (UsuariosCodigo),
	constraint SesionesExpiracionCK check (SesionesFechaExpiracion > SesionesFechaCreacion),
	constraint SesionesEstadoCK check (SesionesEstado = 'A' or SesionesEstado = 'E')
	);

-- Índice para cerrar o contar las sesiones de un usuario.
Create index SesionesUsuarioIX on Acceso.Sesiones (SesionesUsuario);
