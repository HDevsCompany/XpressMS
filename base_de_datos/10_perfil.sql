-- 10_perfil.sql
-- «Mi perfil y datos»: las direcciones de entrega y recojo de cada cliente y las personas que pueden recoger sus equipos.
--   1) Acceso.ClientesDirecciones: las direcciones guardadas (Casa, Oficina...). Una sola puede ser la principal.
--      La dirección con la que el cliente se registró (Acceso.Clientes.ClientesDireccion) se copia sola como «Casa»
--      principal la primera vez que el cliente abre su perfil o le pide direcciones a Juno.
--   2) Acceso.PersonasAutorizadas: quién más puede retirar los equipos del taller (nombre, documento y parentesco).
--      La persona autorizada debe presentar su documento físico en tienda. El máximo por cliente (3) lo controla la aplicación.
-- «Quitar» no borra la fila: la marca como eliminada (Estado = 'E'), para conservar el historial.
-- Estado:  A = activa  E = eliminada.
-- Todo se cambia desde el chat con Juno (con código por SMS para autorizar personas); la página de perfil solo muestra.
-- Ejecutar UNA vez en el SQL Editor de Neon (después de 09_verificacion_telefono.sql), seleccionando todo el script.

Create table Acceso.ClientesDirecciones
	(
	ClientesDireccionesCodigo char(9),
	ClientesDireccionesCliente char(9) not null,
	ClientesDireccionesNombre varchar(40) not null, -- cómo la llama el cliente: Casa, Oficina...
	ClientesDireccionesDireccion varchar(120) not null,
	ClientesDireccionesReferencia varchar(120),
	ClientesDireccionesPrincipal boolean not null Default false,
	ClientesDireccionesFechaRegistro timestamptz not null Default now(),
	ClientesDireccionesEstado char(1) not null Default 'A',
	constraint ClientesDireccionesPK primary key (ClientesDireccionesCodigo),
	constraint ClientesDireccionesClientesFK foreign key (ClientesDireccionesCliente)
		references Acceso.Clientes (ClientesCodigo),
	constraint ClientesDireccionesEstadoCK check (ClientesDireccionesEstado in ('A', 'E'))
	);

-- Para listar rápido las direcciones de un cliente.
Create index ClientesDireccionesClienteIX
	on Acceso.ClientesDirecciones (ClientesDireccionesCliente, ClientesDireccionesEstado);
-- Garantiza que un cliente tenga UNA sola dirección principal activa.
Create unique index ClientesDireccionesPrincipalUX
	on Acceso.ClientesDirecciones (ClientesDireccionesCliente)
	where ClientesDireccionesPrincipal and ClientesDireccionesEstado = 'A';

Create table Acceso.PersonasAutorizadas
	(
	PersonasAutorizadasCodigo char(9),
	PersonasAutorizadasCliente char(9) not null,
	PersonasAutorizadasTipoDocumento char(1) not null Default '1',
	PersonasAutorizadasNumeroDocumento varchar(12) not null,
	PersonasAutorizadasNombre varchar(150) not null,
	PersonasAutorizadasParentesco varchar(40) not null,
	PersonasAutorizadasFechaAutorizacion timestamptz not null Default now(), -- cuándo el cliente la confirmó con el código SMS
	PersonasAutorizadasEstado char(1) not null Default 'A',
	constraint PersonasAutorizadasPK primary key (PersonasAutorizadasCodigo),
	constraint PersonasAutorizadasClientesFK foreign key (PersonasAutorizadasCliente)
		references Acceso.Clientes (ClientesCodigo),
	constraint PersonasAutorizadasTiposDocumentoFK foreign key (PersonasAutorizadasTipoDocumento)
		references Comun.TiposDocumento (TiposDocumentoCodigo),
	constraint PersonasAutorizadasTipoCK check (PersonasAutorizadasTipoDocumento in ('1', '4', '7')),
	constraint PersonasAutorizadasEstadoCK check (PersonasAutorizadasEstado in ('A', 'E'))
	);

Create index PersonasAutorizadasClienteIX
	on Acceso.PersonasAutorizadas (PersonasAutorizadasCliente, PersonasAutorizadasEstado);
-- Una misma persona no puede estar dos veces activa para el mismo cliente.
Create unique index PersonasAutorizadasDocumentoUX
	on Acceso.PersonasAutorizadas (PersonasAutorizadasCliente, PersonasAutorizadasTipoDocumento, PersonasAutorizadasNumeroDocumento)
	where PersonasAutorizadasEstado = 'A';
