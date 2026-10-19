-- quitar_datos_ejemplo_servicio_tecnico.sql
-- Borra SOLO los datos de ejemplo (códigos que empiezan con "EJ-"). No toca los servicios reales.
-- El orden importa: primero la tabla que apunta a las otras (ServiciosEquipos), después las demás.

Delete from ServicioTecnico.ServiciosEquipos where ServiciosEquiposServicio like 'EJ-%';
Delete from ServicioTecnico.Servicios where ServiciosCodigo like 'EJ-%';
Delete from ServicioTecnico.TiposEquipo where TiposEquipoCodigo like 'EJ-%';
