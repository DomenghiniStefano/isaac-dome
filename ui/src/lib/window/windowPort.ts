import { isTauri } from '@tauri-apps/api/core'
import { PhysicalPosition } from '@tauri-apps/api/dpi'
import { emit, emitTo, listen } from '@tauri-apps/api/event'
import {
  availableMonitors,
  getCurrentWindow,
  primaryMonitor,
} from '@tauri-apps/api/window'
import type { Monitor } from '@tauri-apps/api/window'
import {
  WebviewWindow,
  getAllWebviewWindows,
} from '@tauri-apps/api/webviewWindow'
import type { Point } from '@/lib/drag/dragList'
import { fakeWindows } from './fakeWindows'
import { soleWindowPort } from './soleWindow'
import { WindowEventName } from './messages'
import type { WindowMessage } from './messages'
import { windowBackground } from './windowBackground'
import { WindowFloor } from './windowFloor'
import { windowCreated } from './windowCreated'

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

// A monitor as the clamp needs it: its **work area** — the screen minus the taskbar and the
// docks — in desktop physical pixels, and the factor that turns a physical size into the logical
// one `new WebviewWindow` takes. The monitor's full `size` is deliberately not what travels: a
// window restored under the taskbar is a window whose title bar cannot be grabbed.
export interface MonitorArea {
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
  labels: () => Promise<string[]>
  monitors: () => Promise<MonitorArea[]>
  create: (label: string, at: Point, size: Point) => Promise<void>
  send: (label: string, message: WindowMessage) => Promise<void>
  broadcast: (message: WindowMessage) => Promise<void>
  listen: (handler: (message: WindowMessage) => void) => Promise<() => void>
  focus: (label: string) => Promise<void>
  closeSelf: () => Promise<void>
  self: () => Promise<WindowBox>
}

// The time a label is minted at: the clock, or one past the last label when the clock has not
// moved (card #80, R8). Two in the same millisecond are not a user with two hands — `reopen`
// mints one per restored window in a loop — and the same label twice is a window Tauri refuses.
// A counter in the label would do the same and break its shape, which the session's order and
// the tray (`crates/ipc/src/tray.rs`) both read.
export const nextMint = (now: number, last: number): number =>
  Math.max(now, last + 1)

const minted = { last: 0 }

// A label nothing else in this window has minted. Base 36 keeps it short enough to read in a log.
export const newWindowLabel = (): string => {
  minted.last = nextMint(Date.now(), minted.last)
  return `win-${minted.last.toString(36)}`
}

interface Measurable {
  label: string
  outerPosition: () => Promise<{ x: number; y: number }>
  outerSize: () => Promise<{ width: number; height: number }>
  scaleFactor: () => Promise<number>
}

const measureWindow = async (w: Measurable): Promise<WindowBox> => {
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

const areaOf = (m: Monitor): MonitorArea => ({
  left: m.workArea.position.x,
  top: m.workArea.position.y,
  width: m.workArea.size.width,
  height: m.workArea.size.height,
  scaleFactor: m.scaleFactor,
})

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
          return await measureWindow(w)
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
  // **Every window there is, including the ones `list()` hides.** `list()` answers the hit test,
  // so it drops what is minimized or invisible — right for "where can a tab land", wrong for
  // "which windows are open". A minimized window is still part of the session, and a roster that
  // forgot it would write a document that loses it on the next start.
  labels: async () => (await getAllWebviewWindows()).map((w) => w.label),
  // Primary first: it is where a box with nowhere left to go lands.
  monitors: async () => {
    const [all, primary] = await Promise.all([
      availableMonitors(),
      primaryMonitor(),
    ])
    const at = primary
      ? all.findIndex(
          (m) =>
            m.position.x === primary.position.x &&
            m.position.y === primary.position.y,
        )
      : -1
    const ordered =
      at <= 0 ? all : [all[at]!, ...all.filter((_, n) => n !== at)]
    return ordered.map(areaOf)
  },
  create: async (label, at, size) => {
    // The app's own page, with no state in its URL: what the window holds arrives through the
    // handshake. Its own title bar, like the first window's, and the app's colour under the
    // webview so it never opens on a white frame.
    //
    // It is born **hidden and placed afterwards**, for two reasons: `x`/`y` at creation are
    // logical pixels while the point we hold is the desktop's physical ones — the same number
    // until a screen is scaled, and then not — and a window that appears before it is placed
    // jumps across the desktop in front of the user.
    const w = new WebviewWindow(label, {
      url: 'index.html',
      width: size.x,
      height: size.y,
      // The same floor as the window it was torn out of: a window born here never goes through
      // `tauri.conf.json`, so a minimum written only there would hold for every window except
      // the ones the user makes by hand (spec 3.13a §8).
      minWidth: WindowFloor.Width,
      minHeight: WindowFloor.Height,
      decorations: false,
      backgroundColor: windowBackground(),
      visible: false,
    })
    await windowCreated(w)
    await w.setPosition(
      new PhysicalPosition(Math.round(at.x), Math.round(at.y)),
    )
    await w.show()
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
  self: () => measureWindow(getCurrentWindow()),
}

// Outside Tauri — `pnpm ui:dev` in a browser tab — windows do not exist. In development the fake
// invents one so the gesture can be exercised and watched; it is not the verification, which
// needs a real window, a mouse and two monitors. A production build outside Tauri gets the sole
// window instead: `import.meta.env.DEV` is a build-time constant, so the fake's branch and its
// import are dropped from that bundle (card #81, V10).
const outsideTauri = (): WindowPort =>
  import.meta.env.DEV ? fakeWindows() : soleWindowPort
export const windowPort: WindowPort = isTauri() ? tauriPort : outsideTauri()
