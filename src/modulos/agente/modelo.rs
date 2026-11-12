// modelo.rs: las estructuras que representan los datos del agente en Rust.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::modulos::servicio_tecnico::modelo::Hito;

// Una conversación guardada en Agente.Conversaciones.
// "FromRow" permite que sqlx llene esta estructura directamente con el resultado de una consulta.
// Los nombres de los campos deben coincidir con los alias ("as codigo") que se usan en el SQL.
#[derive(FromRow)]
pub struct Conversacion {
    pub codigo: String,          // código de 9 caracteres (clave primaria)
    pub token: String, // código secreto que guarda el navegador para reconocer la conversación
    pub paso: String,  // en qué punto va ('inicio', 'consentimiento', 'telefono'...)
    pub cliente: Option<String>, // código del cliente, si la persona ya se registró o se identificó
    pub datos: String, // lo recogido hasta ahora (JSON guardado como texto)
}

// Lo que el agente va recogiendo durante la conversación. Se guarda en la columna ConversacionesDatos.
// Los datos del registro se borran al terminar (los definitivos viven en Acceso.Clientes);
// el servicio y el equipo elegidos se conservan para la futura cita.
// "Default" crea una versión vacía; "serde(default)" permite leer JSON antiguos aunque falten campos.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Datos {
    pub consentimiento: Option<String>, // fecha y hora exactas en que aceptó el tratamiento de sus datos
    pub tipo_documento: Option<String>, // código SUNAT: "1", "6"...
    pub numero_documento: Option<String>,
    pub paterno: Option<String>,
    pub materno: Option<String>,
    pub nombres: Option<String>,
    pub razon_social: Option<String>,
    pub direccion: Option<String>,
    pub sexo: Option<String>,
    pub fecha_nacimiento: Option<String>,
    pub telefono: Option<String>,
    pub telefono_verificado: bool, // true cuando escribió bien el código que le llegó por SMS a ese celular
    pub correo: Option<String>,
    pub cliente_existente: Option<String>, // si el documento ya estaba registrado: su código
    pub intentos: u8,                      // intentos fallidos al verificar identidad
    pub servicio_elegido: Option<String>,  // código del servicio que la persona está viendo
    pub equipo_elegido: Option<String>,    // código del tipo de equipo elegido para ese servicio
    pub cita_fecha: Option<String>,        // día elegido para la cita: "2026-11-09"
    pub cita_hora: Option<u32>,            // hora elegida: 9, 10, 15... (las citas empiezan en punto)
    pub cita_problema: Option<String>,     // lo que la persona cuenta que le pasa al equipo
    pub orden_en_vista: Option<String>,    // código de la orden que la persona está mirando (para «Cancelar mi cita»)
    pub categoria_elegida: Option<String>, // código de la categoría de productos que está mirando
    pub producto_elegido: Option<String>,  // código del producto que está viendo (para «Apartar este producto»)
    pub apartar_cantidad: Option<i32>,     // unidades que quiere apartar
    pub apartado_en_vista: Option<String>, // código del apartado que está mirando (para «Cancelar mi apartado»)
    pub en_catalogo: bool, // la conversación es la de la ventana de Juno en la tienda (/catalogo): ahí «laptops» o «menos de 500» filtran la vitrina
    pub en_servicios: bool, // la conversación es la de la ventana de Juno en la página de servicios (/servicios): cada respuesta lleva el estado de «Tu solicitud»
    pub solicitud: Option<String>, // la solicitud en curso desde /servicios: «servicio» (ya sabe qué necesita) o «diagnostico» (no sabe qué tiene el equipo)
    // Catálogo público: los filtros que Juno aplicó a la vitrina de productos (se olvidan al volver a abrirla).
    pub catalogo_categoria: Option<String>, // código de la categoría que se está mostrando
    pub catalogo_precio_max: Option<u32>,   // «menos de S/ 500»: precio máximo en soles
    pub catalogo_busqueda: Option<String>,  // palabras que la persona pidió buscar («mouse inalámbrico»)
    pub solo_identificar: bool, // «Ya soy cliente»: se busca su documento pero NO se registra a nadie nuevo
    // El carrito de la tienda: lo que la persona va juntando para apartar en un solo pedido (lo guarda Juno, no la página).
    pub carrito: Vec<ItemCarrito>,
    // Mi perfil: lo que se va recogiendo al añadir una dirección o autorizar a una persona (se borra al terminar).
    pub dir_accion: Option<String>,      // «Añadir dirección», «Hacer principal», «Quitar dirección»
    pub dir_nombre: Option<String>,      // «Casa», «Oficina» o el nombre que escribió
    pub dir_texto: Option<String>,       // la dirección escrita
    pub dir_referencia: Option<String>,  // referencia opcional
    pub dir_objetivo: Option<String>,    // código de la dirección que se quiere cambiar o quitar
    pub per_nombre: Option<String>,      // nombre de la persona a autorizar
    pub per_documento: Option<String>,   // su DNI
    pub per_parentesco: Option<String>,  // qué es de la persona
    pub per_objetivo: Option<String>,    // código de la persona que se quiere quitar
}

impl Datos {
    // Borra lo recogido en un trámite (registro, cita...), pero NO el carrito de la tienda ni el hecho de que esta sea la
    // ventana de la tienda: esos pertenecen a la conversación, no al trámite que termina.
    // Tampoco la solicitud de servicio en curso (con el servicio que eligió): si la persona tuvo que registrarse o identificarse en
    // medio de ella, al terminar se sigue con lo que estaba pidiendo.
    pub fn reiniciar(&mut self) {
        let carrito = std::mem::take(&mut self.carrito);
        let en_catalogo = self.en_catalogo;
        let en_servicios = self.en_servicios;
        let solicitud = self.solicitud.take();
        let servicio_elegido = if solicitud.is_some() { self.servicio_elegido.take() } else { None };
        *self = Datos { carrito, en_catalogo, en_servicios, solicitud, servicio_elegido, ..Default::default() };
    }
}

// Un mensaje del chat, tal como se guarda en Agente.Mensajes.
// Además de FromRow, lleva "Serialize" porque se envía tal cual al navegador como JSON.
#[derive(FromRow, Serialize)]
pub struct MensajeGuardado {
    pub emisor: String, // 'P' = lo escribió la persona, 'A' = lo escribió el agente
    pub texto: String,  // el contenido del mensaje
}

// Estado de verificación de un dato de contacto (se dibuja como un icono junto al dato).
// "Verificado" se usa para el celular confirmado con el código por SMS; "Fallido" todavía no se usa.
#[allow(dead_code)]
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verificacion {
    Verificado, // check verde: la persona confirmó el dato
    Pendiente,  // reloj amarillo: aún no se confirma
    Fallido,    // x roja: no se pudo confirmar
}

// Una fila de la tarjeta de datos: "Nombres" -> "JUAN CARLOS".
// "verificacion" es opcional: solo los datos de contacto (celular) la llevan; si es None, no se envía en el JSON.
#[derive(Serialize)]
pub struct Campo {
    pub etiqueta: String,
    pub valor: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verificacion: Option<Verificacion>,
}

// Un producto dentro del carrito: cuál es (su código) y cuántas unidades.
#[derive(Serialize, Deserialize, Clone)]
pub struct ItemCarrito {
    pub producto: String, // código del producto
    pub cantidad: i32,
}

// Una «caja destacada» del resumen de la ficha: ícono, título pequeño y dato grande («PROCESADOR» · «CORE I5 1235U»).
#[derive(Serialize, Clone)]
pub struct Chip {
    pub icono: String,
    pub titulo: String,
    pub valor: String,
}

// Una fila de la tabla de especificaciones («CAPACIDAD» · «16 GB»).
#[derive(Serialize, Clone)]
pub struct FilaFicha {
    pub titulo: String,
    pub valor: String,
}

// Un grupo de la tabla de especificaciones («MEMORIA» con sus filas).
#[derive(Serialize, Clone)]
pub struct GrupoFicha {
    pub nombre: String,
    pub filas: Vec<FilaFicha>,
}

// Una vista de la galería: la etiqueta («VISTA LATERAL») y la imagen (una dirección o «dibujo:lateral»).
#[derive(Serialize, Clone)]
pub struct Vista {
    pub etiqueta: String,
    pub imagen: String,
}

// Un producto del carrito, ya con sus precios listos para mostrar.
#[derive(Serialize)]
pub struct LineaCarrito {
    pub id: String,              // código del producto (la página lo cruza con la vitrina para dibujar su foto)
    pub nombre: String,
    pub detalle: String,         // su descripción corta
    pub categoria: String,
    pub cantidad: i32,
    pub maximo: i32,             // cuántas unidades se pueden llevar (por el stock y el tope por apartado)
    pub precio_unitario: String, // "S/ 39.00"
    pub subtotal: String,        // "S/ 78.00"
}

// Un producto de un pedido ya apartado.
#[derive(Serialize)]
pub struct LineaPedido {
    pub nombre: String,
    pub cantidad: i32,
    pub subtotal: String, // "S/ 78.00"
}

// Un producto tal como lo dibuja la vitrina del catálogo (precio ya listo para mostrar).
#[derive(Serialize)]
pub struct ProductoCatalogo {
    pub id: String, // código del producto: así Juno sabe cuál es sin depender del nombre
    pub nombre: String,
    pub descripcion: String,
    pub categoria: String,
    pub precio: String, // "S/ 45.50"
    pub unidades: i32,  // 0 = agotado
    pub chips: Vec<Chip>,           // las cajas destacadas del resumen de la ficha
    pub grupos: Vec<GrupoFicha>,    // la tabla completa de especificaciones
    pub vistas: Vec<Vista>,         // la galería (vacía si el producto no tiene imágenes)
}

// Elementos visuales especiales que el agente puede pedirle al navegador que dibuje,
// además de los mensajes de texto. El JSON lleva un campo "tipo" que dice cuál es:
//   {"tipo":"consentimiento"}  -> casilla de términos y condiciones
//   {"tipo":"identidad", ...}  -> credencial (carnet) con los datos encontrados
//   {"tipo":"servicio", ...}   -> tarjeta con el detalle de un servicio técnico
//   {"tipo":"orden", ...}      -> tarjeta de una cita/orden con su seguimiento
//   {"tipo":"producto", ...}   -> tarjeta con el detalle de un producto
//   {"tipo":"apartado", ...}   -> tarjeta de un apartado (o su confirmación)
//   {"tipo":"catalogo", ...}   -> la vitrina de productos del catálogo público (la pantalla completa)
//   {"tipo":"carrito", ...}    -> el carrito de la tienda (lo que Juno guardó) y en qué fase va
//   {"tipo":"pedido", ...}     -> el pedido ya apartado, con su código de retiro
//   {"tipo":"solicitud", ...}  -> «Tu solicitud» de /servicios: en qué paso va la persona y lo que Juno ya anotó
#[derive(Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Componente {
    Consentimiento,
    Identidad {
        variante: String, // "persona" o "empresa": cambia el icono y el color de la credencial
        nombre: String,   // lo que va en grande: "Juan Carlos Perez Gomez"
        subtitulo: String, // lo que va debajo del nombre: "Persona natural"
        campos: Vec<Campo>, // el resto de datos (documento, sexo, nacimiento, dirección)
        estado: String, // "Pendiente de confirmar" (tarjeta de confirmación) o "Activo" (cliente ya registrado)
        // Código de cliente: solo existe en la credencial final. Si falta, no se envía el campo en el JSON.
        #[serde(skip_serializing_if = "Option::is_none")]
        codigo: Option<String>,
    },
    Servicio {
        nombre: String,         // "Formateo e instalación de Windows"
        descripcion: String,    // qué incluye el servicio
        precio: String,         // ya listo para mostrar: "Desde S/ 60.00" o "A cotizar"
        nota: String, // aviso sobre el precio (el costo final se confirma al revisar el equipo)
        tiempo: Option<String>, // "1 día" (si el servicio lo tiene)
        equipo: Option<String>, // el tipo de equipo elegido, "Laptop" (si corresponde)
    },
    Orden {
        // Código de la orden: solo existe una vez guardada (en la tarjeta de confirmación no se envía).
        #[serde(skip_serializing_if = "Option::is_none")]
        codigo: Option<String>,
        estado: String,   // "Pendiente de confirmar", "Cita agendada", "En revisión"...
        servicio: String, // "Formateo e instalación de Windows"
        #[serde(skip_serializing_if = "Option::is_none")]
        equipo: Option<String>,
        cita: String,     // "lunes 9 de noviembre · 15:00"
        problema: String, // lo que contó la persona
        seguimiento: Vec<Hito>, // pasos hechos, actual y pendientes (vacío en la confirmación)
    },
    Producto {
        nombre: String,
        descripcion: String,
        categoria: String,
        precio: String,         // ya listo para mostrar: "S/ 45.50"
        disponibilidad: String, // "3 unidades disponibles" o "Agotado"
    },
    Apartado {
        // Código del apartado: solo existe una vez guardado (en la tarjeta de confirmación no se envía).
        #[serde(skip_serializing_if = "Option::is_none")]
        codigo: Option<String>,
        estado: String,    // "Pendiente de confirmar", "Apartado", "Entregado"...
        producto: String,  // "Mouse inalámbrico M170"
        campos: Vec<Campo>, // cantidad, total, fecha límite, forma de pago...
    },
    Catalogo {
        filtros: Vec<String>, // lo que Juno aplicó, en palabras: «Laptops», «Hasta S/ 500», «“mouse”»
        total: usize,         // cuántos productos tiene el catálogo en total (sin filtros)
        productos: Vec<ProductoCatalogo>, // los que cumplen los filtros
    },
    Carrito {
        fase: String, // "carrito" (se ve la lista) o "cierre" (la página abre «¿Cómo quieres completar tu pedido?»)
        productos: Vec<LineaCarrito>,
        unidades: i32, // cuántas unidades hay en total (el número del botón «Carrito»)
        total: String, // "S/ 257.00"
    },
    Pedido {
        codigo: String, // el código de retiro
        productos: Vec<LineaPedido>,
        total: String,  // "S/ 257.00"
        vence: String,  // hasta cuándo puede recogerlo, listo para mostrar
    },
    Solicitud {
        modo: String, // "servicio" o "diagnostico"
        fase: String, // "servicio", "equipo", "detalle", "dia", "hora", "confirmar", "registro", "listo" o "fin"
        paso: u8,     // en cuál de los 5 pasos va (1 a 5)
        servicio: Option<String>, // nombre del servicio (en el diagnóstico, «Diagnóstico de equipo»)
        servicio_linea: Option<String>, // «Desde S/ 150.00 · 1 a 2 días»
        equipo: Option<String>,
        detalle: Option<String>, // lo que la persona contó que le pasa al equipo
        dia: Option<String>,     // el día elegido, como botón («mar 10/11»)
        cita: Option<String>,    // «martes 10 de noviembre · 15:00» (cuando ya eligió día y hora)
        dias: Vec<DiaCita>,      // los días que se pueden elegir (solo en las fases «dia» y «hora»)
        horas: Vec<HoraCita>,    // las horas del día elegido, con su cupo (solo en la fase «hora»)
        horario: String,         // la nota del horario de atención
        orden: Option<OrdenSolicitud>, // la orden creada (solo en la fase «listo»)
    },
}

// Un día que la persona puede elegir en el selector de fecha.
#[derive(Serialize)]
pub struct DiaCita {
    pub texto: String,  // lo que se envía al elegirlo: «mar 10/11»
    pub semana: String, // «MAR»
    pub numero: u32,    // 10
    pub mes: String,    // «nov»
    pub elegido: bool,
}

// Una hora de un día, con cuánto cupo le queda.
#[derive(Serialize)]
pub struct HoraCita {
    pub texto: String, // lo que se envía al elegirla: «15:00»
    pub cupos: i64,    // 0 = lleno
}

// La orden que quedó creada, para la pantalla de confirmación.
#[derive(Serialize)]
pub struct OrdenSolicitud {
    pub codigo: String,
    pub estado: String,
    pub seguimiento: Vec<Hito>,
}
