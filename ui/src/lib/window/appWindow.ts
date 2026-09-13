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

// This window's size in logical pixels — what `new WebviewWindow` takes. `outerSize` answers in
// physical ones, and on a scaled screen handing those over would open a window twice as large as
// the one it was born from. Outside Tauri, the size the config gives the first window.
export const windowSize = async (): Promise<{ x: number; y: number }> => {
  if (!isTauri()) return { x: 1280, y: 800 }
  const w = getCurrentWindow()
  const [size, factor] = await Promise.all([w.outerSize(), w.scaleFactor()])
  return { x: size.width / factor, y: size.height / factor }
}

// Calls `onChange` whenever the window gains or loses focus; resolves to the unsubscribe.
export const watchWindowFocus = async (
  onChange: (focused: boolean) => void,
): Promise<() => void> => {
  if (!isTauri()) return () => undefined
  return getCurrentWindow().onFocusChanged(({ payload }) => onChange(payload))
}
