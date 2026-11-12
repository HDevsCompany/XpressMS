-- 07_ordenes.sql
-- Órdenes de servicio de H&Devs (esquema ServicioTecnico): citas, seguimiento y historial.
-- Una ORDEN es una sola cosa que nace como cita ("Cita agendada") y avanza por estados hasta
-- "Entregado". Así el cliente ve un único código de principio a fin.
-- La base de datos solo guarda datos y protege su integridad (claves, checks, not null).
-- Las reglas (horarios, cupos por hora, quién puede cambiar qué estado) viven en el backend:
-- src/modulos/servicio_tecnico/ (agenda.rs, estados.rs, repositorio_ordenes.rs).
-- Los códigos de orden (9 caracteres) los genera el backend (src/modulos/comun/codigos.rs).
-- Ejecutar UNA vez en el SQL Editor de Neon (después de 04_acceso_personal.sql), seleccionando todo el script.
-- "Eliminar" nunca borra la fila: pone Estado = 'E'.

-- ---------------------------------------------------------------------------------------------
-- EstadosOrden: catálogo de los pasos por los que pasa una orden.
-- Secuencia ordena los estados en pantalla (CAN va al final: es una salida, no un paso).
-- ---------------------------------------------------------------------------------------------
Create table ServicioTecnico.EstadosOrden
	(
	EstadosOrdenCodigo char(3),
	EstadosOrdenNombre varchar(40) not null,
	EstadosOrdenSecuencia smallint not null,
	EstadosOrdenEstado char(1) Default 'A',
	constraint EstadosOrdenPK primary key (EstadosOrdenCodigo),
	constraint EstadosOrdenNombreUK unique (EstadosOrdenNombre),
	constraint EstadosOrdenSecuenciaUK unique (EstadosOrdenSecuencia),
	constraint EstadosOrdenEstadoCK check (EstadosOrdenEstado = 'A' or EstadosOrdenEstado = 'E')
	);

Insert into ServicioTecnico.EstadosOrden (EstadosOrdenCodigo, EstadosOrdenNombre, EstadosOrdenSecuencia)
	values ('CIT', 'Cita agendada', 1),
		('REC', 'Recibido en taller', 2),
		('REV', 'En revisión', 3),
		('REP', 'En reparación', 4),
		('LIS', 'Listo para recoger', 5),
		('ENT', 'Entregado', 6),
		('CAN', 'Cancelada', 9);

-- ---------------------------------------------------------------------------------------------
-- Ordenes: una fila por cita/orden de servicio.
-- Cliente: quién la pidió (Acceso.Clientes). Servicio y TipoEquipo: qué se va a atender.
-- Problema: lo que el cliente cuenta que le pasa al equipo, con sus palabras.
-- FechaCita: el día y la hora agendados (se guarda con zona horaria; Lima es UTC-5).
-- Tecnico: quién la atiende (Acceso.Usuarios). Es nulo hasta que el Administrador la asigna.
-- EstadoOrden: en qué paso va. Nace en 'CIT'.
-- FechaActualizacion la actualiza el backend cada vez que cambia el estado o el técnico.
-- ---------------------------------------------------------------------------------------------
Create table ServicioTecnico.Ordenes
	(
	OrdenesCodigo char(9),
	OrdenesCliente char(9) not null,
	OrdenesServicio char(9) not null,
	OrdenesTipoEquipo char(9),
	OrdenesProblema varchar(500) not null,
	OrdenesFechaCita timestamptz not null,
	OrdenesTecnico char(9),
	OrdenesEstadoOrden char(3) not null Default 'CIT',
	OrdenesFechaRegistro timestamptz not null Default now(),
	OrdenesFechaActualizacion timestamptz not null Default now(),
	constraint OrdenesPK primary key (OrdenesCodigo),
	constraint OrdenesClientesFK foreign key (OrdenesCliente)
		references Acceso.Clientes (ClientesCodigo),
	constraint OrdenesServiciosFK foreign key (OrdenesServicio)
		references ServicioTecnico.Servicios (ServiciosCodigo),
	constraint OrdenesTiposEquipoFK foreign key (OrdenesTipoEquipo)
		references ServicioTecnico.TiposEquipo (TiposEquipoCodigo),
	constraint OrdenesUsuariosFK foreign key (OrdenesTecnico)
		references Acceso.Usuarios (UsuariosCodigo),
	constraint OrdenesEstadosOrdenFK foreign key (OrdenesEstadoOrden)
		references ServicioTecnico.EstadosOrden (EstadosOrdenCodigo),
	constraint OrdenesProblemaCK check (char_length(btrim(OrdenesProblema)) >= 5)
	);

-- Índices para las consultas más comunes: "mis órdenes" (cliente y técnico), filtrar por estado
-- y contar cuántas citas hay en una hora.
Create index OrdenesClienteIX on ServicioTecnico.Ordenes (OrdenesCliente);
Create index OrdenesTecnicoIX on ServicioTecnico.Ordenes (OrdenesTecnico);
Create index OrdenesEstadoOrdenIX on ServicioTecnico.Ordenes (OrdenesEstadoOrden);
Create index OrdenesFechaCitaIX on ServicioTecnico.Ordenes (OrdenesFechaCita);

-- ---------------------------------------------------------------------------------------------
-- OrdenesHistorial: cada vez que una orden cambia de estado se guarda una fila (quién y cuándo).
-- Es lo que permite mostrar el seguimiento con sus fechas. La primera fila ('CIT') la crea el
-- backend junto con la orden. Usuario es nulo cuando el cambio lo hace el propio cliente
-- (por ejemplo, al agendar o cancelar desde el agente).
-- ---------------------------------------------------------------------------------------------
Create table ServicioTecnico.OrdenesHistorial
	(
	OrdenesHistorialCodigo char(9),
	OrdenesHistorialOrden char(9) not null,
	OrdenesHistorialEstadoOrden char(3) not null,
	OrdenesHistorialUsuario char(9),
	OrdenesHistorialNota varchar(300),
	OrdenesHistorialFecha timestamptz not null Default now(),
	constraint OrdenesHistorialPK primary key (OrdenesHistorialCodigo),
	constraint OrdenesHistorialOrdenesFK foreign key (OrdenesHistorialOrden)
		references ServicioTecnico.Ordenes (OrdenesCodigo),
	constraint OrdenesHistorialEstadosOrdenFK foreign key (OrdenesHistorialEstadoOrden)
		references ServicioTecnico.EstadosOrden (EstadosOrdenCodigo),
	constraint OrdenesHistorialUsuariosFK foreign key (OrdenesHistorialUsuario)
		references Acceso.Usuarios (UsuariosCodigo)
	);

Create index OrdenesHistorialOrdenIX on ServicioTecnico.OrdenesHistorial (OrdenesHistorialOrden);
