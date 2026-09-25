import { onBeforeUnmount, ref, shallowRef } from 'vue'
import type { Ref } from 'vue'
import { EventKey } from '@/lib/constants/eventKeys'
import type { Axis, Box, GhostBox, Point } from '@/lib/drag/dragList'
import {
  DragThreshold,
  boxOf,
  crossedThreshold,
  ghostOrigin,
  grabOffset,
} from '@/lib/drag/dragList'

// The choreography every list drag in the app shares: a press becomes a drag past the
// threshold and only then is the pointer captured (a capture from the first pixel would send
// the click to the container instead of the item), the rectangles are read once, the drop is
// recomputed on every move, and nothing moves until the release.
//
// What a drop *means* is the caller's: `resolve` turns a point into whatever that list calls a
// landing, `commit` applies it. The queue and the tab strip keep their own pure functions.
export interface DragListOptions<D> {
  axis: Axis
  // The element that takes the capture and hears the moves.
  container: Ref<HTMLElement | null>
  // The draggable items, in order, read at the moment the drag starts.
  items: () => HTMLElement[]
  resolve: (p: Point, boxes: Box[], from: number) => D | null
  commit: (from: number, drop: D | null) => void
  // The drag was **called off** — Escape, or the pointer cancelled — as opposed to released.
  // The two cannot be told apart by watching whether a drag is still running: that goes false
  // either way, and a caller who guessed from it undid its own successful drops. Measured on
  // the machine, 2026-09-13.
  cancelled?: () => void
  // A busy screen refuses to start a drag; nothing else stops one.
  enabled?: () => boolean
  threshold?: number
}

export interface DragList<D> {
  moving: Ref<boolean>
  from: Ref<number | null>
  drop: Ref<D | null>
  ghost: Ref<GhostBox | null>
  start: (index: number, e: PointerEvent) => void
  move: (e: PointerEvent) => void
  end: () => void
  cancel: () => void
}

const measure = (el: HTMLElement): Box => boxOf(el.getBoundingClientRect())

export const useDragList = <D>(options: DragListOptions<D>): DragList<D> => {
  const moving = ref(false)
  const from = ref<number | null>(null)
  const drop = shallowRef<D | null>(null)
  const ghost = shallowRef<GhostBox | null>(null)

  let press: Point | null = null
  let offset: Point = { x: 0, y: 0 }
  let size: Point = { x: 0, y: 0 }
  let boxes: Box[] = []
  let captured: number | null = null

  // A drag, once started, can be called off: Escape puts everything back and commits nothing.
  // Without it the only way out of a drag begun by accident is to drop it somewhere.
  const onKeydown = (e: KeyboardEvent) => {
    if (e.key !== EventKey.Escape) return
    e.preventDefault()
    const wasMoving = moving.value
    clear()
    if (wasMoving) options.cancelled?.()
  }

  function clear() {
    moving.value = false
    from.value = null
    drop.value = null
    ghost.value = null
    press = null
    boxes = []
    const el = options.container.value
    if (captured !== null && el?.hasPointerCapture(captured))
      el.releasePointerCapture(captured)
    captured = null
    window.removeEventListener('keydown', onKeydown)
  }

  const begin = (p: Point) => {
    const index = from.value
    const el = index === null ? undefined : options.items()[index]
    if (index === null || !el) return clear()
    boxes = options.items().map(measure)
    const box = measure(el)
    offset = grabOffset(box, press ?? p)
    size = { x: box.width, y: box.height }
    moving.value = true
    window.addEventListener('keydown', onKeydown)
  }

  const draw = (p: Point) => {
    const origin = ghostOrigin(offset, p)
    ghost.value = {
      left: origin.x,
      top: origin.y,
      width: size.x,
      height: size.y,
    }
  }

  const start = (index: number, e: PointerEvent) => {
    if (e.button !== 0 || options.enabled?.() === false) return
    from.value = index
    press = { x: e.clientX, y: e.clientY }
  }

  const move = (e: PointerEvent) => {
    if (from.value === null || !press) return
    const p = { x: e.clientX, y: e.clientY }
    if (!moving.value) {
      if (!crossedThreshold(press, p, options.threshold ?? DragThreshold))
        return
      begin(p)
      if (!moving.value) return
      options.container.value?.setPointerCapture(e.pointerId)
      captured = e.pointerId
    }
    draw(p)
    drop.value = options.resolve(p, boxes, from.value)
  }

  const end = () => {
    const index = from.value
    const landing = drop.value
    const dragged = moving.value
    clear()
    if (dragged && index !== null) options.commit(index, landing)
  }

  onBeforeUnmount(clear)

  return { moving, from, drop, ghost, start, move, end, cancel: clear }
}
