# Dockerfile: la "receta" que usa Render para construir y ejecutar XpressMS.
# Tiene dos etapas: primero se COMPILA (imagen grande, con Rust) y luego se copia solo el programa
# resultante a una imagen pequeña. Así el servidor final es liviano.

# ---------- Etapa 1: compilar ----------
# "rust:1-bookworm" trae Rust y las herramientas de compilación.
FROM rust:1-bookworm AS compilar

# Carpeta de trabajo dentro del contenedor.
WORKDIR /app

# Truco para acelerar: copiamos SOLO la lista de dependencias y compilamos con un main vacío.
# Así Docker guarda en caché las dependencias y no las recompila cada vez que cambias tu código.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Ahora sí copiamos tu código y las plantillas (inicio.html se incrusta al compilar con include_str!).
COPY src ./src
COPY templates ./templates

# "touch" marca main.rs como modificado para que cargo recompile tu código y no reuse el main vacío.
RUN touch src/main.rs && cargo build --release

# ---------- Etapa 2: ejecutar ----------
# Imagen pequeña de Debian, sin Rust ni herramientas de compilación.
FROM debian:bookworm-slim

# ca-certificates permite la conexión segura (TLS) con Neon.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Carpeta de trabajo del programa en ejecución. La carpeta static/ se busca relativa a ella.
WORKDIR /app

# Copiamos solo el programa ya compilado desde la etapa 1 (se llama "xpressms", como el paquete).
COPY --from=compilar /app/target/release/xpressms /usr/local/bin/xpressms

# Archivos que el navegador descarga tal cual: estilos (css), scripts (js) y textos legales.
COPY static ./static

# Comando que se ejecuta al encender el contenedor.
# Render le da el puerto en la variable PORT y config.rs ya la lee.
CMD ["xpressms"]
