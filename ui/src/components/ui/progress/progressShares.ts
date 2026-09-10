import { clamp } from 'lodash-es'

export interface ProgressShares {
  /** Percent of the bar that is done. */
  value: number
  /** Percent of the bar that can't be read: never "not done", never part of `value`. */
  unknown: number
}

// The bar's two filled segments as percentages of max. Together they never pass 100, and
// a bar with nothing to measure is empty rather than NaN.
export const progressShares = (
  value: number,
  unknown: number,
  max: number,
): ProgressShares => {
  if (max <= 0) return { value: 0, unknown: 0 }
  const done = clamp(value, 0, max)
  const unreadable = clamp(unknown, 0, max - done)
  return { value: (done / max) * 100, unknown: (unreadable / max) * 100 }
}
