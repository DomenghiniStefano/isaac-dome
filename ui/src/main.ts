import { createPinia } from 'pinia'
import { createApp } from 'vue'
import './assets/main.css'
import App from './App.vue'
import { i18n } from './i18n'
import { DevRoute } from './lib/constants/devRoutes'
import { router } from './router'

// The development pages (the Kit, the verification page) sit behind import.meta.env.DEV:
// the production build drops their imports entirely.
const mountKit = async () => {
  const { default: KitPage } = await import('./kit/KitPage.vue')
  createApp(KitPage).use(i18n).mount('#app')
}

const mountVerify = async () => {
  const { default: VerifyPage } = await import('./verify/VerifyPage.vue')
  createApp(VerifyPage).use(i18n).mount('#app')
}

const hash = window.location.hash
if (import.meta.env.DEV && hash === DevRoute.Kit) {
  void mountKit()
} else if (import.meta.env.DEV && hash === DevRoute.Verify) {
  void mountVerify()
} else {
  createApp(App).use(createPinia()).use(router).use(i18n).mount('#app')
}
