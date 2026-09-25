import { WindowMessageKind } from './messages'
import { currentLayout, sameLayout, setLayout } from './layout'
import type { Layout } from './layout'
import { windowPort } from './windowPort'

// One sidebar layout for the app, so a window that changes it tells the rest — and a layout it
// has already said, or has just heard, is never said again.
export interface LayoutEcho {
  /** Tells every window the layout as it is now. Also the answer a newborn gets. */
  tell: () => void
  /** This window's own layout moved: said once, and not again for the same layout. */
  changed: () => void
  /** A layout heard from another window, or restored: taken, and counted as already said. */
  take: (layout: Layout) => void
}

export const layoutEcho = (): LayoutEcho => {
  // The layout this window has already told the others about, so that hearing it back — or
  // hearing it from somebody else — is not a reason to say it again.
  const said: { layout: Layout | null } = { layout: null }
  const tell = (): void => {
    void windowPort.broadcast({
      kind: WindowMessageKind.Layout,
      ...currentLayout(),
    })
  }
  return {
    tell,
    changed: () => {
      const now = currentLayout()
      if (sameLayout(said.layout, now)) return
      said.layout = now
      tell()
    },
    // **Taken as already said.** Setting it wakes this window's own watcher, which would
    // broadcast it back, which would wake everyone else's: one drag would cost a round of
    // messages per window. Recording it as said *before* setting it is what stops that, and it
    // does not depend on when the watcher happens to flush — a synchronous flag would.
    take: (layout) => {
      said.layout = layout
      setLayout(layout)
    },
  }
}
