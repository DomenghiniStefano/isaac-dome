import { clamp } from 'lodash-es'

export interface ProgressShares {
  /** Percent of the bar that is done. */
  value: number
  /** Percent of the bar that can't be read: never "not done", never part of `value`. */
  unknown: number
}

export interface ProgressBounds {
  value: number
  max: number
}

// Reka's own default: the max an unusable one falls back to.
export const defaultProgressMax = 100

const finiteOrZero = (n: number): number => (Number.isFinite(n) ? n : 0)

const isMeasurable = (max: number): boolean => Number.isFinite(max) && max > 0

// What the progress primitive is told, so its aria values agree with the drawn bar:
// never a value beyond max, never NaN, never an unusable max.
export const progressBounds = (value: number, max: number): ProgressBounds =>
  isMeasurable(max)
    ? { value: clamp(finiteOrZero(value), 0, max), max }
    : { value: 0, max: defaultProgressMax }

// The bar's two filled segments as percentages of max. Together they never pass 100, and
// a bar with nothing to measure is empty rather than NaN.
export const progressShares = (
  value: number,
  unknown: number,
  max: number,
): ProgressShares => {
  if (!isMeasurable(max)) return { value: 0, unknown: 0 }
  const done = clamp(finiteOrZero(value), 0, max)
  const unreadable = clamp(finiteOrZero(unknown), 0, max - done)
  return { value: (done / max) * 100, unknown: (unreadable / max) * 100 }
}
