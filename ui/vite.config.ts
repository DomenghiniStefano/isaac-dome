/// <reference types="vitest/config" />
import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  server: { port: 1420, strictPort: true },
  test: {
    include: ['src/**/*.test.ts'],
    // Vitest doesn't load CSS by default and hands a `?raw` import an empty string.
    // `cn()` reads its token names from the theme files, and base.test.ts reads base.css,
    // so those load as source. No `$` anchor: the module id ends in `?raw`.
    css: { include: [/src\/assets\/theme\/.+\.css/, /src\/assets\/base\.css/] },
  },
})
