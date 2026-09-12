import { isTauri } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'

// The app's version, read once from the bundle's own metadata (`crates/app/tauri.conf.json`)
// — never typed in the frontend, where it would drift from the thing that ships. Outside
// Tauri, on the development server, there is no bundle: `null`, and About says so.
export const appVersion = async (): Promise<string | null> =>
  isTauri() ? getVersion() : null
