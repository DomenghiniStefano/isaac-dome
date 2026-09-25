import { ref, shallowRef } from 'vue'
import type { Ref } from 'vue'
import { TabDrag, moveIndex, tabDropAt } from '@/lib/shell/tabs'
import type { TabDrop } from '@/lib/shell/tabs'
import { useDragList } from '@/composables/useDragList'
import type { DragList } from '@/composables/useDragList'
import { Axis, measure } from '@/lib/drag/dragList'
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
import { useSettingsStore } from '@/stores/settings'
import type { PointerWatch } from '@/lib/window/pointerSource'
import {
  grabOffset,
  pastTearBand,
  stripUnderPoint,
  toDesktop,
  windowOrigin,
} from '@/lib/window/tearOff'
import { windowPort } from '@/lib/window/windowPort'
import type { WindowBox } from '@/lib/window/windowPort'

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

// Every window and this one, read once when the tab leaves: windows do not move while a tab is
// over them, and asking the backend for the list on every frame would be a command per frame.
// `null` is a roster that could not be read — a closed window still listed answers
// `window not found` to every question, seen on the machine.
const measureWindows = async (): Promise<{
  all: WindowBox[]
  own: WindowBox
} | null> => {
  try {
    const [all, own] = await Promise.all([windowPort.list(), windowPort.self()])
    return { all, own }
  } catch {
    return null
  }
}

// The tab strip's drag, continued past the strip's edge. Inside the strip it is the reorder
// `useDragList` already does; past the tear band the DOM ghost gives way to a preview window
// that follows the cursor, and the release either joins the tab to the window under the point
// or opens one for it there.
export const useTabDrag = (options: TabDragOptions): TabDrag => {
  // The strip band is in rem, so it is measured at the interface's scale (card 80, item 08).
  const settings = useSettingsStore()
  const detached = ref(false)
  // What `measureWindows` read when the tab left.
  const windows = shallowRef<WindowBox[]>([])
  const own = shallowRef<WindowBox | null>(null)
  // One drag's worth of state, from the press to the release.
  const gesture: {
    pointer: PointerWatch | null
    hovered: string | null
    grabbed: number | null
    // Whether the card for this drag has been built yet. Reset when the drag ends, so the next
    // one builds its own — the label on the card is the tab being dragged.
    warmed: boolean
    // `grabOffset`, measured when the tab leaves.
    offset: Point
  } = {
    pointer: null,
    hovered: null,
    grabbed: null,
    warmed: false,
    offset: { x: 0, y: 0 },
  }

  const stripBox = (): Box | null => {
    const el = options.strip.value
    return el ? measure(el) : null
  }

  // Only one window is told at a time, and it is always told when the tab leaves it: a marker
  // left behind in a strip the tab is no longer over is a lie about where it will land.
  const tellHovered = (label: string | null, at: Point) => {
    if (gesture.hovered && gesture.hovered !== label)
      void windowPort.send(gesture.hovered, {
        kind: WindowMessageKind.HoverLeft,
      })
    gesture.hovered = label
    if (label)
      void windowPort.send(label, { kind: WindowMessageKind.Hovering, at })
  }

  const targets = (): WindowBox[] =>
    windows.value.filter((w) => w.label !== PreviewLabel)

  // Ends the detached half of the gesture without deciding what becomes of the tab.
  const stopWatching = () => {
    detached.value = false
    gesture.pointer?.stop()
    gesture.pointer = null
    void hidePreview()
    tellHovered(null, { x: 0, y: 0 })
    gesture.grabbed = null
    gesture.warmed = false
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
    const target = stripUnderPoint(
      targets(),
      p,
      focusOrder.value,
      settings.scale,
    )
    tellHovered(target, p)
    // **The card never leaves the cursor until the release** (owner, 2026-09-13). It used to
    // hide itself over a strip, on the grounds that the marker there says the same thing — but
    // the two say different things: the marker says *where among these tabs*, the card says
    // *what you are holding*, and the hand holding it should never look empty.
    void movePreview(p)
  }

  const onOutsideRelease = (p: Point) => {
    const target = gesture.hovered
    // **The target keeps its marker until the tab lands on it.** Forgetting the hover first
    // tells it "the tab left" — and with it goes the index the marker was pointing at, so the
    // tab arrived unaimed and was appended to the end, wherever it had been dropped. Every
    // time, which is how the owner found it. Clearing `hovered` here means `stopWatching`
    // has nothing to take back.
    gesture.hovered = null
    const origin = windowOrigin(p, gesture.offset, own.value?.scaleFactor ?? 1)
    stopWatching()
    options.settle(target, p, origin)
  }

  // Where the hand is holding the tab, and where a tab sits inside a window: both read from the
  // page while they still exist, because a moment later the tab is gone from the strip.
  const rememberGrab = (p: Point): void => {
    const ghost = drag.ghost.value
    const strip = stripBox()
    if (ghost && strip) gesture.offset = grabOffset(p, ghost, strip)
  }

  const detach = async (p: Point) => {
    const grabbed = gesture.grabbed
    if (detached.value || grabbed === null) return
    // The tab leaves the strip here, not at the release: from now on it is in flight.
    const label = options.labelOf(grabbed)
    rememberGrab(p)
    if (!options.lift(grabbed)) return
    detached.value = true
    // Anything that goes wrong between the tab leaving the strip and the watch being installed
    // leaves the tab in flight with nobody holding it: put it back rather than lose it. This
    // is not caution — it is the shape of a real failure, seen on the machine.
    const measured = await measureWindows()
    if (measured === null) {
      abort()
      return
    }
    windows.value = measured.all
    own.value = measured.own
    // **The watch first, the picture second.** The preview is what the gesture looks like; the
    // watch is what the gesture *is*. Waiting for a window to be created before listening for
    // the release means a preview that never opens takes the whole gesture down with it — no
    // error, no window, nothing at all. Measured on the machine, 2026-09-13.
    gesture.pointer = watchPointer({
      onMove: onOutsideMove,
      onRelease: onOutsideRelease,
      // The webview went silent, or the cursor cannot be read: put the tab back rather than
      // land it somewhere nobody released it.
      onLost: abort,
    })
    const desktop = toDesktop(p, measured.own)
    void showPreview(label, desktop).catch(() => undefined)
  }

  // The card is built while the drag is still a reorder, so that leaving the strip costs
  // nothing but showing it. Once per drag, and idempotent besides.
  const warm = (from: number) => {
    if (gesture.warmed) return
    gesture.warmed = true
    void warmPreview(options.labelOf(from)).catch(() => undefined)
  }

  // What a move does beyond aiming a reorder: past the tear band the tab leaves the strip, and
  // from then on the drag is the window's and no drop inside the strip is resolved.
  const onMove = (p: Point, from: number): boolean => {
    gesture.grabbed = from
    warm(from)
    const strip = stripBox()
    if (!detached.value && strip && pastTearBand(p, strip)) {
      void detach(p)
      return true
    }
    return detached.value
  }

  const drag = useDragList<TabDrop>({
    axis: Axis.X,
    container: options.strip,
    items: options.items,
    threshold: TabDrag.Threshold,
    onMove,
    resolve: (p, boxes, from) => tabDropAt(boxes, p, from),
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
