-- 13_servicios_imagen_y_aviso.sql
-- Dos datos nuevos para cada servicio técnico, que el Administrador maneja desde el panel (Servicios → Juno):
--   1. La IMAGEN del servicio (opcional). Se guarda en la propia base (no en una carpeta del servidor) porque Render borra los archivos
--      del disco cada vez que reinicia; con la imagen en la base, nunca se pierde. El navegador la reduce antes de subirla, así que
--      pesa poco (la página limita cada imagen a 1.5 MB).
--   2. El AVISO «El precio puede cambiar»: si está activado, la ficha del servicio le dice al cliente que el costo final se confirma
--      después de revisar su equipo. Los servicios que ya existían quedan con el aviso ACTIVADO (así nada cambia para tus clientes);
--      los nuevos nacen con el aviso apagado y se enciende desde el panel.
--
-- Es seguro ejecutarlo más de una vez. Ejecutar en el SQL Editor de Neon (base XpressMS), todo el archivo de una vez.

alter table ServicioTecnico.Servicios add column if not exists ServiciosImagen bytea;
alter table ServicioTecnico.Servicios add column if not exists ServiciosImagenTipo text;
alter table ServicioTecnico.Servicios add column if not exists ServiciosAvisoPrecio boolean not null default true;

-- Desde ahora, los servicios nuevos nacen con el aviso apagado (los que ya existían conservan el aviso activado).
alter table ServicioTecnico.Servicios alter column ServiciosAvisoPrecio set default false;

-- Solo se aceptan los formatos que el navegador sabe mostrar.
do $$
begin
	if not exists (select 1 from pg_constraint where conname = 'servicios_imagen_tipo_valido') then
		alter table ServicioTecnico.Servicios
			add constraint servicios_imagen_tipo_valido
			check (ServiciosImagenTipo is null or ServiciosImagenTipo in ('image/jpeg', 'image/png', 'image/webp'));
	end if;
end $$;

-- Comprobación: debe mostrar las tres columnas nuevas.
select column_name, data_type, column_default
from information_schema.columns
where table_schema = 'serviciotecnico' and table_name = 'servicios'
  and column_name in ('serviciosimagen', 'serviciosimagentipo', 'serviciosavisoprecio')
order by column_name;
