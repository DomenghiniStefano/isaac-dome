import { EventKey } from '@/lib/constants/eventKeys'
import { assertNever } from '@/lib/assertNever'

// Where a dragged row goes, named the way the backend takes it: right below a row, or the top.
// Never an index: the view leaves completed and unresolved rows out, so its positions are not
// the saved queue's.
export interface Anchor {
  after: number | null
}

export const DropEdge = { Above: 'above', Below: 'below' } as const
export type DropEdge = (typeof DropEdge)[keyof typeof DropEdge]

export const StepDirection = { Up: 'up', Down: 'down' } as const
export type StepDirection = (typeof StepDirection)[keyof typeof StepDirection]

// The half of the row under the pointer decides; the exact middle counts as below.
export const dropEdge = (
  pointerY: number,
  top: number,
  height: number,
): DropEdge => (pointerY < top + height / 2 ? DropEdge.Above : DropEdge.Below)

// A drop on one edge of the row at `target` fills the gap above or below it. The gaps right
// above and right below the dragged row are where it already is: nothing to send.
export const dropAnchor = (
  ids: number[],
  from: number,
  target: number,
  edge: DropEdge,
): Anchor | null => {
  if (from < 0 || from >= ids.length || target < 0 || target >= ids.length)
    return null
  const gap = edge === DropEdge.Above ? target : target + 1
  if (gap === from || gap === from + 1) return null
  return { after: gap === 0 ? null : (ids[gap - 1] ?? null) }
}

// Alt+arrow on a row: the anchor a drop on the neighbour's far edge would give.
export const stepAnchor = (
  ids: number[],
  index: number,
  direction: StepDirection,
): Anchor | null => {
  switch (direction) {
    case StepDirection.Up:
      return dropAnchor(ids, index, index - 1, DropEdge.Above)
    case StepDirection.Down:
      return dropAnchor(ids, index, index + 1, DropEdge.Below)
    default:
      return assertNever(direction)
  }
}

// The keyboard's drag: Alt with an arrow moves the row one step. A bare arrow is not a move —
// it scrolls the page, as it would anywhere else.
export const stepDirection = (
  key: string,
  alt: boolean,
): StepDirection | null => {
  if (!alt) return null
  if (key === EventKey.ArrowUp) return StepDirection.Up
  if (key === EventKey.ArrowDown) return StepDirection.Down
  return null
}
