-- servicios_de_prueba.sql
-- SOLO PARA DESARROLLO. Llena la página /servicios con 2 tipos de equipo (Laptop y PC) y 6 servicios de ejemplo para probarla como
-- si fueras un cliente. LOS PRECIOS Y LOS TIEMPOS SON INVENTADOS (no son los de H&Devs): los reales los carga el personal desde el
-- panel (Servicios), y estos se pueden borrar desde allí cuando ya no hagan falta.
-- Es seguro ejecutarlo más de una vez: no repite lo que ya existe (compara por nombre entre los activos).
-- Ejecuta ANTES 12_servicios_y_diagnostico.sql si quieres que la cita de diagnóstico también tenga estos equipos (o ejecútalo después:
-- el diagnóstico se asigna a los tipos de equipo que existan en ese momento).
-- Ejecutar en el SQL Editor de Neon (base XpressMS), todo el archivo de una vez.
--
-- Para saber qué servicios hay ahora mismo:
--   select ServiciosNombre, ServiciosPrecioBase, ServiciosTiempoEstimado from ServicioTecnico.Servicios where ServiciosEstado = 'A';

do $$
declare
	alfabeto text := 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789'; -- el mismo alfabeto de los códigos de la plataforma
	nuevo text;
	codigo_servicio char(9);
	codigo_equipo char(9);
	fila record;
	equipo text;
begin
	-- Tipos de equipo
	for fila in select * from (values ('Laptop'), ('PC')) v(nombre) loop
		if not exists (select 1 from ServicioTecnico.TiposEquipo
		               where lower(TiposEquipoNombre) = lower(fila.nombre) and TiposEquipoEstado = 'A') then
			nuevo := '';
			for i in 1..9 loop
				nuevo := nuevo || substr(alfabeto, 1 + floor(random() * 32)::int, 1);
			end loop;
			insert into ServicioTecnico.TiposEquipo (TiposEquipoCodigo, TiposEquipoNombre) values (nuevo, fila.nombre);
		end if;
	end loop;

	-- Servicios: nombre, descripción, precio base (vacío = a cotizar), tiempo estimado y equipos (separados por coma)
	for fila in
		select * from (values
			('Formateo e instalación de Windows', 'Respaldo básico, instalación limpia de Windows, controladores y programas esenciales.', 60.00::numeric,  '1 día',       'Laptop,PC'),
			('Cambio de pantalla',                'Reemplazo de pantalla rota o con fallas de imagen. Repuesto compatible según el modelo.', 150.00::numeric, '1 a 2 días',  'Laptop'),
			('Cambio de teclado',                 'Cambio de teclado con teclas que no responden, líquidos derramados o desgaste.',          80.00::numeric,  '1 día',       'Laptop'),
			('Cambio de batería',                 'Batería que dura poco o no carga. Se verifica el cargador y se instala la nueva.',        null::numeric,   '1 día',       'Laptop'),
			('Mantenimiento preventivo',          'Limpieza interna, cambio de pasta térmica y revisión general para que rinda como nuevo.', 50.00::numeric,  '1 día',       'Laptop,PC'),
			('Reparación de placa madre',         'Diagnóstico y reparación de fallas en la placa: no enciende, corto, sin carga.',          null::numeric,   '3 a 5 días',  'Laptop,PC')
		) v(nombre, descripcion, precio, tiempo, equipos)
	loop
		if not exists (select 1 from ServicioTecnico.Servicios
		               where lower(ServiciosNombre) = lower(fila.nombre) and ServiciosEstado = 'A') then
			nuevo := '';
			for i in 1..9 loop
				nuevo := nuevo || substr(alfabeto, 1 + floor(random() * 32)::int, 1);
			end loop;
			codigo_servicio := nuevo;
			insert into ServicioTecnico.Servicios
				(ServiciosCodigo, ServiciosNombre, ServiciosDescripcion, ServiciosPrecioBase, ServiciosTiempoEstimado)
			values (codigo_servicio, fila.nombre, fila.descripcion, fila.precio, fila.tiempo);

			foreach equipo in array string_to_array(fila.equipos, ',') loop
				select TiposEquipoCodigo into codigo_equipo from ServicioTecnico.TiposEquipo
				where lower(TiposEquipoNombre) = lower(equipo) and TiposEquipoEstado = 'A' limit 1;
				if codigo_equipo is not null then
					insert into ServicioTecnico.ServiciosEquipos (ServiciosEquiposServicio, ServiciosEquiposTipoEquipo)
					values (codigo_servicio, codigo_equipo) on conflict do nothing;
				end if;
			end loop;
		end if;
	end loop;
end $$;

-- Comprobación: los servicios activos con sus equipos.
select s.ServiciosNombre, s.ServiciosPrecioBase, s.ServiciosTiempoEstimado,
       (select string_agg(t.TiposEquipoNombre, ', ' order by t.TiposEquipoNombre)
        from ServicioTecnico.ServiciosEquipos se
        join ServicioTecnico.TiposEquipo t on t.TiposEquipoCodigo = se.ServiciosEquiposTipoEquipo
        where se.ServiciosEquiposServicio = s.ServiciosCodigo) as equipos
from ServicioTecnico.Servicios s
where s.ServiciosEstado = 'A'
order by s.ServiciosNombre;
