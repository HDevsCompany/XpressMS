// Módulo "perfil": «Mi perfil y datos» del cliente. Muestra lo que guardamos de él (datos, contacto, privacidad),
// sus direcciones de entrega y recojo y las personas autorizadas a recoger sus equipos.
// Regla de la plataforma: la página solo MUESTRA. Todo cambio (añadir una dirección, autorizar a alguien...) lo hace Juno
// en el chat (agente/flujo_perfil.rs), que usa las consultas de repositorio.rs.
pub mod modelo;      // las estructuras de datos del perfil (lo que viaja a la página)
pub mod repositorio; // TODAS las consultas SQL de direcciones y personas autorizadas
pub mod rutas;       // POST /perfil/datos
