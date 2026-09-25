import { assertNever } from '@/lib/assertNever'

// Which way the list runs. A strip of tabs runs along x, a queue of rows along y, and the
// gesture reads travel and hit tests on that axis only: dragging a tab downwards is not a
// reorder, it is the tear-off
// (`docs/superpowers/specs/2026-09-13-drag-and-windows-design.md` §3).
export const Axis = { X: 'x', Y: 'y' } as const
export type Axis = (typeof Axis)[keyof typeof Axis]

export interface Point {
  x: number
  y: number
}

// A rectangle as the gesture needs it: the DOMRect of an item, snapshotted once when the press
// becomes a drag. Nothing moves until the release, so reading it per pointermove would be
// waste — and a layout read per item per frame is the waste that shows.
export interface Box {
  left: number
  top: number
  width: number
  height: number
}

// The box of a measured rectangle, and nothing else of it: a `DOMRect` also carries `x`, `y`,
// `right`, `bottom` and `toJSON`, and a snapshot is the four numbers a hit test reads. The
// caller measures (`el.getBoundingClientRect()`); this only copies.
export const boxOf = (r: Box): Box => ({
  left: r.left,
  top: r.top,
  width: r.width,
  height: r.height,
})

// Where the lifted copy is drawn, in client pixels.
export type GhostBox = Box

// Pixels the pointer travels before a press becomes a drag: below it, a click. Both screens had
// chosen 4 independently.
export const DragThreshold = 4

// A press becomes a drag on travel in **any** direction, and the list's axis has nothing to do
// with it. It used to read the axis alone, and that was wrong in the one case that matters:
// the strip runs along x, the tear-off is a vertical gesture, so dragging a tab straight down
// out of the window never started a drag at all. Measured on the machine, 2026-09-13. The axis
// still decides the hit test; it does not decide what a drag is.
export const crossedThreshold = (
  from: Point,
  to: Point,
  threshold: number,
): boolean => Math.hypot(to.x - from.x, to.y - from.y) >= threshold

const holds = (box: Box, p: Point, axis: Axis): boolean => {
  switch (axis) {
    case Axis.X:
      return p.x >= box.left && p.x < box.left + box.width
    case Axis.Y:
      return p.y >= box.top && p.y < box.top + box.height
    default:
      return assertNever(axis)
  }
}

// Which snapshotted box the point falls in, along the list's axis only. Null between the items
// or past the ends, which is a real answer: a drop there means something else.
export const boxAt = (boxes: Box[], p: Point, axis: Axis): number | null => {
  for (const [index, box] of boxes.entries())
    if (holds(box, p, axis)) return index
  return null
}

// Where inside the grabbed item the press landed, and the top-left that keeps it there while the
// pointer moves: a lifted row that jumps so its corner meets the cursor reads as a different row.
export const grabOffset = (box: Box, press: Point): Point => ({
  x: press.x - box.left,
  y: press.y - box.top,
})

export const ghostOrigin = (offset: Point, p: Point): Point => ({
  x: p.x - offset.x,
  y: p.y - offset.y,
})
