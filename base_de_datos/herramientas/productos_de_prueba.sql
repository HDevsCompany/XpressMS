-- productos_de_prueba.sql
-- SOLO PARA DESARROLLO. Llena el catálogo con 3 categorías y 8 productos de ejemplo (modelos reales, nombres y descripciones
-- en MAYÚSCULAS) para ver la vitrina funcionando.
-- Es seguro ejecutarlo más de una vez: no repite lo que ya existe (compara por nombre entre los activos). Si ya habías
-- ejecutado la versión anterior (nombres como «Laptop 15.6" Ryzen 5»), esos 8 productos se ACTUALIZAN a los nuevos datos
-- en lugar de duplicarse.
-- Los productos reales los carga el personal desde el panel; borra estos desde allí cuando ya no los necesites.
-- Ejecutar en el SQL Editor de Neon (base XpressMS), todo el archivo de una vez.
--
-- Para saber si el catálogo está vacío (por eso Juno dice «Por ahora no tengo productos para mostrarte»):
--   select count(*) from Catalogo.Productos where ProductosEstado = 'A';

do $$
declare
	alfabeto text := 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789'; -- el mismo alfabeto de los códigos de la plataforma
	nuevo text;
	codigo_categoria char(9);
	fila record;
begin
	-- Categorías
	for fila in select * from (values ('Laptops'), ('Repuestos'), ('Accesorios')) v(nombre) loop
		if not exists (select 1 from Catalogo.Categorias
		               where lower(CategoriasNombre) = lower(fila.nombre) and CategoriasEstado = 'A') then
			nuevo := '';
			for i in 1..9 loop
				nuevo := nuevo || substr(alfabeto, 1 + floor(random() * 32)::int, 1);
			end loop;
			insert into Catalogo.Categorias (CategoriasCodigo, CategoriasNombre) values (nuevo, fila.nombre);
		end if;
	end loop;

	-- Productos (la última columna es el nombre de la versión anterior de este archivo, para actualizarlo si ya existe)
	for fila in
		select * from (values
			('Laptops',    'LAPTOP LENOVO V15 G4 AMN RYZEN 5',          'RYZEN 5 7520U - 8GB RAM - 256GB SSD NVME',                  1899.00, 6,  'Laptop 15.6" Ryzen 5'),
			('Laptops',    'ASUS VIVOBOOK 15 X1502ZA INTEL CORE I5',    'CORE I5 1235U - 16GB RAM - 512GB SSD',                      2450.00, 2,  'Laptop 14" Core i5'),
			('Repuestos',  'DISCO DURO SSD KINGSTON NV2 500GB',         'M.2 2280 NVME PCIE 4.0 - HASTA 3500 MB/S',                   129.00, 14, 'SSD NVMe 500 GB'),
			('Repuestos',  'MEMORIA RAM SODIMM DDR4 8GB 3200MHZ',       'CL22 PARA LAPTOP - KINGSTON FURY / CRUCIAL',                  89.00, 9,  'Memoria RAM 8 GB DDR4'),
			('Repuestos',  'PANTALLA LED 15.6" FHD SLIM 30 PINES',      'CONECTOR INFERIOR DERECHO - TIPO IPS',                       259.00, 0,  'Pantalla 15.6" FHD'),
			('Repuestos',  'BATERÍA COMPATIBLE ASUS A41N1603',          'LI-ION 14.4V 48W - ORIGINAL GRADE A',                        149.00, 3,  'Batería para laptop'),
			('Accesorios', 'MOUSE INALÁMBRICO LOGITECH M170',           '2.4 GHZ - 1000 DPI - PILA AA INCLUIDA',                       39.00, 25, 'Mouse inalámbrico M170'),
			('Accesorios', 'TECLADO EN ESPAÑOL USB MULTIMEDIA',         'DISEÑO RESISTENTE A SALPICADURAS - PLUG & PLAY',              55.00, 17, 'Teclado USB español')
		) v(categoria, nombre, descripcion, precio, stock, nombre_anterior)
	loop
		if exists (select 1 from Catalogo.Productos
		           where lower(ProductosNombre) = lower(fila.nombre) and ProductosEstado = 'A') then
			null; -- ya está con los datos nuevos
		elsif exists (select 1 from Catalogo.Productos
		              where lower(ProductosNombre) = lower(fila.nombre_anterior) and ProductosEstado = 'A') then
			update Catalogo.Productos
				set ProductosNombre = fila.nombre, ProductosDescripcion = fila.descripcion,
				    ProductosPrecio = fila.precio, ProductosStock = fila.stock
				where lower(ProductosNombre) = lower(fila.nombre_anterior) and ProductosEstado = 'A';
		else
			select CategoriasCodigo into codigo_categoria from Catalogo.Categorias
				where lower(CategoriasNombre) = lower(fila.categoria) and CategoriasEstado = 'A';
			nuevo := '';
			for i in 1..9 loop
				nuevo := nuevo || substr(alfabeto, 1 + floor(random() * 32)::int, 1);
			end loop;
			insert into Catalogo.Productos
				(ProductosCodigo, ProductosCategoria, ProductosNombre, ProductosDescripcion, ProductosPrecio, ProductosStock)
				values (nuevo, codigo_categoria, fila.nombre, fila.descripcion, fila.precio, fila.stock);
		end if;
	end loop;
end $$;

-- Para comprobar: debe mostrar los productos activos.
select c.CategoriasNombre as categoria, p.ProductosNombre as producto, p.ProductosPrecio as precio, p.ProductosStock as stock
	from Catalogo.Productos p
	join Catalogo.Categorias c on c.CategoriasCodigo = p.ProductosCategoria
	where p.ProductosEstado = 'A'
	order by c.CategoriasNombre, p.ProductosNombre;
