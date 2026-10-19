-- 02_agente.sql
-- Esquema del agente conversacional: tablas Conversaciones y Mensajes.
-- La base de datos solo guarda datos y protege su integridad (claves, checks, not null).
-- Los códigos de 9 caracteres y el token los genera el backend (src/modulos/comun/codigos.rs).
-- Ejecutar UNA vez en el SQL Editor de Neon (base XpressMS), seleccionando todo el script.

Create schema Agente;

-- ---------------------------------------------------------------------------------------------
-- Conversaciones: una por visita. El cliente puede ser nulo (visitante que no se registró).
-- Paso y Datos guardan "en qué punto va" la conversación y lo que se ha recogido hasta ahora.
-- Resultado, IntervencionHumana y Valoracion alimentan los indicadores del proyecto.
-- ---------------------------------------------------------------------------------------------
Create table Agente.Conversaciones
	(
	ConversacionesCodigo char(9),
	ConversacionesCliente char(9),
	ConversacionesToken char(32) not null,
	ConversacionesPaso varchar(40) not null Default 'inicio',
	ConversacionesDatos jsonb not null Default '{}',
	ConversacionesResultado varchar(20),
	ConversacionesIntervencionHumana char(1) not null Default 'N',
	ConversacionesValoracion smallint,
	ConversacionesFechaInicio timestamptz not null Default now(),
	ConversacionesFechaActividad timestamptz not null Default now(),
	ConversacionesEstado char(1) Default 'A',
	constraint ConversacionesPK primary key (ConversacionesCodigo),
	constraint ConversacionesTokenUK unique (ConversacionesToken),
	constraint ConversacionesClientesFK foreign key (ConversacionesCliente)
		references Acceso.Clientes (ClientesCodigo),
	constraint ConversacionesResultadoCK check (ConversacionesResultado in
		('registro', 'consulta', 'cita', 'abandono')),
	constraint ConversacionesIntervencionHumanaCK check (ConversacionesIntervencionHumana = 'S' or
		ConversacionesIntervencionHumana = 'N'),
	constraint ConversacionesValoracionCK check (ConversacionesValoracion between 1 and 5),
	constraint ConversacionesEstadoCK check (ConversacionesEstado = 'A' or ConversacionesEstado = 'E')
	);

-- ---------------------------------------------------------------------------------------------
-- Mensajes: cada burbuja del chat. La hora de cada mensaje permite medir el tiempo de respuesta.
-- Emisor: 'P' = persona, 'A' = agente.
-- ---------------------------------------------------------------------------------------------
Create table Agente.Mensajes
	(
	MensajesCodigo char(9),
	MensajesConversacion char(9) not null,
	MensajesEmisor char(1) not null,
	MensajesTexto text not null,
	MensajesFecha timestamptz not null Default now(),
	constraint MensajesPK primary key (MensajesCodigo),
	constraint MensajesConversacionesFK foreign key (MensajesConversacion)
		references Agente.Conversaciones (ConversacionesCodigo),
	constraint MensajesEmisorCK check (MensajesEmisor = 'P' or MensajesEmisor = 'A')
	);

-- Índice para listar rápido los mensajes de una conversación en orden.
Create index MensajesConversacionIX on Agente.Mensajes (MensajesConversacion, MensajesFecha);
