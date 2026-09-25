import { EventKey } from '@/lib/constants/eventKeys'
import { Axis, boxAt } from '@/lib/drag/dragList'
import type { Box, Point } from '@/lib/drag/dragList'
import type { WindowBox } from '@/lib/window/windowPort'

// A tab's origin, drawn as an icon on the tab itself (DESIGN-BRIEF.md §4.2): the mixed bar
// stays readable because every tab says which part of the app it comes from.
export const TabOrigin = {
  Search: 'search',
  Wiki: 'wiki',
  Progress: 'progress',
  Tool: 'tool',
  Settings: 'settings',
} as const
export type TabOrigin = (typeof TabOrigin)[keyof typeof TabOrigin]

// What a tab shows. A view type of the shell, not an IPC one: cycle 3 builds it from the
// tab model.
export interface TabView {
  id: string
  label: string
  origin: TabOrigin
}

// The ARIA role a tab's root carries. TabItem sets it and TabStrip finds tabs by it, so the
// two can't drift apart.
export const TabRole = 'tab'

// A tab from another window, hovering over this strip. The point is in desktop pixels —
// the sender cannot know our scale factor — and `window` is this window's own geometry, which
// is the only thing that can convert it (`lib/window/tearOff.ts`).
export interface IncomingHover {
  at: Point
  window: WindowBox
}

export const DropSide = { Before: 'before', After: 'after' } as const
export type DropSide = (typeof DropSide)[keyof typeof DropSide]

// Pixels the pointer travels before a press on a tab becomes a drag: below it, a click.
export const TabDrag = { Threshold: 4 } as const

// Which side of the tab under the pointer a dropped tab lands on: the half decides, and
// the exact middle counts as after.
export const dropSide = (
  pointerX: number,
  left: number,
  width: number,
): DropSide => (pointerX < left + width / 2 ? DropSide.Before : DropSide.After)

// The index a tab ends at when moved from `from` to one side of the tab at `target`. Both
// are positions before the move; taking the tab out first shifts everything after it left.
export const moveIndex = (
  from: number,
  target: number,
  side: DropSide,
): number => {
  if (from === target) return from
  const slot = side === DropSide.Before ? target : target + 1
  return slot > from ? slot - 1 : slot
}

// Where a tab arriving from another window would land, from the point it hovers at and the
// strip's tabs: beside the tab under it, by its half, or at the end past every tab. The gap
// the marker is drawn in **is** where the drop lands — one computation, so what you saw is what
// you get.
export const arrivalGap = (boxes: Box[], p: Point, count: number): number => {
  const index = boxAt(boxes, p, Axis.X)
  const box = index === null ? undefined : boxes[index]
  if (index === null || !box) return count
  return dropSide(p.x, box.left, box.width) === DropSide.Before
    ? index
    : index + 1
}

// The tab the arrow keys move to from the active one, stopping at either end. `null` for any
// other key, or with no active tab in the strip.
export const neighbourIndex = (
  ids: string[],
  activeId: string | null,
  key: string,
): number | null => {
  const index = ids.findIndex((id) => id === activeId)
  if (index < 0) return null
  if (key === EventKey.ArrowRight) return Math.min(index + 1, ids.length - 1)
  if (key === EventKey.ArrowLeft) return Math.max(index - 1, 0)
  return null
}

// Where a tab being reordered would land: a side of the tab under the pointer. Nothing over the
// tab being moved, which is where it already is, nor past the strip's tabs.
export interface TabDrop {
  index: number
  side: DropSide
}

export const tabDropAt = (
  boxes: Box[],
  p: Point,
  from: number,
): TabDrop | null => {
  const index = boxAt(boxes, p, Axis.X)
  const box = index === null ? undefined : boxes[index]
  if (index === null || index === from || !box) return null
  return { index, side: dropSide(p.x, box.left, box.width) }
}
