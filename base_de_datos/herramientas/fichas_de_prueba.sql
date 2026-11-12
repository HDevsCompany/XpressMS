-- fichas_de_prueba.sql
-- SOLO PARA DESARROLLO. Carga las características (cajitas con ícono y especificaciones) y las vistas de los 8 productos de
-- ejemplo de productos_de_prueba.sql. Los datos salen de sus nombres y descripciones: no se inventó nada más.
-- Las vistas «dibujo:...» son dibujos de prueba (no fotos). Solo la laptop, el mouse y el SSD las tienen: los demás
-- productos no muestran galería hasta que se carguen fotos reales.
-- Es seguro ejecutarlo más de una vez: actualiza lo que ya existe. Ejecutar DESPUÉS de 11_pedidos_y_fichas.sql y de
-- productos_de_prueba.sql, en el SQL Editor de Neon, todo el archivo de una vez.

do $$
declare
	alfabeto text := 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
	nuevo text;
	fila record;
	codigo_producto char(9);
begin
	-- Características: (producto, grupo, título, valor, ícono o nulo, orden). El grupo RESUMEN son las cajas destacadas.
	for fila in
		select * from (values
			('LAPTOP LENOVO V15 G4 AMN RYZEN 5', 'RESUMEN', 'PROCESADOR', 'RYZEN 5 7520U', 'cpu', 1),
			('LAPTOP LENOVO V15 G4 AMN RYZEN 5', 'RESUMEN', 'MEMORIA RAM', '8 GB', 'ram', 2),
			('LAPTOP LENOVO V15 G4 AMN RYZEN 5', 'RESUMEN', 'ALMACENAMIENTO', '256 GB SSD NVME', 'disco', 3),
			('LAPTOP LENOVO V15 G4 AMN RYZEN 5', 'RESUMEN', 'MARCA', 'LENOVO', 'marca', 4),
			('LAPTOP LENOVO V15 G4 AMN RYZEN 5', 'PROCESADOR', 'MODELO', 'RYZEN 5 7520U', null, 5),
			('LAPTOP LENOVO V15 G4 AMN RYZEN 5', 'MEMORIA', 'CAPACIDAD', '8 GB', null, 6),
			('LAPTOP LENOVO V15 G4 AMN RYZEN 5', 'ALMACENAMIENTO', 'CAPACIDAD', '256 GB', null, 7),
			('LAPTOP LENOVO V15 G4 AMN RYZEN 5', 'ALMACENAMIENTO', 'TIPO', 'SSD NVME', null, 8),
			('LAPTOP LENOVO V15 G4 AMN RYZEN 5', 'GENERAL', 'MARCA', 'LENOVO', null, 9),
			('LAPTOP LENOVO V15 G4 AMN RYZEN 5', 'GENERAL', 'MODELO', 'V15 G4 AMN', null, 10),
			('ASUS VIVOBOOK 15 X1502ZA INTEL CORE I5', 'RESUMEN', 'PROCESADOR', 'CORE I5 1235U', 'cpu', 1),
			('ASUS VIVOBOOK 15 X1502ZA INTEL CORE I5', 'RESUMEN', 'MEMORIA RAM', '16 GB', 'ram', 2),
			('ASUS VIVOBOOK 15 X1502ZA INTEL CORE I5', 'RESUMEN', 'ALMACENAMIENTO', '512 GB SSD', 'disco', 3),
			('ASUS VIVOBOOK 15 X1502ZA INTEL CORE I5', 'RESUMEN', 'MARCA', 'ASUS', 'marca', 4),
			('ASUS VIVOBOOK 15 X1502ZA INTEL CORE I5', 'PROCESADOR', 'MODELO', 'CORE I5 1235U', null, 5),
			('ASUS VIVOBOOK 15 X1502ZA INTEL CORE I5', 'MEMORIA', 'CAPACIDAD', '16 GB', null, 6),
			('ASUS VIVOBOOK 15 X1502ZA INTEL CORE I5', 'ALMACENAMIENTO', 'CAPACIDAD', '512 GB', null, 7),
			('ASUS VIVOBOOK 15 X1502ZA INTEL CORE I5', 'ALMACENAMIENTO', 'TIPO', 'SSD', null, 8),
			('ASUS VIVOBOOK 15 X1502ZA INTEL CORE I5', 'GENERAL', 'MARCA', 'ASUS', null, 9),
			('ASUS VIVOBOOK 15 X1502ZA INTEL CORE I5', 'GENERAL', 'MODELO', 'VIVOBOOK 15 X1502ZA', null, 10),
			('DISCO DURO SSD KINGSTON NV2 500GB', 'RESUMEN', 'CAPACIDAD', '500 GB', 'capacidad', 1),
			('DISCO DURO SSD KINGSTON NV2 500GB', 'RESUMEN', 'FORMATO', 'M.2 2280', 'ram', 2),
			('DISCO DURO SSD KINGSTON NV2 500GB', 'RESUMEN', 'INTERFAZ', 'NVME PCIE 4.0', 'usb', 3),
			('DISCO DURO SSD KINGSTON NV2 500GB', 'RESUMEN', 'VELOCIDAD', 'HASTA 3500 MB/S', 'rayo', 4),
			('DISCO DURO SSD KINGSTON NV2 500GB', 'RESUMEN', 'MARCA', 'KINGSTON', 'marca', 5),
			('DISCO DURO SSD KINGSTON NV2 500GB', 'RESUMEN', 'SERIE', 'NV2', 'disco', 6),
			('DISCO DURO SSD KINGSTON NV2 500GB', 'ALMACENAMIENTO', 'CAPACIDAD', '500 GB', null, 7),
			('DISCO DURO SSD KINGSTON NV2 500GB', 'ALMACENAMIENTO', 'FORMATO', 'M.2 2280', null, 8),
			('DISCO DURO SSD KINGSTON NV2 500GB', 'CONEXIÓN', 'INTERFAZ', 'NVME PCIE 4.0', null, 9),
			('DISCO DURO SSD KINGSTON NV2 500GB', 'RENDIMIENTO', 'VELOCIDAD', 'HASTA 3500 MB/S', null, 10),
			('DISCO DURO SSD KINGSTON NV2 500GB', 'GENERAL', 'MARCA', 'KINGSTON', null, 11),
			('DISCO DURO SSD KINGSTON NV2 500GB', 'GENERAL', 'SERIE', 'NV2', null, 12),
			('MEMORIA RAM SODIMM DDR4 8GB 3200MHZ', 'RESUMEN', 'TIPO', 'DDR4 SODIMM', 'ram', 1),
			('MEMORIA RAM SODIMM DDR4 8GB 3200MHZ', 'RESUMEN', 'CAPACIDAD', '8 GB', 'capacidad', 2),
			('MEMORIA RAM SODIMM DDR4 8GB 3200MHZ', 'RESUMEN', 'VELOCIDAD', '3200 MHZ', 'rayo', 3),
			('MEMORIA RAM SODIMM DDR4 8GB 3200MHZ', 'RESUMEN', 'LATENCIA', 'CL22', 'reloj', 4),
			('MEMORIA RAM SODIMM DDR4 8GB 3200MHZ', 'RESUMEN', 'USO', 'PARA LAPTOP', 'uso', 5),
			('MEMORIA RAM SODIMM DDR4 8GB 3200MHZ', 'RESUMEN', 'MARCA', 'KINGSTON FURY / CRUCIAL', 'marca', 6),
			('MEMORIA RAM SODIMM DDR4 8GB 3200MHZ', 'MEMORIA', 'TIPO', 'DDR4', null, 7),
			('MEMORIA RAM SODIMM DDR4 8GB 3200MHZ', 'MEMORIA', 'FORMATO', 'SODIMM', null, 8),
			('MEMORIA RAM SODIMM DDR4 8GB 3200MHZ', 'MEMORIA', 'CAPACIDAD', '8 GB', null, 9),
			('MEMORIA RAM SODIMM DDR4 8GB 3200MHZ', 'RENDIMIENTO', 'VELOCIDAD', '3200 MHZ', null, 10),
			('MEMORIA RAM SODIMM DDR4 8GB 3200MHZ', 'RENDIMIENTO', 'LATENCIA', 'CL22', null, 11),
			('MEMORIA RAM SODIMM DDR4 8GB 3200MHZ', 'GENERAL', 'USO', 'PARA LAPTOP', null, 12),
			('MEMORIA RAM SODIMM DDR4 8GB 3200MHZ', 'GENERAL', 'MARCA', 'KINGSTON FURY / CRUCIAL', null, 13),
			('PANTALLA LED 15.6" FHD SLIM 30 PINES', 'RESUMEN', 'TAMAÑO', '15.6"', 'pantalla', 1),
			('PANTALLA LED 15.6" FHD SLIM 30 PINES', 'RESUMEN', 'RESOLUCIÓN', 'FHD', 'resol', 2),
			('PANTALLA LED 15.6" FHD SLIM 30 PINES', 'RESUMEN', 'PANEL', 'IPS', 'panel', 3),
			('PANTALLA LED 15.6" FHD SLIM 30 PINES', 'RESUMEN', 'CONECTOR', '30 PINES', 'enchufe', 4),
			('PANTALLA LED 15.6" FHD SLIM 30 PINES', 'PANTALLA', 'TAMAÑO', '15.6"', null, 5),
			('PANTALLA LED 15.6" FHD SLIM 30 PINES', 'PANTALLA', 'RESOLUCIÓN', 'FHD', null, 6),
			('PANTALLA LED 15.6" FHD SLIM 30 PINES', 'PANTALLA', 'TIPO', 'IPS', null, 7),
			('PANTALLA LED 15.6" FHD SLIM 30 PINES', 'PANTALLA', 'DISEÑO', 'SLIM', null, 8),
			('PANTALLA LED 15.6" FHD SLIM 30 PINES', 'CONEXIÓN', 'CONECTOR', '30 PINES', null, 9),
			('PANTALLA LED 15.6" FHD SLIM 30 PINES', 'CONEXIÓN', 'POSICIÓN', 'INFERIOR DERECHO', null, 10),
			('BATERÍA COMPATIBLE ASUS A41N1603', 'RESUMEN', 'TECNOLOGÍA', 'LI-ION', 'bateria', 1),
			('BATERÍA COMPATIBLE ASUS A41N1603', 'RESUMEN', 'VOLTAJE', '14.4 V', 'rayo', 2),
			('BATERÍA COMPATIBLE ASUS A41N1603', 'RESUMEN', 'ENERGÍA', '48 W', 'capacidad', 3),
			('BATERÍA COMPATIBLE ASUS A41N1603', 'RESUMEN', 'CALIDAD', 'ORIGINAL GRADE A', 'calidad', 4),
			('BATERÍA COMPATIBLE ASUS A41N1603', 'RESUMEN', 'COMPATIBLE', 'ASUS', 'marca', 5),
			('BATERÍA COMPATIBLE ASUS A41N1603', 'RESUMEN', 'MODELO', 'A41N1603', 'disco', 6),
			('BATERÍA COMPATIBLE ASUS A41N1603', 'BATERÍA', 'TECNOLOGÍA', 'LI-ION', null, 7),
			('BATERÍA COMPATIBLE ASUS A41N1603', 'BATERÍA', 'VOLTAJE', '14.4 V', null, 8),
			('BATERÍA COMPATIBLE ASUS A41N1603', 'BATERÍA', 'ENERGÍA', '48 W', null, 9),
			('BATERÍA COMPATIBLE ASUS A41N1603', 'GENERAL', 'CALIDAD', 'ORIGINAL GRADE A', null, 10),
			('BATERÍA COMPATIBLE ASUS A41N1603', 'GENERAL', 'COMPATIBLE CON', 'ASUS', null, 11),
			('BATERÍA COMPATIBLE ASUS A41N1603', 'GENERAL', 'MODELO', 'A41N1603', null, 12),
			('MOUSE INALÁMBRICO LOGITECH M170', 'RESUMEN', 'CONEXIÓN', 'INALÁMBRICO 2.4 GHZ', 'wifi', 1),
			('MOUSE INALÁMBRICO LOGITECH M170', 'RESUMEN', 'SENSOR', '1000 DPI', 'sensor', 2),
			('MOUSE INALÁMBRICO LOGITECH M170', 'RESUMEN', 'ALIMENTACIÓN', 'PILA AA INCLUIDA', 'pila', 3),
			('MOUSE INALÁMBRICO LOGITECH M170', 'RESUMEN', 'MARCA', 'LOGITECH', 'marca', 4),
			('MOUSE INALÁMBRICO LOGITECH M170', 'CONEXIÓN', 'TIPO', 'INALÁMBRICO', null, 5),
			('MOUSE INALÁMBRICO LOGITECH M170', 'CONEXIÓN', 'FRECUENCIA', '2.4 GHZ', null, 6),
			('MOUSE INALÁMBRICO LOGITECH M170', 'SENSOR', 'RESOLUCIÓN', '1000 DPI', null, 7),
			('MOUSE INALÁMBRICO LOGITECH M170', 'ENERGÍA', 'ALIMENTACIÓN', 'PILA AA', null, 8),
			('MOUSE INALÁMBRICO LOGITECH M170', 'ENERGÍA', 'PILA', 'INCLUIDA', null, 9),
			('MOUSE INALÁMBRICO LOGITECH M170', 'GENERAL', 'MARCA', 'LOGITECH', null, 10),
			('MOUSE INALÁMBRICO LOGITECH M170', 'GENERAL', 'MODELO', 'M170', null, 11),
			('TECLADO EN ESPAÑOL USB MULTIMEDIA', 'RESUMEN', 'INTERFAZ', 'USB', 'usb', 1),
			('TECLADO EN ESPAÑOL USB MULTIMEDIA', 'RESUMEN', 'IDIOMA', 'ESPAÑOL', 'idioma', 2),
			('TECLADO EN ESPAÑOL USB MULTIMEDIA', 'RESUMEN', 'DISEÑO', 'RESISTENTE A SALPICADURAS', 'escudo', 3),
			('TECLADO EN ESPAÑOL USB MULTIMEDIA', 'RESUMEN', 'INSTALACIÓN', 'PLUG & PLAY', 'enchufe', 4),
			('TECLADO EN ESPAÑOL USB MULTIMEDIA', 'CONEXIÓN', 'INTERFAZ', 'USB', null, 5),
			('TECLADO EN ESPAÑOL USB MULTIMEDIA', 'CONEXIÓN', 'INSTALACIÓN', 'PLUG & PLAY', null, 6),
			('TECLADO EN ESPAÑOL USB MULTIMEDIA', 'GENERAL', 'IDIOMA', 'ESPAÑOL', null, 7),
			('TECLADO EN ESPAÑOL USB MULTIMEDIA', 'GENERAL', 'TIPO', 'MULTIMEDIA', null, 8),
			('TECLADO EN ESPAÑOL USB MULTIMEDIA', 'GENERAL', 'DISEÑO', 'RESISTENTE A SALPICADURAS', null, 9)
		) v(producto, grupo, titulo, valor, icono, orden)
	loop
		select ProductosCodigo into codigo_producto from Catalogo.Productos
			where ProductosNombre = fila.producto and ProductosEstado = 'A';
		if codigo_producto is null then
			continue; -- ese producto no está cargado: se salta
		end if;
		if exists (select 1 from Catalogo.Caracteristicas
		           where CaracteristicasProducto = codigo_producto and CaracteristicasGrupo = fila.grupo
		             and CaracteristicasTitulo = fila.titulo) then
			update Catalogo.Caracteristicas
				set CaracteristicasValor = fila.valor, CaracteristicasIcono = fila.icono,
				    CaracteristicasOrden = fila.orden, CaracteristicasEstado = 'A'
				where CaracteristicasProducto = codigo_producto and CaracteristicasGrupo = fila.grupo
				  and CaracteristicasTitulo = fila.titulo;
		else
			nuevo := '';
			for i in 1..9 loop
				nuevo := nuevo || substr(alfabeto, 1 + floor(random() * 32)::int, 1);
			end loop;
			insert into Catalogo.Caracteristicas
				(CaracteristicasCodigo, CaracteristicasProducto, CaracteristicasGrupo, CaracteristicasTitulo,
				 CaracteristicasValor, CaracteristicasIcono, CaracteristicasOrden)
				values (nuevo, codigo_producto, fila.grupo, fila.titulo, fila.valor, fila.icono, fila.orden);
		end if;
	end loop;

	-- Vistas de prueba (dibujos) para la laptop, el mouse y el SSD: (producto, etiqueta, url, orden)
	for fila in
		select p.nombre, v.etiqueta, v.url, v.orden
		from (values
			('ASUS VIVOBOOK 15 X1502ZA INTEL CORE I5'),
			('LAPTOP LENOVO V15 G4 AMN RYZEN 5'),
			('MOUSE INALÁMBRICO LOGITECH M170'),
			('DISCO DURO SSD KINGSTON NV2 500GB')
		) p(nombre)
		cross join (values
			('VISTA FRONTAL', 'dibujo:frontal', 1),
			('VISTA LATERAL', 'dibujo:lateral', 2),
			('VISTA TRASERA', 'dibujo:trasera', 3),
			('EMPAQUE',       'dibujo:empaque', 4)
		) v(etiqueta, url, orden)
	loop
		select ProductosCodigo into codigo_producto from Catalogo.Productos
			where ProductosNombre = fila.nombre and ProductosEstado = 'A';
		if codigo_producto is null then
			continue;
		end if;
		if exists (select 1 from Catalogo.Imagenes where ImagenesProducto = codigo_producto and ImagenesOrden = fila.orden) then
			update Catalogo.Imagenes
				set ImagenesEtiqueta = fila.etiqueta, ImagenesUrl = fila.url, ImagenesEstado = 'A'
				where ImagenesProducto = codigo_producto and ImagenesOrden = fila.orden;
		else
			nuevo := '';
			for i in 1..9 loop
				nuevo := nuevo || substr(alfabeto, 1 + floor(random() * 32)::int, 1);
			end loop;
			insert into Catalogo.Imagenes (ImagenesCodigo, ImagenesProducto, ImagenesEtiqueta, ImagenesUrl, ImagenesOrden)
				values (nuevo, codigo_producto, fila.etiqueta, fila.url, fila.orden);
		end if;
	end loop;
end $$;

-- Comprobar: cuántas características e imágenes quedaron por producto.
-- select p.ProductosNombre, (select count(*) from Catalogo.Caracteristicas c where c.CaracteristicasProducto = p.ProductosCodigo) as caracteristicas,
--        (select count(*) from Catalogo.Imagenes i where i.ImagenesProducto = p.ProductosCodigo) as imagenes
-- from Catalogo.Productos p where p.ProductosEstado = 'A' order by 1;
