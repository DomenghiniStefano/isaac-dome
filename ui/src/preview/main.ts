import { createApp } from 'vue'
import '../assets/main.css'
import PreviewPage from './PreviewPage.vue'

// The preview is one card with a name on it: no router, no store, no i18n, no IPC. Everything
// it does not load is time it does not spend, and it has to appear inside a drag.
createApp(PreviewPage).mount('#app')
