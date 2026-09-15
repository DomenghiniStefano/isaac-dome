import type { Point } from '@/lib/drag/dragList'
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
