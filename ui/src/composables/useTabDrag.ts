import { ref, shallowRef } from 'vue'
import type { Ref } from 'vue'
import { TabDrag, dropSide, moveIndex } from '@/components/shell/tabs'
import type { DropSide } from '@/components/shell/tabs'
import { useDragList } from '@/composables/useDragList'
import type { DragList } from '@/composables/useDragList'
import { Axis, boxAt } from '@/lib/drag/dragList'
import type { Box, Point } from '@/lib/drag/dragList'
import { focusOrder } from '@/lib/window/focusOrder'
import { WindowMessageKind } from '@/lib/window/messages'
import {
  PreviewLabel,
  hidePreview,
  movePreview,
  showPreview,
  warmPreview,
} from '@/lib/window/preview'
import { watchPointer } from '@/lib/window/pointerSource'
import type { PointerWatch } from '@/lib/window/pointerSource'
import { pastTearBand, stripUnderPoint, toDesktop } from '@/lib/window/tearOff'
import { windowPort } from '@/lib/window/windowPort'
import type { WindowBox } from '@/lib/window/windowPort'

export interface TabDrop {
  index: number
  side: DropSide
}

export interface TabDragOptions {
  strip: Ref<HTMLElement | null>
  items: () => HTMLElement[]
  labelOf: (index: number) => string
  reorder: (from: number, to: number) => void
  // Takes the tab at `index` out of the strip: from here on it is in flight and belongs to no
  // window. False when there was no such tab.
  lift: (index: number) => boolean
  // The drag ended. A label lands the tab on that window's strip — this window's own included,
  // since the tab has already left it — and null lands it on the bare desktop. `at` is where the
  // cursor was, which is what a strip needs; `origin` is where a **new window's top-left** goes
  // so that the tab you were holding ends up under the cursor rather than the window's corner.
  settle: (target: string | null, at: Point, origin: Point) => void
  // The drag was called off or lost: the tab goes back where it sat.
  putBack: () => void
}

export interface TabDrag {
  drag: DragList<TabDrop>
  detached: Ref<boolean>
}

// The tab strip's drag, continued past the strip's edge. Inside the strip it is the reorder
// `useDragList` already does; past the tear band the DOM ghost gives way to a preview window
// that follows the cursor, and the release either joins the tab to the window under the point
// or opens one for it there.
export const useTabDrag = (options: TabDragOptions): TabDrag => {
  const detached = ref(false)
  // Read once when the tab leaves: windows do not move while a tab is over them, and asking
  // the backend for the list on every frame would be a command per frame.
  const windows = shallowRef<WindowBox[]>([])
  const self = shallowRef<WindowBox | null>(null)
  let pointer: PointerWatch | null = null
  let hovered: string | null = null
  let grabbed: number | null = null
  // Whether the card for this drag has been built yet. Reset when the drag ends, so the next
  // one builds its own — the label on the card is the tab being dragged.
  let warmed = false
  // From the cursor to where a new window's top-left belongs, in this window's logical pixels:
  // the grab inside the tab, plus where the first tab sits inside a window. Measured when the
  // tab leaves, so the window that opens is drawn **around the tab you are holding**.
  let tabOffset: Point = { x: 0, y: 0 }

  const stripBox = (): Box | null => {
    const el = options.strip.value
    if (!el) return null
    const r = el.getBoundingClientRect()
    return { left: r.left, top: r.top, width: r.width, height: r.height }
  }

  // Only one window is told at a time, and it is always told when the tab leaves it: a marker
  // left behind in a strip the tab is no longer over is a lie about where it will land.
  const tellHovered = (label: string | null, at: Point) => {
    if (hovered && hovered !== label)
      void windowPort.send(hovered, { kind: WindowMessageKind.HoverLeft })
    hovered = label
    if (label)
      void windowPort.send(label, { kind: WindowMessageKind.Hovering, at })
  }

  const targets = (): WindowBox[] =>
    windows.value.filter((w) => w.label !== PreviewLabel)

  // Ends the detached half of the gesture without deciding what becomes of the tab.
  const stopWatching = () => {
    detached.value = false
    pointer?.stop()
    pointer = null
    void hidePreview()
    tellHovered(null, { x: 0, y: 0 })
    grabbed = null
    warmed = false
    drag.cancel()
  }

  // The drag died on us — Escape, a lost pointer, a silent webview. The tab is in flight and
  // belongs to nobody: it goes back where it sat rather than landing where nobody dropped it.
  const abort = () => {
    const wasDetached = detached.value
    stopWatching()
    if (wasDetached) options.putBack()
  }

  const onOutsideMove = (p: Point) => {
    // A **strip** under the cursor, not a window: over a window's content there is no landing,
    // so nothing is aimed. Our own strip counts like any other — the tab has already left it,
    // so putting it back there is a landing and not a special case.
    const target = stripUnderPoint(targets(), p, focusOrder.value)
    tellHovered(target, p)
    // **The card never leaves the cursor until the release** (owner, 2026-09-13). It used to
    // hide itself over a strip, on the grounds that the marker there says the same thing — but
    // the two say different things: the marker says *where among these tabs*, the card says
    // *what you are holding*, and the hand holding it should never look empty.
    void movePreview(p)
  }

  const onOutsideRelease = (p: Point) => {
    const target = hovered
    // **The target keeps its marker until the tab lands on it.** Forgetting the hover first
    // tells it "the tab left" — and with it goes the index the marker was pointing at, so the
    // tab arrived unaimed and was appended to the end, wherever it had been dropped. Every
    // time, which is how the owner found it. Clearing `hovered` here means `stopWatching`
    // has nothing to take back.
    hovered = null
    // The window a drop on the desktop opens is placed so the tab lands under the cursor. The
    // offset was measured in this window's logical pixels; the cursor speaks in the desktop's.
    const factor = self.value?.scaleFactor ?? 1
    const origin = {
      x: Math.round(p.x - tabOffset.x * factor),
      y: Math.round(p.y - tabOffset.y * factor),
    }
    stopWatching()
    options.settle(target, p, origin)
  }

  const detach = async (p: Point) => {
    if (detached.value || grabbed === null) return
    // The tab leaves the strip here, not at the release: from now on it is in flight.
    const label = options.labelOf(grabbed)
    // Where the hand is holding the tab, and where a tab sits inside a window: both read from
    // the page while they still exist, because a moment later the tab is gone from the strip.
    const ghost = drag.ghost.value
    const strip = stripBox()
    if (ghost && strip)
      tabOffset = {
        x: p.x - ghost.left + strip.left,
        y: p.y - ghost.top + strip.top,
      }
    if (!options.lift(grabbed)) return
    detached.value = true
    // Anything that goes wrong between the tab leaving the strip and the watch being installed
    // leaves the tab in flight with nobody holding it: put it back rather than lose it. This
    // is not caution — it is the shape of a real failure, seen on the machine (a closed window
    // still listed, answering `window not found` to every question).
    let all: WindowBox[]
    let mine: WindowBox
    try {
      ;[all, mine] = await Promise.all([windowPort.list(), windowPort.self()])
    } catch {
      abort()
      return
    }
    windows.value = all
    self.value = mine
    // **The watch first, the picture second.** The preview is what the gesture looks like; the
    // watch is what the gesture *is*. Waiting for a window to be created before listening for
    // the release means a preview that never opens takes the whole gesture down with it — no
    // error, no window, nothing at all. Measured on the machine, 2026-09-13.
    pointer = watchPointer({
      onMove: onOutsideMove,
      onRelease: onOutsideRelease,
      // The webview went silent, or the cursor cannot be read: put the tab back rather than
      // land it somewhere nobody released it.
      onLost: abort,
    })
    const desktop = toDesktop(p, mine)
    void showPreview(label, desktop).catch(() => undefined)
  }

  const resolve = (p: Point, boxes: Box[], from: number): TabDrop | null => {
    grabbed = from
    // The card is built while the drag is still a reorder, so that leaving the strip costs
    // nothing but showing it. Once per drag, and idempotent besides.
    if (!warmed) {
      warmed = true
      void warmPreview(options.labelOf(from)).catch(() => undefined)
    }
    const strip = stripBox()
    if (!detached.value && strip && pastTearBand(p, strip)) {
      void detach(p)
      return null
    }
    if (detached.value) return null
    const index = boxAt(boxes, p, Axis.X)
    const box = index === null ? undefined : boxes[index]
    if (index === null || index === from || !box) return null
    return { index, side: dropSide(p.x, box.left, box.width) }
  }

  const drag = useDragList<TabDrop>({
    axis: Axis.X,
    container: options.strip,
    items: options.items,
    threshold: TabDrag.Threshold,
    resolve,
    commit: (from, landing) => {
      // A drag that left the window has already landed the tab, or put it back.
      if (detached.value || !landing) return
      const to = moveIndex(from, landing.index, landing.side)
      if (to !== from) options.reorder(from, to)
    },
    // Escape while the tab is out of the strip: it has nowhere to be, so it goes back. Only a
    // real cancellation reaches here — a release, inside or outside, does not.
    cancelled: () => {
      if (detached.value) abort()
    },
  })

  return { drag, detached }
}
