import type { Box, Point } from '@/lib/drag/dragList'
import type { WindowBox } from './windowPort'

// The top of a window, in its own logical pixels: where its tab strip is drawn. A drop there
// joins that window's strip; a drop lower down is a drop on a window, which means the same
// thing but reads as less deliberate — so the band wins ties in `windowUnderPoint`.
export const StripBand = 40

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

export const inStripBand = (w: WindowBox, p: Point): boolean =>
  holdsPoint(w, p) && p.y < w.top + StripBand * w.scaleFactor

const rank = (order: string[], label: string): number => {
  const at = order.indexOf(label)
  return at < 0 ? order.length : at
}

// Which window a point belongs to. There is no z-order API (tauri#5656), so overlapping windows
// are ambiguous and the rule is ours: a strip beats a body, and among equals the most recently
// focused wins. With nothing to go on it still answers a window rather than null — a drop that
// lands somewhere beats a drop that vanishes.
export const windowUnderPoint = (
  windows: WindowBox[],
  p: Point,
  order: string[],
): string | null => {
  const holding = windows.filter((w) => holdsPoint(w, p))
  if (holding.length === 0) return null
  const strips = holding.filter((w) => inStripBand(w, p))
  const candidates = strips.length > 0 ? strips : holding
  const ranked = [...candidates].sort(
    (a, b) => rank(order, a.label) - rank(order, b.label),
  )
  return ranked[0]?.label ?? null
}

// Has the tab left the strip? Measured perpendicular to the strip, from its own rectangle, in
// client pixels — the pointer is still inside the window when this first becomes true.
export const pastTearBand = (p: Point, strip: Box): boolean =>
  p.y > strip.top + strip.height + TearBand || p.y < strip.top - TearBand
