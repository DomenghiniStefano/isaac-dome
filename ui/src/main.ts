import { createPinia } from 'pinia'
import { createApp } from 'vue'
import './assets/main.css'
import App from './App.vue'
import { i18n } from './i18n'
import { DevRoute } from './lib/constants/devRoutes'
import { router } from './router'

// The splash `index.html` painted before the bundle existed: it goes when something is on
// screen to replace it, never before (`docs/BACKLOG.md` B18).
const mounted = () => document.getElementById('splash')?.remove()

// The development pages (the Kit, the verification page) sit behind import.meta.env.DEV:
// the production build drops their imports entirely.
const mountKit = async () => {
  const { default: KitPage } = await import('./kit/KitPage.vue')
  createApp(KitPage).use(i18n).mount('#app')
  mounted()
}

const mountVerify = async () => {
  const { default: VerifyPage } = await import('./verify/VerifyPage.vue')
  createApp(VerifyPage).use(i18n).mount('#app')
  mounted()
}

// The interface's size is applied **before the first paint**: the window would otherwise
// draw the shell at 100 and jump to the user's size a moment later. What the user sees
// meanwhile is the splash `index.html` painted. A command that fails is not fatal — the app
// mounts at 100, and the Appearance screen says what happened.
const mountApp = async () => {
  try {
    const { settings } = await import('./lib/ipc/settings')
    const { applyScale } = await import('./lib/scale/apply')
    applyScale((await settings()).scale)
  } catch {
    // No backend, or a settings file that won't read: `--app-scale` keeps its CSS fallback.
  }
  createApp(App).use(createPinia()).use(router).use(i18n).mount('#app')
  mounted()
}

const hash = window.location.hash
if (import.meta.env.DEV && hash === DevRoute.Kit) {
  void mountKit()
} else if (import.meta.env.DEV && hash === DevRoute.Verify) {
  void mountVerify()
} else {
  void mountApp()
}
