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

// Where the lifted copy is drawn, in client pixels.
export type GhostBox = Box

// Pixels the pointer travels before a press becomes a drag: below it, a click. Both screens had
// chosen 4 independently.
export const DragThreshold = 4

const travel = (axis: Axis, from: Point, to: Point): number => {
  switch (axis) {
    case Axis.X:
      return Math.abs(to.x - from.x)
    case Axis.Y:
      return Math.abs(to.y - from.y)
    default:
      return assertNever(axis)
  }
}

export const crossedThreshold = (
  axis: Axis,
  from: Point,
  to: Point,
  threshold: number,
): boolean => travel(axis, from, to) >= threshold

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
