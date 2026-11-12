-- reiniciar_base.sql
-- SOLO PARA DESARROLLO. Deja la base como recién instalada: borra TODOS los datos escritos
-- (clientes, conversaciones, mensajes, códigos de verificación del celular, citas/órdenes, apartados, personal, usuarios, sesiones,
-- servicios, tipos de equipo, categorías y productos).
-- NO toca las listas fijas del sistema (se cargan con los scripts 01 a 08 y la plataforma las necesita):
--   Comun.TiposDocumento, Acceso.Roles, ServicioTecnico.EstadosOrden, Catalogo.EstadosReserva.
--
-- Requiere haber ejecutado 09_verificacion_telefono.sql (si aún no lo hiciste, quita de abajo las líneas de Acceso.VerificacionesTelefono).
-- ⚠ La base de Neon es la misma que usa la versión publicada en Render: también se vaciará allí.
-- ⚠ NUNCA ejecutes esto cuando ya existan clientes reales. No se puede deshacer
--   (si quieres una red de seguridad, crea antes una rama en Neon: Branches > Create branch).
--
-- Cómo usarlo: Neon > tu proyecto > SQL Editor (base XpressMS) > pega TODO este archivo > Run.
-- Después, crea tu administrador desde tu computadora:  cargo run -- crear-admin

begin;

-- Se listan todas juntas porque están relacionadas entre sí por claves foráneas.
-- (Sin CASCADE a propósito: si faltara alguna tabla relacionada, PostgreSQL avisa en vez de borrar de más.)
truncate
	Agente.Mensajes,
	Agente.Conversaciones,
	Acceso.VerificacionesTelefono,
	Acceso.Sesiones,
	Acceso.UsuariosRoles,
	Acceso.Usuarios,
	Acceso.Empleados,
	Acceso.Clientes,
	ServicioTecnico.OrdenesHistorial,
	ServicioTecnico.Ordenes,
	ServicioTecnico.ServiciosEquipos,
	ServicioTecnico.Servicios,
	ServicioTecnico.TiposEquipo,
	Catalogo.Reservas,
	Catalogo.Productos,
	Catalogo.Categorias;

commit;

-- Comprobación: las 16 tablas de arriba deben dar 0, y las 4 listas fijas deben conservar sus filas.
select 'Agente.Mensajes' as tabla, count(*) as filas from Agente.Mensajes
union all select 'Agente.Conversaciones', count(*) from Agente.Conversaciones
union all select 'Acceso.VerificacionesTelefono', count(*) from Acceso.VerificacionesTelefono
union all select 'Acceso.Sesiones', count(*) from Acceso.Sesiones
union all select 'Acceso.UsuariosRoles', count(*) from Acceso.UsuariosRoles
union all select 'Acceso.Usuarios', count(*) from Acceso.Usuarios
union all select 'Acceso.Empleados', count(*) from Acceso.Empleados
union all select 'Acceso.Clientes', count(*) from Acceso.Clientes
union all select 'ServicioTecnico.OrdenesHistorial', count(*) from ServicioTecnico.OrdenesHistorial
union all select 'ServicioTecnico.Ordenes', count(*) from ServicioTecnico.Ordenes
union all select 'ServicioTecnico.ServiciosEquipos', count(*) from ServicioTecnico.ServiciosEquipos
union all select 'ServicioTecnico.Servicios', count(*) from ServicioTecnico.Servicios
union all select 'ServicioTecnico.TiposEquipo', count(*) from ServicioTecnico.TiposEquipo
union all select 'Catalogo.Reservas', count(*) from Catalogo.Reservas
union all select 'Catalogo.Productos', count(*) from Catalogo.Productos
union all select 'Catalogo.Categorias', count(*) from Catalogo.Categorias
union all select '(fija) Comun.TiposDocumento', count(*) from Comun.TiposDocumento
union all select '(fija) Acceso.Roles', count(*) from Acceso.Roles
union all select '(fija) ServicioTecnico.EstadosOrden', count(*) from ServicioTecnico.EstadosOrden
union all select '(fija) Catalogo.EstadosReserva', count(*) from Catalogo.EstadosReserva;
