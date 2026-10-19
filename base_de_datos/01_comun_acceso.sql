Create schema Comun;
Create schema Acceso;

Create table Comun.TiposDocumento
	(
	TiposDocumentoCodigo char(1),
	TiposDocumentoNombre varchar(50) not null,
	TiposDocumentoLongitud smallint not null,
	TiposDocumentoEstado char(1) Default 'A',
	constraint TiposDocumentoPK primary key (TiposDocumentoCodigo),
	constraint TiposDocumentoNombreUK unique (TiposDocumentoNombre),
	constraint TiposDocumentoLongitudCK check (TiposDocumentoLongitud > 0),
	constraint TiposDocumentoEstadoCK check (TiposDocumentoEstado = 'A' or
		TiposDocumentoEstado = 'E')
	);

Insert into Comun.TiposDocumento (TiposDocumentoCodigo, TiposDocumentoNombre, TiposDocumentoLongitud)
	values ('1', 'DNI', 8), ('6', 'RUC', 11), ('4', 'Carnet de extranjería', 12), ('7', 'Pasaporte', 12);

Create table Acceso.Clientes
	(
	ClientesCodigo char(9),
	ClientesTipoDocumento char(1) not null,
	ClientesNumeroDocumento varchar(12) not null,
	ClientesPaterno varchar(100) Default '',
	ClientesMaterno varchar(100),
	ClientesNombres varchar(100),
	ClientesRazonSocial varchar(200),
	ClientesNombreCompleto varchar(400) generated always as
		(Upper(btrim(coalesce(nullif(ClientesRazonSocial, '') || ' ', '') ||
			coalesce(nullif(ClientesPaterno, '') || ' ', '') ||
			coalesce(nullif(ClientesMaterno, '') || ' ', '') ||
			coalesce(nullif(ClientesNombres, ''), '')))) stored,
	ClientesDireccion varchar(120) not null,
	ClientesSexo char(1),
	ClientesFechaNacimiento date,
	ClientesTelefono varchar(25) not null,
	ClientesCorreo varchar(150),
	ClientesFechaRegistro timestamptz not null Default now(),
	ClientesFechaConsentimiento timestamptz not null,
	ClientesEstado char(1) Default 'A',
	constraint ClientesPK primary key (ClientesCodigo),
	constraint ClientesDocumentoUK unique (ClientesTipoDocumento, ClientesNumeroDocumento),
	constraint ClientesTiposDocumentoFK foreign key (ClientesTipoDocumento)
		references Comun.TiposDocumento (TiposDocumentoCodigo),
	constraint ClientesNombreCK check (nullif(ClientesRazonSocial, '') is not null or
		nullif(ClientesNombres, '') is not null),
	constraint ClientesSexoCK check (ClientesSexo = 'M' or ClientesSexo = 'F'),
	constraint ClientesEstadoCK check (ClientesEstado = 'A' or ClientesEstado = 'E')
	);
