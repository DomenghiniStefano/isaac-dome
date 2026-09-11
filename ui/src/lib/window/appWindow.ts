import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

// The only module that talks to the window, as lib/ipc is for commands. Outside Tauri (the
// development server in a browser) the controls do nothing and the window counts as focused.
export const minimizeWindow = async (): Promise<void> => {
  if (isTauri()) await getCurrentWindow().minimize()
}

export const toggleMaximizeWindow = async (): Promise<void> => {
  if (isTauri()) await getCurrentWindow().toggleMaximize()
}

export const closeWindow = async (): Promise<void> => {
  if (isTauri()) await getCurrentWindow().close()
}

// Calls `onChange` whenever the window gains or loses focus; resolves to the unsubscribe.
export const watchWindowFocus = async (
  onChange: (focused: boolean) => void,
): Promise<() => void> => {
  if (!isTauri()) return () => undefined
  return getCurrentWindow().onFocusChanged(({ payload }) => onChange(payload))
}
