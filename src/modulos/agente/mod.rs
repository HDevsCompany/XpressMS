// Módulo del agente conversacional. Cada archivo tiene un trabajo:
pub mod flujo;       // qué responde el agente y en qué paso va la conversación
pub mod modelo;      // las estructuras de datos del módulo (Conversacion, MensajeGuardado)
pub mod motor;       // entiende lo que escribe la persona (intención). No toca la base de datos
pub mod repositorio; // TODAS las consultas SQL del agente viven aquí
pub mod rutas;       // las "puertas" del chat: reciben y devuelven JSON
