/// <reference types="vitest/config" />
import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  // vue-i18n's feature flags (Optimization guide). Undeclared they default to `true`, so
  // both halves the app never touches were shipped. The value has to end up a boolean
  // literal in the emitted code — a string would leave the minifier a condition it cannot
  // evaluate — which is what `define` writes when given `'false'`.
  define: {
    // No `<i18n-t>`, `<i18n-n>`, `<i18n-d>` and no `v-t` anywhere in `src/`: every string
    // goes through `useMessages()`. The components and the directive leave the bundle.
    __VUE_I18N_FULL_INSTALL__: 'false',
    // `createI18n({ legacy: false })`: the Options-API path has no caller here. The flag
    // itself is expected to go in v12, with the API it guards.
    __VUE_I18N_LEGACY_API__: 'false',
    // `__INTLIFY_DROP_MESSAGE_COMPILER__` stays at its default, deliberately: dropping the
    // compiler requires the messages to be pre-compiled to AST by
    // `@intlify/unplugin-vue-i18n`, and ours are plain TS objects whose `{named}` and
    // `a | b` are parsed at runtime. Setting it would leave nothing able to read them.
  },
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
