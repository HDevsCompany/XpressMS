-- crear_usuario_peter.sql
-- Crea el usuario «peter» (Administrador) para el equipo de H&Devs.
--   Usuario: peter        Clave temporal: Peter12345678  (se guarda cifrada con Argon2; el sistema le pedirá cambiarla en su primer ingreso)
-- El empleado lleva datos de prueba (PETER PRUEBA, documento ficticio 70000001): edítalos luego en Panel -> Personal.
-- Es seguro ejecutarlo más de una vez: si «peter» ya existe, no hace nada.
-- Ejecutar en el SQL Editor de Neon (base XpressMS), todo el archivo de una vez.

do $$
begin
	if not exists (select 1 from Acceso.Usuarios where UsuariosNombreUsuario = 'peter') then
		insert into Acceso.Empleados (EmpleadosCodigo, EmpleadosTipoDocumento, EmpleadosNumeroDocumento, EmpleadosPaterno, EmpleadosNombres)
			values ('ZHNBC77HG', '1', '70000001', 'PRUEBA', 'PETER');
		insert into Acceso.Usuarios (UsuariosCodigo, UsuariosEmpleado, UsuariosNombreUsuario, UsuariosClaveHash, UsuariosDebeCambiarClave)
			values ('9CXE5T8X6', 'ZHNBC77HG', 'peter', '$argon2id$v=19$m=19456,t=2,p=1$4Gak9u8mNA0Ab4r/lDkXNw$IRA6UVwnp1l6cuC7oIVhtR097Jrae4lZlchUv0sdsNE', true);
		insert into Acceso.UsuariosRoles (UsuariosRolesUsuario, UsuariosRolesRol) values ('9CXE5T8X6', 'ADM');
	end if;
end $$;

-- Comprobación: debe mostrar una fila con el usuario peter y el rol ADM.
select u.UsuariosNombreUsuario, e.EmpleadosNombres, e.EmpleadosPaterno, ur.UsuariosRolesRol, u.UsuariosDebeCambiarClave
from Acceso.Usuarios u
join Acceso.Empleados e on e.EmpleadosCodigo = u.UsuariosEmpleado
join Acceso.UsuariosRoles ur on ur.UsuariosRolesUsuario = u.UsuariosCodigo
where u.UsuariosNombreUsuario = 'peter';
