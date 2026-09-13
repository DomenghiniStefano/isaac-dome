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
} from '@/lib/window/preview'
import { watchPointer } from '@/lib/window/pointerSource'
import type { PointerWatch } from '@/lib/window/pointerSource'
import { pastTearBand, toDesktop, windowUnderPoint } from '@/lib/window/tearOff'
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
  // A window holding one tab refuses to open a second one for it.
  canTear: () => boolean
  reorder: (from: number, to: number) => void
  // The tab at `index` joins that window, at that desktop point.
  giveTo: (index: number, label: string, at: Point) => void
  // The tab at `index` gets a window of its own, its top-left at that desktop point.
  openWith: (index: number, at: Point) => void
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
  let watch: PointerWatch | null = null
  let hovered: string | null = null
  let grabbed: number | null = null

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

  const attach = () => {
    detached.value = false
    watch?.stop()
    watch = null
    void hidePreview()
  }

  const finish = () => {
    tellHovered(null, { x: 0, y: 0 })
    attach()
    grabbed = null
    drag.cancel()
  }

  const onOutsideMove = (p: Point) => {
    const mine = self.value
    const target = windowUnderPoint(targets(), p, focusOrder.value)
    // Back over our own window: the tab comes home and the gesture is a reorder again. The
    // composable never stopped following the pointer inside, so nothing has to be restarted.
    if (mine && target === mine.label) {
      tellHovered(null, p)
      attach()
      return
    }
    tellHovered(target, p)
    if (target) void hidePreview()
    else void movePreview(p)
  }

  const onOutsideRelease = (p: Point) => {
    const index = grabbed
    const target = hovered
    finish()
    if (index === null) return
    if (target) options.giveTo(index, target, p)
    else options.openWith(index, p)
  }

  const detach = async (p: Point) => {
    if (detached.value || grabbed === null || !options.canTear()) return
    detached.value = true
    const [all, mine] = await Promise.all([
      windowPort.list(),
      windowPort.self(),
    ])
    windows.value = all
    self.value = mine
    const desktop = toDesktop(p, mine)
    await showPreview(options.labelOf(grabbed), desktop)
    watch = watchPointer({
      onMove: onOutsideMove,
      onRelease: onOutsideRelease,
      // The webview went silent, or the cursor cannot be read: put the tab back rather than
      // land it somewhere nobody released it.
      onLost: finish,
    })
  }

  const resolve = (p: Point, boxes: Box[], from: number): TabDrop | null => {
    grabbed = from
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
  })

  return { drag, detached }
}
