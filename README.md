# XpressMS

Plataforma web de H&Devs (Hardware and Development, Trujillo) con un asistente conversacional que informa
sobre el servicio técnico, agenda citas, da seguimiento a las órdenes y permite apartar productos.
Nombre provisional del proyecto.

## Qué incluye
- **Lienzo de Juno** (`/`): la pantalla principal. Juno al centro con «Hola, soy Juno», una caja para escribir y cuatro píldoras de acción rápida; en cuanto la persona escribe, aparece la conversación. Arriba, el botón ☰ abre el menú lateral (acciones rápidas que le envían un mensaje a Juno) y, a la derecha, «Iniciar sesión» (visitante) o el avatar del cliente reconocido («Hola, Pedro»). «Iniciar sesión» abre la tarjeta de ingreso (`/ingreso`): el cliente se reconoce con su documento, se registra con Juno o solo mira, y el personal entra al portal de equipo. La interfaz solo muestra: todo lo que tiene consecuencias lo hace Juno en la conversación.
- **Asistente Juno** (`/chat`, la misma pantalla del lienzo): registro de clientes con consentimiento, servicios y precios,
  citas, seguimiento de órdenes y catálogo de productos con apartado (el pago es en tienda).
- **Catálogo público / tienda** (`/catalogo`, página propia y pública, como «Mi perfil y datos»; se abre con «Ver catálogo de productos» del menú y no toca el chat principal): la cuadrícula de productos con precios en S/. La página no tiene buscador ni filtros: Juno vive allí como una burbuja flotante abajo a la derecha y es quien filtra («solo laptops», «menos de S/ 500», «mouse») y aparta; la página solo muestra lo que ella decide. La ventana de Juno es una conversación aparte (`xpressms_token_tienda`), sin saludo guardado. La lógica vive en `src/modulos/agente/flujo_catalogo.rs`; la página, en `templates/catalogo.html`, `static/js/tienda.js`, `static/js/tienda/juno.js` y `static/css/tienda.css`; los productos salen de `/publico/catalogo`.
- **Mi perfil y datos** (`/perfil`, solo clientes reconocidos): el pase del cliente (formato de DNI, con reverso y código de barras, portada y foto a elección en el propio dispositivo, «Mostrar en tienda» y «Descargar»), sus datos personales y de contacto, sus direcciones de entrega y recojo (hasta 5) y las personas autorizadas a recoger (hasta 3; autorizar exige un código SMS al celular del cliente). La página solo muestra: los botones «… con Juno» llevan la petición al lienzo y Juno la ejecuta.
- **Panel interno** (`/ingresar` y `/panel`) con tres roles:
  - Administrador: servicios y equipos, personal y usuarios, órdenes, productos y apartados.
  - Técnico: solo las órdenes que tiene asignadas.
  - Vendedor: productos, categorías y apartados.

## Stack
Rust (Axum, SQLx, Tokio) · HTML en el servidor y JavaScript sin frameworks · PostgreSQL en Neon ·
despliegue en Render con Docker.

## Estructura
- `src/modulos/acceso`: clientes, personal, roles, claves y sesiones.
- `src/modulos/servicio_tecnico`: servicios, equipos, citas y órdenes.
- `src/modulos/catalogo`: categorías, productos y apartados.
- `src/modulos/agente`: el asistente (cada flujo en su archivo: registro, servicios, citas, productos, perfil).
- `src/modulos/perfil`: «Mi perfil y datos»: los datos de la página (`/perfil/datos`) y las consultas de direcciones y personas autorizadas.
- `src/modulos/panel`: las páginas del panel interno.
- `src/modulos/publico`: los datos públicos del catálogo (`/publico/inicio`); el lienzo ya no los usa, quedan para una futura página de catálogo.
- `src/modulos/comun`: códigos, fechas, formatos y validaciones compartidas.
- `base_de_datos/`: scripts SQL numerados; se ejecutan en orden en Neon (`01` al `10`).
  `herramientas/` trae scripts de apoyo (datos de ejemplo y limpieza de pruebas).
- `templates/`: páginas HTML (`parciales/` trae el encabezado y el menú que comparten el lienzo y el perfil). `static/`: estilos, scripts y textos legales.

## Cómo correrlo
1. Copia `.env.example` como `.env` y completa `DATABASE_URL`, `TEXTBEE_API_KEY` y `FACTILIZA_TOKEN` (nunca subas el `.env`). Para programar sin enviar SMS reales, agrega `SMS_MODO=simulado` (el código sale en la terminal) y `CONSULTA_MODO=simulado` (datos de DNI/RUC inventados).
2. Ejecuta en Neon los scripts de `base_de_datos/` en orden, del `01` al `10`.
3. Crea el primer administrador: `cargo run -- crear-admin`.
4. `cargo run` y abre http://localhost:3000. El portal de equipo está en http://localhost:3000/ingresar (y en el lado «Portal de equipo» de la ventana «Iniciar sesión»).

Pruebas automáticas: `cargo test`.

## Despliegue
Render construye la imagen con el `Dockerfile` en cada push a `main`. La variable `DATABASE_URL`
y las claves de Textbee (`TEXTBEE_API_KEY`, `TEXTBEE_DEVICE_ID`) y de Factiliza (`FACTILIZA_TOKEN`) se configuran en el panel de Render.
