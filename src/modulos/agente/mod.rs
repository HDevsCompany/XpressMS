// Módulo del agente conversacional. Cada archivo tiene un trabajo:
pub mod flujo;           // registro de clientes y conversación libre: qué responde el agente en cada paso
pub mod flujo_carrito;   // carrito de la tienda: añadir, quitar, cambiar cantidades y apartar todo en un solo pedido
pub mod flujo_catalogo;  // catálogo público: la vitrina de productos que Juno filtra (por categoría, precio o palabras)
pub mod flujo_citas;     // citas y seguimiento: elegir día y hora, confirmar, ver la orden, cancelar
pub mod flujo_perfil;    // mi perfil: direcciones de entrega y recojo, personas autorizadas, y cambios que se hacen en tienda
pub mod flujo_productos; // productos y apartados: categorías, detalle, apartar, mis apartados, cancelar
pub mod flujo_servicios; // consulta de servicios técnicos: lista, elegir servicio, elegir equipo, detalle
pub mod flujo_solicitud; // solicitudes de /servicios: «Solicitar un servicio técnico» y «Cita de diagnóstico» (lo que muestra «Tu solicitud»)
pub mod identidad;       // cómo se presenta el agente: nombre e icono (Juno por defecto)
pub mod identificacion;  // reconocer a un cliente desde la página de ingreso (documento + 3 dígitos del celular)
pub mod limite_intentos; // freno a los intentos fallidos repetidos (por documento y por dirección de internet)
pub mod modelo;          // las estructuras de datos del módulo (Conversacion, Datos, Componente)
pub mod motor;           // entiende lo que escribe la persona (intención, elegir una opción). No toca la base de datos
pub mod panel_interprete; // el «oído» de Juno en el panel de servicios: lee precio, tiempo, equipos y nombre de lo que escribe el Administrador
pub mod panel_servicios;   // el cerebro de Juno en el panel de servicios: armar la ficha, editar y eliminar con confirmación
pub mod pasos;           // los nombres de los pasos de la conversación (el "mapa" del agente)
pub mod repositorio;     // TODAS las consultas SQL del agente viven aquí
pub mod respuesta;       // las piezas comunes para armar respuestas (Respuesta, botones del menú)
pub mod rutas;           // las "puertas" del chat: reciben y devuelven JSON
pub mod rutas_panel;     // la puerta de Juno en el panel de servicios: POST /panel/api/juno/servicios (solo Administrador)
