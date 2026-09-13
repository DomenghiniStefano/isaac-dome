import { isTauri } from '@tauri-apps/api/core'
import { emit, emitTo, listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  WebviewWindow,
  getAllWebviewWindows,
} from '@tauri-apps/api/webviewWindow'
import type { Point } from '@/lib/drag/dragList'
import { fakeWindows } from './fakeWindows'
import { WindowEventName } from './messages'
import type { WindowMessage } from './messages'
import { windowBackground } from './windowBackground'

// The first window's label, fixed by `tauri.conf.json`. Every other window is born here.
export const MainLabel = 'main'

// A window as the hit test needs it: desktop physical pixels, the units `cursorPosition()`
// answers in.
export interface WindowBox {
  label: string
  left: number
  top: number
  width: number
  height: number
  scaleFactor: number
}

export interface WindowPort {
  label: () => string
  isMain: () => boolean
  list: () => Promise<WindowBox[]>
  create: (label: string, at: Point, size: Point) => Promise<void>
  send: (label: string, message: WindowMessage) => Promise<void>
  broadcast: (message: WindowMessage) => Promise<void>
  listen: (handler: (message: WindowMessage) => void) => Promise<() => void>
  focus: (label: string) => Promise<void>
  closeSelf: () => Promise<void>
  self: () => Promise<WindowBox>
}

// A label nothing else holds: two windows are created in the same millisecond only if the user
// has two hands. Base 36 keeps it short enough to read in a log.
export const newWindowLabel = (): string => `win-${Date.now().toString(36)}`

interface Measurable {
  label: string
  outerPosition: () => Promise<{ x: number; y: number }>
  outerSize: () => Promise<{ width: number; height: number }>
  scaleFactor: () => Promise<number>
}

const boxOf = async (w: Measurable): Promise<WindowBox> => {
  const [p, s, f] = await Promise.all([
    w.outerPosition(),
    w.outerSize(),
    w.scaleFactor(),
  ])
  return {
    label: w.label,
    left: p.x,
    top: p.y,
    width: s.width,
    height: s.height,
    scaleFactor: f,
  }
}

const tauriPort: WindowPort = {
  label: () => getCurrentWindow().label,
  isMain: () => getCurrentWindow().label === MainLabel,
  list: async () => {
    const all = await getAllWebviewWindows()
    const boxes = await Promise.all(
      all.map(async (w) => {
        try {
          if (!(await w.isVisible())) return null
          if (await w.isMinimized()) return null
          return await boxOf(w)
        } catch {
          // **A window that is not there is not an error.** `getAllWebviewWindows` keeps
          // listing a window for a while after it closes, and every call on it then answers
          // `window not found` — which, thrown from inside a drag, killed the gesture and put
          // the tab back. Measured on the machine, 2026-09-13: a stale window is one fewer
          // target, nothing more.
          return null
        }
      }),
    )
    return boxes.filter((box) => box !== null)
  },
  create: async (label, at, size) => {
    // The app's own page, with no state in its URL: what the window holds arrives through the
    // handshake. Its own title bar, like the first window's, and the app's colour under the
    // webview so it never opens on a white frame.
    const w = new WebviewWindow(label, {
      url: 'index.html',
      x: at.x,
      y: at.y,
      width: size.x,
      height: size.y,
      decorations: false,
      backgroundColor: windowBackground(),
    })
    await new Promise<void>((resolve, reject) => {
      void w.once('tauri://created', () => resolve())
      void w.once('tauri://error', (e) => reject(new Error(String(e.payload))))
    })
  },
  send: async (label, message) => {
    // **The target is named by kind, not by label alone.** A bare string means "whatever
    // carries this label", and a `WebviewWindow` carries it three times over — as a window, as
    // a webview, and as the pair — so the same message arrived twice and a docked tab was
    // inserted twice. The other messages hid it: `Ready` is answered once because the debt is
    // consumed, `Seed` because a seeded window is no longer pending. `Docked` had nothing to
    // make it idempotent, and it was the one the owner saw duplicate.
    await emitTo({ kind: 'WebviewWindow', label }, WindowEventName, message)
  },
  broadcast: async (message) => {
    await emit(WindowEventName, message)
  },
  // **A window listens for what was sent to it.** `listen` defaults to `{ kind: 'Any' }` — it
  // hears every message on the channel, whoever it was addressed to — so a tab docked into one
  // window was docked into every window at once. Naming the target on both sides is what makes
  // `send` mean send, and it is the whole of the duplication the owner kept seeing.
  listen: (handler) =>
    listen<WindowMessage>(WindowEventName, (e) => handler(e.payload), {
      target: { kind: 'WebviewWindow', label: getCurrentWindow().label },
    }),
  focus: async (label) => {
    const all = await getAllWebviewWindows()
    await all.find((w) => w.label === label)?.setFocus()
  },
  closeSelf: async () => {
    await getCurrentWindow().close()
  },
  self: () => boxOf(getCurrentWindow()),
}

// Outside Tauri — `pnpm ui:dev` in a browser tab — windows do not exist. The fake invents one
// so the gesture can be exercised and watched; it is not the verification, which needs a real
// window, a mouse and two monitors.
export const windowPort: WindowPort = isTauri() ? tauriPort : fakeWindows()
