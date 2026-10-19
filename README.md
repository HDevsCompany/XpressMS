# XpressMS

Plataforma web de H&Devs (Hardware and Development) con un agente conversacional
para informar sobre servicio técnico y agendar citas.

## Stack
Rust (Axum, SQLx, Askama, HTMX) · PostgreSQL en Neon · despliegue en Render.

## Estructura
- `src/modulos/acceso`: registro, inicio de sesión y roles.
- `src/modulos/servicio_tecnico`: servicios, citas y órdenes.
- `src/modulos/agente`: el asistente conversacional.
- `migrations/`: SQL de las tablas, un archivo por cambio.
- `templates/`: HTML de Askama, una carpeta por módulo.
- `static/`: css, js e imágenes.

## Cómo correrlo
1. Copia `.env.example` como `.env` y completa `DATABASE_URL`.
2. `cargo run`
