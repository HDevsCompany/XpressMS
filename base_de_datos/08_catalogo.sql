-- 08_catalogo.sql
-- Catálogo de productos de H&Devs y apartados (esquema Catalogo).
-- El cliente registrado ve los productos en el asistente y APARTA uno: se le descuenta del stock y
-- tiene unos días para pasar a recogerlo y pagarlo en tienda (no hay pago en línea). El Vendedor
-- confirma la entrega o cancela el apartado desde el panel. Si el apartado vence, el stock vuelve.
-- La base de datos solo guarda datos y protege su integridad (claves, checks, not null).
-- Las reglas (cuántos días dura un apartado, cuántas unidades se pueden apartar, quién puede cambiar
-- qué) viven en el backend: src/modulos/catalogo/.
-- Los códigos de 9 caracteres los genera el backend (src/modulos/comun/codigos.rs).
-- Ejecutar UNA vez en el SQL Editor de Neon (después de 04_acceso_personal.sql), seleccionando todo el script.
-- "Eliminar" nunca borra la fila: pone Estado = 'E'.

Create schema Catalogo;

-- ---------------------------------------------------------------------------------------------
-- Categorias: cómo se agrupan los productos (Laptops, Periféricos, Almacenamiento...).
-- El nombre es único solo entre las ACTIVAS y sin importar mayúsculas (igual que en 05).
-- ---------------------------------------------------------------------------------------------
Create table Catalogo.Categorias
	(
	CategoriasCodigo char(9),
	CategoriasNombre varchar(60) not null,
	CategoriasEstado char(1) Default 'A',
	constraint CategoriasPK primary key (CategoriasCodigo),
	constraint CategoriasEstadoCK check (CategoriasEstado = 'A' or CategoriasEstado = 'E')
	);

Create unique index CategoriasNombreActivoUX
	on Catalogo.Categorias (lower(CategoriasNombre)) where CategoriasEstado = 'A';

-- ---------------------------------------------------------------------------------------------
-- Productos: lo que la tienda vende. Precio en soles. Stock = unidades disponibles para apartar
-- (las unidades apartadas ya están descontadas). FechaActualizacion la actualiza el backend.
-- ---------------------------------------------------------------------------------------------
Create table Catalogo.Productos
	(
	ProductosCodigo char(9),
	ProductosCategoria char(9) not null,
	ProductosNombre varchar(120) not null,
	ProductosDescripcion varchar(500) not null,
	ProductosPrecio numeric(10,2) not null,
	ProductosStock integer not null Default 0,
	ProductosFechaRegistro timestamptz not null Default now(),
	ProductosFechaActualizacion timestamptz not null Default now(),
	ProductosEstado char(1) Default 'A',
	constraint ProductosPK primary key (ProductosCodigo),
	constraint ProductosCategoriasFK foreign key (ProductosCategoria)
		references Catalogo.Categorias (CategoriasCodigo),
	constraint ProductosPrecioCK check (ProductosPrecio >= 0),
	constraint ProductosStockCK check (ProductosStock >= 0),
	constraint ProductosEstadoCK check (ProductosEstado = 'A' or ProductosEstado = 'E')
	);

Create unique index ProductosNombreActivoUX
	on Catalogo.Productos (lower(ProductosNombre)) where ProductosEstado = 'A';
Create index ProductosCategoriaIX on Catalogo.Productos (ProductosCategoria);

-- ---------------------------------------------------------------------------------------------
-- EstadosReserva: en qué punto está un apartado.
--   PEN = Apartado (esperando que el cliente lo recoja), ENT = Entregado,
--   CAN = Cancelado (por el cliente o por la tienda), VEN = Vencido (pasó el plazo).
-- ---------------------------------------------------------------------------------------------
Create table Catalogo.EstadosReserva
	(
	EstadosReservaCodigo char(3),
	EstadosReservaNombre varchar(40) not null,
	EstadosReservaEstado char(1) Default 'A',
	constraint EstadosReservaPK primary key (EstadosReservaCodigo),
	constraint EstadosReservaNombreUK unique (EstadosReservaNombre),
	constraint EstadosReservaEstadoCK check (EstadosReservaEstado = 'A' or EstadosReservaEstado = 'E')
	);

Insert into Catalogo.EstadosReserva (EstadosReservaCodigo, EstadosReservaNombre)
	values ('PEN', 'Apartado'), ('ENT', 'Entregado'), ('CAN', 'Cancelado'), ('VEN', 'Vencido');

-- ---------------------------------------------------------------------------------------------
-- Reservas: un apartado de un cliente. Un apartado es de UN producto (y una cantidad).
-- PrecioUnitario guarda el precio del momento: si el producto cambia de precio después, el
-- apartado conserva el que vio el cliente.
-- FechaVence: hasta cuándo puede recogerlo. Usuario: el personal que lo entregó o canceló
-- (nulo si lo canceló el propio cliente o venció solo).
-- ---------------------------------------------------------------------------------------------
Create table Catalogo.Reservas
	(
	ReservasCodigo char(9),
	ReservasCliente char(9) not null,
	ReservasProducto char(9) not null,
	ReservasCantidad integer not null,
	ReservasPrecioUnitario numeric(10,2) not null,
	ReservasEstadoReserva char(3) not null Default 'PEN',
	ReservasFechaVence timestamptz not null,
	ReservasUsuario char(9),
	ReservasFechaRegistro timestamptz not null Default now(),
	ReservasFechaActualizacion timestamptz not null Default now(),
	constraint ReservasPK primary key (ReservasCodigo),
	constraint ReservasClientesFK foreign key (ReservasCliente)
		references Acceso.Clientes (ClientesCodigo),
	constraint ReservasProductosFK foreign key (ReservasProducto)
		references Catalogo.Productos (ProductosCodigo),
	constraint ReservasEstadosReservaFK foreign key (ReservasEstadoReserva)
		references Catalogo.EstadosReserva (EstadosReservaCodigo),
	constraint ReservasUsuariosFK foreign key (ReservasUsuario)
		references Acceso.Usuarios (UsuariosCodigo),
	constraint ReservasCantidadCK check (ReservasCantidad > 0),
	constraint ReservasPrecioUnitarioCK check (ReservasPrecioUnitario >= 0)
	);

Create index ReservasClienteIX on Catalogo.Reservas (ReservasCliente);
Create index ReservasProductoIX on Catalogo.Reservas (ReservasProducto);
-- Para encontrar rápido los apartados pendientes que ya vencieron.
Create index ReservasPendientesIX on Catalogo.Reservas (ReservasFechaVence) where ReservasEstadoReserva = 'PEN';
