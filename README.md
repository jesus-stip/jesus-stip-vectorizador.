# Vectorizador

Herramienta interna para convertir imágenes (PNG, JPG, BMP, GIF) en vectores SVG.
Todo ocurre en el navegador: las imágenes no se suben a ningún servidor.

- `web/`: la app (Vite + TypeScript, sin framework). Su build es un sitio estático.
- `engine/`: el motor en Rust (preprocesado + [VTracer](https://github.com/visioncortex/vtracer), MIT o Apache-2.0),
  compilado a WebAssembly. El `.wasm` ya compilado está en `web/src/engine/vectorizer.wasm`.
- `replica/`: planeación (mapa, arquitectura, prueba de motores, registro de construcción).
- `spike/`: scripts de la prueba de motores.

## Subirlo a cPanel (subdominio)

1. En cPanel: **Dominios → Crear un dominio nuevo**, por ejemplo `vector.tudominio.com`. Anota su carpeta raíz.
2. **SSL/TLS Status**: ejecuta AutoSSL para el subdominio (HTTPS).
3. **Administrador de archivos**: entra en la carpeta del subdominio, sube `vectorizador-dist.zip` y usa **Extraer**.
   Activa "Mostrar archivos ocultos" y verifica que esté `.htaccess`.
4. **Privacidad de directorios**: protege esa carpeta con usuario y contraseña.
5. Abre `https://vector.tudominio.com`, sube una imagen y descarga el SVG.

## Desarrollo

```bash
cd web && npm install && npm run dev
```

Compilar el motor (requiere Rust con el target `wasm32-unknown-unknown`):

```bash
cd web && npm run build:wasm
```

Build para producción (genera `web/dist/`):

```bash
cd web && npm run build
```

Prueba de punta a punta (con `npm run preview` corriendo):

```bash
cd web && node e2e/slice.mjs http://localhost:4173/ ruta/a/imagen.png ../replica/clone-screens
```
