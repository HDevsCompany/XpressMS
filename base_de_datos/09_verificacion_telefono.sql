-- 09_verificacion_telefono.sql
-- Verificación del celular con un código de 6 dígitos que llega por SMS (se pide al registrarse).
--   1) Acceso.VerificacionesTelefono: cada código enviado, con su vencimiento, intentos y estado.
--   2) Acceso.Clientes.ClientesTelefonoVerificado: cuándo se verificó el celular del cliente (NULL = nunca).
-- El código NUNCA se guarda tal cual: solo su huella (SHA-256 con una sal propia de cada código).
-- Estado del código:  P = pendiente (se puede usar)  V = verificado (ya se usó)  E = expirado (venció, se agotaron
-- los intentos o se pidió uno más nuevo).
-- Esta tabla también sirve de freno: se cuentan los códigos pedidos por celular y por dirección de internet
-- en los últimos 10 minutos (máximo 3 de cada uno).
-- Ejecutar UNA vez en el SQL Editor de Neon (después de 02_agente.sql), seleccionando todo el script.

Alter table Acceso.Clientes add column ClientesTelefonoVerificado timestamptz;

Create table Acceso.VerificacionesTelefono
	(
	VerificacionesTelefonoCodigo char(9),
	VerificacionesTelefonoConversacion char(9) not null,
	VerificacionesTelefonoTelefono varchar(13) not null, -- formato internacional E.164: +51987654321
	VerificacionesTelefonoSal char(32) not null,
	VerificacionesTelefonoHash char(64) not null,
	VerificacionesTelefonoIntentos smallint not null Default 0,
	VerificacionesTelefonoRed varchar(64) not null, -- dirección de internet de quien lo pidió
	VerificacionesTelefonoEstado char(1) not null Default 'P',
	VerificacionesTelefonoFechaCreacion timestamptz not null Default now(),
	VerificacionesTelefonoFechaExpiracion timestamptz not null,
	VerificacionesTelefonoFechaVerificacion timestamptz,
	constraint VerificacionesTelefonoPK primary key (VerificacionesTelefonoCodigo),
	constraint VerificacionesTelefonoConversacionesFK foreign key (VerificacionesTelefonoConversacion)
		references Agente.Conversaciones (ConversacionesCodigo),
	constraint VerificacionesTelefonoEstadoCK check (VerificacionesTelefonoEstado in ('P', 'V', 'E')),
	constraint VerificacionesTelefonoIntentosCK check (VerificacionesTelefonoIntentos >= 0)
	);

-- Para contar rápido los códigos pedidos por celular y por red en la ventana de tiempo.
Create index VerificacionesTelefonoTelefonoIX
	on Acceso.VerificacionesTelefono (VerificacionesTelefonoTelefono, VerificacionesTelefonoFechaCreacion);
Create index VerificacionesTelefonoRedIX
	on Acceso.VerificacionesTelefono (VerificacionesTelefonoRed, VerificacionesTelefonoFechaCreacion);
-- Para encontrar el código pendiente de una conversación.
Create index VerificacionesTelefonoConversacionIX
	on Acceso.VerificacionesTelefono (VerificacionesTelefonoConversacion, VerificacionesTelefonoEstado);
