-- 12_servicios_y_diagnostico.sql
-- Crea el servicio «Diagnóstico de equipo», con el que la página /servicios agenda la «Cita de diagnóstico»: el cliente que NO sabe
-- qué tiene su equipo lo lleva al local, lo revisan y recibe el diagnóstico. Es una orden normal (aparece en el panel de órdenes
-- como cualquier otra, con el servicio «Diagnóstico de equipo»).
--
-- Su código es fijo (DIAGNOSTI): así el sistema lo reconoce aunque el personal le cambie el nombre desde el panel. No lo elimines
-- desde el panel: sin él, la tarjeta «Agendar cita de diagnóstico» de /servicios aparece como no disponible.
-- El precio base y el tiempo estimado quedan vacíos a propósito (se muestra «A cotizar»): los define el negocio desde el panel
-- (Servicios -> Diagnóstico de equipo). Aquí no se inventan.
--
-- Es seguro ejecutarlo más de una vez. Ejecutar en el SQL Editor de Neon (base XpressMS), todo el archivo de una vez.
-- Si ya tienes tipos de equipo (Laptop, PC...), el diagnóstico se asigna a todos los activos; si agregas otro tipo después, asígnalo
-- desde el panel (Servicios -> Diagnóstico de equipo).

do $$
begin
	if not exists (select 1 from ServicioTecnico.Servicios where ServiciosCodigo = 'DIAGNOSTI') then
		if exists (select 1 from ServicioTecnico.Servicios
		           where lower(ServiciosNombre) = lower('Diagnóstico de equipo') and ServiciosEstado = 'A') then
			raise exception 'Ya existe un servicio activo llamado «Diagnóstico de equipo» con otro código. Cámbiale el nombre desde el panel y vuelve a ejecutar este archivo.';
		end if;
		insert into ServicioTecnico.Servicios (ServiciosCodigo, ServiciosNombre, ServiciosDescripcion)
		values ('DIAGNOSTI', 'Diagnóstico de equipo',
		        'Revisamos tu equipo en el local y te damos el diagnóstico de lo que tiene, para que decidas cómo seguir.');
	end if;

	insert into ServicioTecnico.ServiciosEquipos (ServiciosEquiposServicio, ServiciosEquiposTipoEquipo)
	select 'DIAGNOSTI', TiposEquipoCodigo
	from ServicioTecnico.TiposEquipo
	where TiposEquipoEstado = 'A'
	on conflict do nothing;
end $$;

-- Comprobación: debe mostrar una fila con el servicio y sus equipos.
select s.ServiciosCodigo, s.ServiciosNombre, s.ServiciosEstado,
       (select string_agg(t.TiposEquipoNombre, ', ' order by t.TiposEquipoNombre)
        from ServicioTecnico.ServiciosEquipos se
        join ServicioTecnico.TiposEquipo t on t.TiposEquipoCodigo = se.ServiciosEquiposTipoEquipo
        where se.ServiciosEquiposServicio = s.ServiciosCodigo) as equipos
from ServicioTecnico.Servicios s
where s.ServiciosCodigo = 'DIAGNOSTI';
