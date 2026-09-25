import { isTauri } from '@tauri-apps/api/core'
import { LogicalPosition } from '@tauri-apps/api/dpi'
import {
  WebviewWindow,
  getAllWebviewWindows,
} from '@tauri-apps/api/webviewWindow'
import type { Point } from '@/lib/drag/dragList'
import { windowCreated } from './windowCreated'

// The tab that follows the cursor once it has left the strip. It is created on the first
// tear-off of the session and afterwards only hidden and shown again: creating a window costs
// a webview, and the second drag must not pay for it.
export const PreviewLabel = 'tab-preview'

const Size = { width: 200, height: 34 }
// Where the cursor holds it: near the top-left, the way it held the tab.
const Grab = { x: 24, y: 16 }
// Where it waits while hidden. A window created at the cursor and only then hidden can flash
// there for a frame; one created far off-screen cannot.
const Offscreen = { x: -4000, y: -4000 }

let preview: WebviewWindow | null = null
let shownLabel: string | null = null

const found = async (): Promise<WebviewWindow | null> => {
  // Same reason as `appEvents`: outside Tauri these APIs do not answer "no", they throw from
  // inside whatever hook called them.
  if (!isTauri()) return null
  if (preview) return preview
  const all = await getAllWebviewWindows()
  preview = all.find((w) => w.label === PreviewLabel) ?? null
  return preview
}

const at = (p: Point): LogicalPosition =>
  new LogicalPosition(p.x - Grab.x, p.y - Grab.y)

// Creates the card **hidden**, for the label about to be dragged. Called when a drag starts in
// the strip, not when it leaves it: a window costs a webview and the first one costs the most,
// and paying for it at the tear-off leaves the cursor with nothing to carry for as long as it
// takes — which is exactly the moment the eye is watching. Idempotent.
export const warmPreview = async (label: string): Promise<void> => {
  if (!isTauri()) return
  const existing = await found()
  if (existing && shownLabel === label) return
  // The label is baked into the page at creation, so a different tab means a different card.
  // Recreating the window is what keeps `@tauri-apps/api` out of `src/preview/`: one window per
  // distinct tab dragged, and it is created before anyone is waiting for it.
  if (existing) {
    await existing.close()
    preview = null
  }
  // `focus: false` is the whole gesture's life: a window that takes the focus takes the
  // origin's pointer capture with it, and the drag dies in the act.
  const created = new WebviewWindow(PreviewLabel, {
    url: `preview.html?label=${encodeURIComponent(label)}`,
    width: Size.width,
    height: Size.height,
    x: Offscreen.x,
    y: Offscreen.y,
    decorations: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    shadow: false,
    resizable: false,
    focus: false,
    visible: false,
  })
  preview = created
  shownLabel = label
  // A card Tauri would not make is forgotten, so the next drag tries again instead of
  // showing a window that is not there.
  await windowCreated(created).catch((e: unknown) => {
    preview = null
    shownLabel = null
    throw e
  })
}

export const showPreview = async (label: string, p: Point): Promise<void> => {
  if (!isTauri()) return
  await warmPreview(label)
  const w = await found()
  if (!w) return
  // Placed before it is shown: a card that appears at the far corner and then jumps to the
  // cursor is worse than one that appears late.
  await w.setPosition(at(p))
  await w.show()
}

export const movePreview = async (p: Point): Promise<void> => {
  const w = await found()
  await w?.setPosition(at(p))
}

export const hidePreview = async (): Promise<void> => {
  const w = await found()
  await w?.hide()
}
