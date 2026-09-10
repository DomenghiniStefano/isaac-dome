import { createApp } from 'vue'
import './assets/main.css'
import App from './App.vue'
import { i18n } from './i18n'
import { DevRoute } from './lib/constants/devRoutes'

// The Kit page shows every primitive in every state, to compare with the design export.
// Behind import.meta.env.DEV the production build drops the import entirely.
const mountKit = async () => {
  const { default: KitPage } = await import('./kit/KitPage.vue')
  createApp(KitPage).use(i18n).mount('#app')
}

if (import.meta.env.DEV && window.location.hash === DevRoute.Kit) {
  void mountKit()
} else {
  createApp(App).use(i18n).mount('#app')
}
