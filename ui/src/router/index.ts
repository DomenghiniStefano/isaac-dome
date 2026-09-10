import { createMemoryHistory, createRouter } from 'vue-router'
import { routes } from './routes'

// Memory history: a tab is a position and no one reads a URL in a Tauri window; the hash
// belongs to the development pages. The router only ever shows the active tab's location.
export const router = createRouter({ history: createMemoryHistory(), routes })
