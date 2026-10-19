// Módulo del agente conversacional. Cada archivo tiene un trabajo:
pub mod flujo;           // registro de clientes y conversación libre: qué responde el agente en cada paso
pub mod flujo_servicios; // consulta de servicios técnicos: lista, elegir servicio, elegir equipo, detalle
pub mod modelo;          // las estructuras de datos del módulo (Conversacion, Datos, Componente)
pub mod motor;           // entiende lo que escribe la persona (intención, elegir una opción). No toca la base de datos
pub mod pasos;           // los nombres de los pasos de la conversación (el "mapa" del agente)
pub mod repositorio;     // TODAS las consultas SQL del agente viven aquí
pub mod respuesta;       // las piezas comunes para armar respuestas (Respuesta, botones del menú)
pub mod rutas;           // las "puertas" del chat: reciben y devuelven JSON
