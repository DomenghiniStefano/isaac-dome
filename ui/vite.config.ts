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
  build: {
    rollupOptions: {
      input: {
        index: fileURLToPath(new URL('./index.html', import.meta.url)),
        // The tab that follows the cursor during a tear-off: a second page, not a route, so
        // it does not carry the app's bundle into a window that shows one word.
        preview: fileURLToPath(new URL('./preview.html', import.meta.url)),
      },
    },
  },
  test: {
    include: ['src/**/*.test.ts'],
    // Vitest doesn't load CSS by default and hands a `?raw` import an empty string.
    // `cn()` reads its token names from the theme files, and base.test.ts reads base.css,
    // so those load as source. No `$` anchor: the module id ends in `?raw`.
    css: { include: [/src\/assets\/theme\/.+\.css/, /src\/assets\/base\.css/] },
  },
})
