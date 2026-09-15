// Where a list was scrolled to, and the list it was measured against. The second number is what
// makes the first mean anything: an offset is a distance into a list of a given length, and the
// same distance into a different list is a different place.
export interface ScrollOffset {
  top: number
  rows: number
}

const isCount = (value: unknown): value is number =>
  typeof value === 'number' && Number.isFinite(value) && value >= 0

export const readScrollOffset = (value: unknown): ScrollOffset | null => {
  if (typeof value !== 'object' || value === null) return null
  const { top, rows } = value as { top?: unknown; rows?: unknown }
  return isCount(top) && isCount(rows) ? { top, rows } : null
}

// What to scroll to, or nothing at all. Nothing is the honest answer for a list that changed
// underneath: the top of a list you can read beats a position in a list that is not the one the
// number came from.
export const offsetToApply = (
  stored: ScrollOffset | null,
  rows: number,
): number | null =>
  stored !== null && stored.rows === rows ? stored.top : null
