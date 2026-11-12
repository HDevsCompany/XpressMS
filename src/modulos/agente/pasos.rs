// pasos.rs: los nombres de los PASOS de la conversación (el "mapa" del agente).
// La conversación siempre está en uno de estos pasos (se guarda en la base de datos) y, según el paso,
// lo que escribe la persona significa una cosa u otra. Están aquí, juntos, para ver el mapa completo
// de un vistazo y no equivocarnos al escribirlos.

pub const LIBRE: &str = "libre"; // conversación abierta: la persona pregunta lo que quiera

// Registro de un cliente nuevo.
pub const CONSENTIMIENTO: &str = "consentimiento";
pub const TIPO_DOCUMENTO: &str = "tipo_documento";
pub const NUMERO_DOCUMENTO: &str = "numero_documento";
pub const VERIFICAR_TELEFONO: &str = "verificar_telefono";
pub const CONFIRMAR_IDENTIDAD: &str = "confirmar_identidad";
pub const MANUAL_NOMBRES: &str = "manual_nombres";
pub const MANUAL_PATERNO: &str = "manual_paterno";
pub const MANUAL_MATERNO: &str = "manual_materno";
pub const MANUAL_RAZON_SOCIAL: &str = "manual_razon_social";
pub const DIRECCION: &str = "direccion";
pub const SEXO: &str = "sexo"; // opcional: solo si la consulta del documento no lo trajo
pub const NACIMIENTO: &str = "nacimiento"; // opcional: solo si la consulta del documento no lo trajo
pub const TELEFONO: &str = "telefono";
pub const VERIFICAR_CELULAR: &str = "verificar_celular"; // escribe el código de 6 dígitos que le llegó por SMS
pub const CORREO: &str = "correo";
pub const AGREGAR_CORREO: &str = "agregar_correo"; // un cliente ya registrado agrega su correo después

// Consulta de servicios técnicos.
pub const ELEGIR_SERVICIO: &str = "elegir_servicio"; // la persona escoge un servicio de la lista
pub const ELEGIR_EQUIPO: &str = "elegir_equipo"; // la persona escoge para qué equipo lo necesita

// Citas y seguimiento de órdenes (flujo_citas.rs).
pub const CITA_FECHA: &str = "cita_fecha"; // elige el día
pub const CITA_HORA: &str = "cita_hora"; // elige la hora
pub const CITA_PROBLEMA: &str = "cita_problema"; // cuenta qué le pasa al equipo
pub const CITA_CONFIRMAR: &str = "cita_confirmar"; // confirma, cambia la fecha o cancela
pub const ELEGIR_ORDEN: &str = "elegir_orden"; // tiene varias órdenes y elige cuál ver
pub const CANCELAR_CITA: &str = "cancelar_cita"; // confirma que quiere cancelar su cita

// Catálogo público (flujo_catalogo.rs): la vitrina de productos que Juno filtra a pedido de la persona.
pub const CATALOGO: &str = "catalogo"; // la vitrina está abierta: lo que escribe la persona afina la búsqueda

// Productos y apartados (flujo_productos.rs).
pub const ELEGIR_CATEGORIA: &str = "elegir_categoria"; // elige una categoría del catálogo
pub const ELEGIR_PRODUCTO: &str = "elegir_producto"; // elige un producto de la categoría
pub const APARTAR_CANTIDAD: &str = "apartar_cantidad"; // elige cuántas unidades quiere apartar
pub const APARTAR_CONFIRMAR: &str = "apartar_confirmar"; // confirma, cambia la cantidad o cancela
pub const ELEGIR_APARTADO: &str = "elegir_apartado"; // tiene varios apartados y elige cuál ver
pub const CANCELAR_APARTADO: &str = "cancelar_apartado"; // confirma que quiere cancelar su apartado

// Mi perfil (flujo_perfil.rs): direcciones de entrega y recojo, y personas autorizadas a recoger.
pub const DIR_ACCION: &str = "dir_accion"; // añadir, hacer principal o quitar una dirección
pub const DIR_CUAL: &str = "dir_cual"; // elige cuál dirección (hacer principal / quitar)
pub const DIR_TIPO: &str = "dir_tipo"; // casa, oficina u otra
pub const DIR_NOMBRE: &str = "dir_nombre"; // el nombre de una dirección «otra»
pub const DIR_TEXTO: &str = "dir_texto"; // escribe la dirección
pub const DIR_REFERENCIA: &str = "dir_referencia"; // referencia opcional («frente al parque»)
pub const DIR_PRINCIPAL: &str = "dir_principal"; // ¿la deja como principal?
pub const DIR_QUITAR_CONFIRMAR: &str = "dir_quitar_confirmar"; // confirma que quiere quitarla
pub const PER_ACCION: &str = "per_accion"; // autorizar o quitar a una persona
pub const PER_NOMBRE: &str = "per_nombre"; // nombre de la persona autorizada
pub const PER_DOCUMENTO: &str = "per_documento"; // DNI de la persona autorizada
pub const PER_PARENTESCO: &str = "per_parentesco"; // qué es de la persona
pub const PER_CONFIRMAR: &str = "per_confirmar"; // revisa y pide el código por SMS
pub const PER_CODIGO: &str = "per_codigo"; // escribe el código que le llegó por SMS
pub const PER_QUITAR_CUAL: &str = "per_quitar_cual"; // elige a quién quitar
pub const PER_QUITAR_CONFIRMAR: &str = "per_quitar_confirmar"; // confirma que la quita
