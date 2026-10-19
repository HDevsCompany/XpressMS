-- limpiar_pruebas.sql
-- SOLO PARA DESARROLLO. Sirve para borrar los clientes y conversaciones de prueba y poder volver a probar el registro.
-- NUNCA ejecutes la opción B cuando ya existan clientes reales: borra TODO.
-- Ejecutar en el SQL Editor de Neon (base XpressMS), una opción a la vez.

-- ---------------------------------------------------------------------------------------------
-- Opción A: borrar un solo cliente de prueba por su número de documento (cambia el número).
-- Primero se desvinculan sus conversaciones y luego se borra el cliente.
-- ---------------------------------------------------------------------------------------------
update Agente.Conversaciones
	set ConversacionesCliente = null
	where ConversacionesCliente in
		(select ClientesCodigo from Acceso.Clientes where ClientesNumeroDocumento = '72322732');

delete from Acceso.Clientes where ClientesNumeroDocumento = '72322732';

-- ---------------------------------------------------------------------------------------------
-- Opción B: dejar vacías las tres tablas (todos los clientes, conversaciones y mensajes).
-- Se listan juntas porque están relacionadas entre sí.
-- ---------------------------------------------------------------------------------------------
-- truncate Agente.Mensajes, Agente.Conversaciones, Acceso.Clientes;
