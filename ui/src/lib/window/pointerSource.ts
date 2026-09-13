import { isTauri } from '@tauri-apps/api/core'
import { cursorPosition } from '@tauri-apps/api/window'
import type { Point } from '@/lib/drag/dragList'

// Where the pointer is once the tab has left the window, and when the button comes up.
//
// **The one module the spike decides** (`docs/superpowers/specs/2026-09-13-drag-and-windows-design.md`
// §2). WebView2's pointer capture is unreliable once the cursor leaves the control
// (microsoft-ui-xaml #8677), and whether the events still arrive can only be measured on a real
// window with a real mouse — Task 1 of the plan, not run yet. This is the implementation for
// the answer "they arrive": the release comes from the DOM. If the measurement says otherwise,
// only this file changes — a thread in `crates/app` watching the mouse button replaces the
// listeners below, and everything downstream stays as it is.
//
// The **position** does not come from the events either way: `cursorPosition()` answers in the
// desktop's physical pixels, which is what the hit test compares against, and it keeps
// answering with the cursor anywhere on the desktop. Reading it once per animation frame is
// cheaper than converting an event's client coordinates through a window that may not be the
// one under the cursor.

export interface PointerWatch {
  stop: () => void
}

// How long the gesture waits for a pointer event before it gives up. If the webview goes
// silent — the very thing the spike is about — the drag is **cancelled**, not dropped: the tab
// goes back where it was. Landing a tab somewhere the user never released it would be worse
// than doing nothing.
const SilenceTimeout = 10000

export const watchPointer = (handlers: {
  onMove: (p: Point) => void
  onRelease: (p: Point) => void
  onLost: () => void
}): PointerWatch => {
  let last: Point = { x: 0, y: 0 }
  let frame = 0
  let silence = 0
  let stopped = false

  const stop = () => {
    if (stopped) return
    stopped = true
    cancelAnimationFrame(frame)
    window.clearTimeout(silence)
    window.removeEventListener('pointerup', onUp)
    window.removeEventListener('pointercancel', onUp)
    window.removeEventListener('pointermove', heard)
  }

  const waitAgain = () => {
    window.clearTimeout(silence)
    silence = window.setTimeout(() => {
      stop()
      handlers.onLost()
    }, SilenceTimeout)
  }

  function heard() {
    waitAgain()
  }

  function onUp() {
    stop()
    handlers.onRelease(last)
  }

  const tick = () => {
    if (stopped) return
    void cursorPosition()
      .then((p) => {
        if (stopped) return
        last = { x: p.x, y: p.y }
        handlers.onMove(last)
        frame = requestAnimationFrame(tick)
      })
      .catch(() => {
        // The cursor cannot be read at all: better to put the tab back than to follow a
        // position we are inventing.
        stop()
        handlers.onLost()
      })
  }

  if (!isTauri()) {
    // On the development server there is no desktop cursor to follow. The gesture still ends
    // on the release, at the last place the DOM reported.
    const onDomMove = (e: PointerEvent) => {
      last = { x: e.screenX, y: e.screenY }
      handlers.onMove(last)
      waitAgain()
    }
    window.addEventListener('pointermove', onDomMove)
    window.addEventListener('pointerup', onUp)
    window.addEventListener('pointercancel', onUp)
    waitAgain()
    return {
      stop: () => {
        window.removeEventListener('pointermove', onDomMove)
        stop()
      },
    }
  }

  window.addEventListener('pointerup', onUp)
  window.addEventListener('pointercancel', onUp)
  window.addEventListener('pointermove', heard)
  waitAgain()
  frame = requestAnimationFrame(tick)
  return { stop }
}
