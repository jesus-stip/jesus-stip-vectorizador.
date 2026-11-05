import { defineConfig } from 'vite';

// base './' so the build works from any folder or subdomain on the cPanel host
export default defineConfig({
  base: './',
  worker: { format: 'es' },
  build: { target: 'es2022', assetsInlineLimit: 0 },
});
