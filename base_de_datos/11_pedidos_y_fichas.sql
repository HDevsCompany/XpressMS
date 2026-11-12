-- 11_pedidos_y_fichas.sql
-- El carrito con Juno y la ficha de cada producto.
--   1) Catalogo.Reservas.ReservasPedido: un pedido (carrito apartado) crea UN apartado por producto y todos comparten
--      el mismo código de pedido, que es el «código de retiro» que se le da al cliente. Los apartados de un solo
--      producto (el flujo anterior) lo dejan vacío.
--   2) Catalogo.Caracteristicas: lo que muestra la ficha de un producto. Cada fila es un dato (Titulo = Valor) dentro de
--      un Grupo (MEMORIA, ALMACENAMIENTO...) y forma la tabla de «Especificaciones». Las filas del grupo especial RESUMEN
--      NO salen en la tabla: son las «cajas destacadas» del resumen (con su Icono, título pequeño y dato grande).
--      Iconos que entiende la página: cpu, ram, disco, pantalla, bateria, pila, mouse, wifi, sensor, marca, usb,
--      teclado, idioma, escudo, calidad, enchufe, rayo, capacidad, resol, panel, reloj, uso.
--   3) Catalogo.Imagenes: las vistas del producto (frontal, lateral, trasera, empaque). Url puede ser una dirección
--      (https://... o /static/...) o «dibujo:frontal», «dibujo:lateral», «dibujo:trasera», «dibujo:empaque» (dibujos de prueba
--      mientras no haya fotos reales). Si un producto no tiene imágenes, la ficha no muestra la galería.
-- Es seguro ejecutarlo más de una vez. «Eliminar» no borra: pone Estado = 'E'.
-- Ejecutar en el SQL Editor de Neon (después de 10_perfil.sql), seleccionando todo el script.

Alter table Catalogo.Reservas add column if not exists ReservasPedido char(9);
Create index if not exists ReservasPedidoIX on Catalogo.Reservas (ReservasPedido);

Create table if not exists Catalogo.Caracteristicas
	(
	CaracteristicasCodigo char(9),
	CaracteristicasProducto char(9) not null,
	CaracteristicasGrupo varchar(40) not null,
	CaracteristicasTitulo varchar(40) not null,
	CaracteristicasValor varchar(120) not null,
	CaracteristicasIcono varchar(20),
	CaracteristicasOrden integer not null Default 0,
	CaracteristicasEstado char(1) not null Default 'A',
	constraint CaracteristicasPK primary key (CaracteristicasCodigo),
	constraint CaracteristicasProductosFK foreign key (CaracteristicasProducto)
		references Catalogo.Productos (ProductosCodigo),
	constraint CaracteristicasUnicaUK unique (CaracteristicasProducto, CaracteristicasGrupo, CaracteristicasTitulo),
	constraint CaracteristicasEstadoCK check (CaracteristicasEstado in ('A', 'E'))
	);
Create index if not exists CaracteristicasProductoIX on Catalogo.Caracteristicas (CaracteristicasProducto);

Create table if not exists Catalogo.Imagenes
	(
	ImagenesCodigo char(9),
	ImagenesProducto char(9) not null,
	ImagenesEtiqueta varchar(40) not null,
	ImagenesUrl varchar(300) not null,
	ImagenesOrden integer not null Default 0,
	ImagenesEstado char(1) not null Default 'A',
	constraint ImagenesPK primary key (ImagenesCodigo),
	constraint ImagenesProductosFK foreign key (ImagenesProducto)
		references Catalogo.Productos (ProductosCodigo),
	constraint ImagenesOrdenUK unique (ImagenesProducto, ImagenesOrden),
	constraint ImagenesEstadoCK check (ImagenesEstado in ('A', 'E'))
	);
Create index if not exists ImagenesProductoIX on Catalogo.Imagenes (ImagenesProducto);
