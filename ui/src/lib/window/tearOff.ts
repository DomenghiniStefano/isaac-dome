import type { Box, Point } from '@/lib/drag/dragList'
import { remToPx } from '@/lib/scale/rows'
import type { WindowBox } from './windowPort'

// The top of a window: where its tab strip is drawn. **This band is the only landing a window
// offers** — a drop lower down is a drop on content, and content has no place to put a tab (see
// `stripUnderPoint`). In rem, like the title bar it has to cover (`--spacing-titlebar`): a fixed
// 40px was covered by a 45px strip at 150% and a 60px one at 200%, so a drop on the lower part
// of the strip opened a window (card 80, item 08). 2.5rem is the 40px it was at 100%.
const StripBandRem = 2.5

// The band at the interface's scale, in logical pixels.
export const stripBandPx = (percent: number): number =>
  remToPx(StripBandRem, percent)

// How far from the strip the pointer travels before the tab leaves the window. Only
// perpendicular travel counts: sliding far along the strip is how you reach its far end.
export const TearBand = 24

// Desktop physical pixels — what `cursorPosition()` answers — into one window's logical page
// pixels. Two monitors can disagree on the factor, so the window's own is the only one to use:
// `devicePixelRatio` is *this* window's, and this window may not be the one under the pointer.
export const toClient = (p: Point, w: WindowBox): Point => ({
  x: (p.x - w.left) / w.scaleFactor,
  y: (p.y - w.top) / w.scaleFactor,
})

export const toDesktop = (p: Point, w: WindowBox): Point => ({
  x: p.x * w.scaleFactor + w.left,
  y: p.y * w.scaleFactor + w.top,
})

export const holdsPoint = (w: WindowBox, p: Point): boolean =>
  p.x >= w.left &&
  p.x < w.left + w.width &&
  p.y >= w.top &&
  p.y < w.top + w.height

export const inStripBand = (w: WindowBox, p: Point, percent: number): boolean =>
  holdsPoint(w, p) && p.y < w.top + stripBandPx(percent) * w.scaleFactor

const rank = (order: string[], label: string): number => {
  const at = order.indexOf(label)
  return at < 0 ? order.length : at
}

// Which window's **strip** a point is over, and only a strip: a tab docks onto a bar of tabs,
// never onto a window's content. Answering for the whole window was wrong twice — the card that
// follows the cursor hides itself wherever a marker will speak instead, so it disappeared every
// time the cursor crossed any window; and a drop on a body has no place to land, so the rule had
// to invent one. Over a body the answer is the same as over the desktop: nothing, and a release
// there opens a window of its own.
//
// Two strips over one point can happen (a small window over another's title bar). There is no
// z-order API (tauri#5656), so the most recently focused wins — the only ordering we can observe.
export const stripUnderPoint = (
  windows: WindowBox[],
  p: Point,
  order: string[],
  percent: number,
): string | null => {
  const strips = windows.filter((w) => inStripBand(w, p, percent))
  if (strips.length === 0) return null
  const ranked = [...strips].sort(
    (a, b) => rank(order, a.label) - rank(order, b.label),
  )
  return ranked[0]?.label ?? null
}

// Has the tab left the strip? Measured perpendicular to the strip, from its own rectangle, in
// client pixels — the pointer is still inside the window when this first becomes true.
export const pastTearBand = (p: Point, strip: Box): boolean =>
  p.y > strip.top + strip.height + TearBand || p.y < strip.top - TearBand

// From the cursor to where a new window's top-left belongs, in this window's logical pixels:
// the grab inside the tab (`p` against the ghost's corner), plus where the strip sits inside a
// window. Read when the tab leaves, while the ghost and the strip are still on the page, so the
// window that opens is drawn **around the tab you are holding**.
export const grabOffset = (p: Point, ghost: Box, strip: Box): Point => ({
  x: p.x - ghost.left + strip.left,
  y: p.y - ghost.top + strip.top,
})

// Where a window opened for a tab dropped at `p` goes, so the tab lands under the cursor. The
// offset is in this window's logical pixels; the cursor speaks in the desktop's.
export const windowOrigin = (
  p: Point,
  offset: Point,
  scaleFactor: number,
): Point => ({
  x: Math.round(p.x - offset.x * scaleFactor),
  y: Math.round(p.y - offset.y * scaleFactor),
})
