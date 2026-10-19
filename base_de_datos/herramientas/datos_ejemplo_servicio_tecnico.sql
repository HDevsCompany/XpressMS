-- datos_ejemplo_servicio_tecnico.sql
-- ⚠ SOLO PARA PROBAR EL AGENTE. Los nombres, descripciones y PRECIOS son inventados (no son los de H&Devs).
-- ⚠ La base de Neon es la misma que usa la versión publicada en Render: mientras estos datos existan,
--   los clientes que usen el agente los verán. Quítalos con quitar_datos_ejemplo_servicio_tecnico.sql
--   y reemplázalos por los servicios reales (cuando exista la pantalla de gestión, o con inserts propios).
-- Los códigos empiezan con "EJ-" para poder reconocerlos y borrarlos fácilmente.
-- Requiere haber ejecutado antes 03_servicio_tecnico.sql.

Insert into ServicioTecnico.TiposEquipo (TiposEquipoCodigo, TiposEquipoNombre) values
	('EJ-EQ-LAP', 'Laptop'),
	('EJ-EQ-PCS', 'PC de escritorio'),
	('EJ-EQ-CMP', 'Componentes'),
	('EJ-EQ-IMP', 'Impresora');

Insert into ServicioTecnico.Servicios (ServiciosCodigo, ServiciosNombre, ServiciosDescripcion, ServiciosPrecioBase, ServiciosTiempoEstimado) values
	('EJ-SV-001', 'Formateo e instalación de Windows',
	 'Respaldo de tus archivos, instalación del sistema operativo, controladores y programas básicos.', 60.00, '1 día'),
	('EJ-SV-002', 'Mantenimiento preventivo',
	 'Limpieza interna, cambio de pasta térmica y revisión general del equipo.', 40.00, 'Mismo día'),
	('EJ-SV-003', 'Limpieza de cabezales',
	 'Limpieza y calibración del cabezal de impresión para corregir líneas y manchas.', 35.00, 'Mismo día'),
	('EJ-SV-004', 'Diagnóstico de fallas',
	 'Revisión completa para encontrar la causa del problema y cotizar la reparación.', NULL, '1 a 2 días');

Insert into ServicioTecnico.ServiciosEquipos (ServiciosEquiposServicio, ServiciosEquiposTipoEquipo) values
	('EJ-SV-001', 'EJ-EQ-LAP'), ('EJ-SV-001', 'EJ-EQ-PCS'),
	('EJ-SV-002', 'EJ-EQ-LAP'), ('EJ-SV-002', 'EJ-EQ-PCS'),
	('EJ-SV-003', 'EJ-EQ-IMP'),
	('EJ-SV-004', 'EJ-EQ-LAP'), ('EJ-SV-004', 'EJ-EQ-PCS'), ('EJ-SV-004', 'EJ-EQ-CMP'), ('EJ-SV-004', 'EJ-EQ-IMP');
