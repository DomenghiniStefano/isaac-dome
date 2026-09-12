import { scaleFactor } from './steps'

// The base every rem token is measured against, and the one place it is written on the
// TypeScript side: `base.css` has the same 16 in `calc(16px * var(--app-scale))`.
export const RootFontPx = 16

// How tall a virtualized row is, in device pixels, at a given scale. A virtualizer
// positions its rows with a **number**, while they are drawn with a token: at any scale but
// 100 a fixed 40 would stack 40px apart rows that are 80px tall, and nothing would fail —
// the rows would simply overlap. So the number is derived from the same value the token is.
export const RowWideRem = 2.5

// A search result carries a second line: taller than a table row, and measured the same way.
export const RowResultRem = 3.5

export const remToPx = (rem: number, percent: number): number =>
  rem * RootFontPx * scaleFactor(percent)

export const rowWidePx = (percent: number): number =>
  remToPx(RowWideRem, percent)

export const rowResultPx = (percent: number): number =>
  remToPx(RowResultRem, percent)
