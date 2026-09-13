import { isTauri } from '@tauri-apps/api/core'
import { LogicalPosition } from '@tauri-apps/api/dpi'
import {
  WebviewWindow,
  getAllWebviewWindows,
} from '@tauri-apps/api/webviewWindow'
import type { Point } from '@/lib/drag/dragList'

// The tab that follows the cursor once it has left the strip. It is created on the first
// tear-off of the session and afterwards only hidden and shown again: creating a window costs
// a webview, and the second drag must not pay for it.
export const PreviewLabel = 'tab-preview'

const Size = { width: 200, height: 34 }
// Where the cursor holds it: near the top-left, the way it held the tab.
const Grab = { x: 24, y: 16 }

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

export const showPreview = async (label: string, p: Point): Promise<void> => {
  if (!isTauri()) return
  const existing = await found()
  // The label is baked into the page at creation. A different tab means a different card, and
  // recreating the window is what keeps `@tauri-apps/api` out of `src/preview/` — one window
  // per distinct tab dragged, which is one per drag at worst.
  if (existing && shownLabel !== label) {
    await existing.close()
    preview = null
  } else if (existing) {
    await existing.setPosition(at(p))
    await existing.show()
    return
  }
  // `focus: false` is the whole gesture's life: a window that takes the focus takes the
  // origin's pointer capture with it, and the drag dies in the act.
  const created = new WebviewWindow(PreviewLabel, {
    url: `preview.html?label=${encodeURIComponent(label)}`,
    width: Size.width,
    height: Size.height,
    x: p.x - Grab.x,
    y: p.y - Grab.y,
    decorations: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    shadow: false,
    resizable: false,
    focus: false,
  })
  preview = created
  shownLabel = label
  await new Promise<void>((resolve) => {
    void created.once('tauri://created', () => resolve())
  })
}

export const movePreview = async (p: Point): Promise<void> => {
  const w = await found()
  await w?.setPosition(at(p))
}

export const hidePreview = async (): Promise<void> => {
  const w = await found()
  await w?.hide()
}
