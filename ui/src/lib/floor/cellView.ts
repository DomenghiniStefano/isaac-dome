import { TargetView } from '@/lib/ipc/types'
import type { FloorSolutionView } from '@/lib/ipc/types'
import { WIDTH, xy } from './painting'

// What one cell of the grid is allowed to say (B64).
//
// Before this, a cell said one thing: a rank, or "painted", or "empty" — so fourteen room
// kinds came out as one grey square, and of the three targets only the Secret Room ever
// reached the map. It now says two: **what you drew** (a colour and a drawing, both in the
// component) and **what the rules make of it**, which is this file.
//
// The rules' half was four small squares in the corners, and that was wrong twice over: four
// pips on a 2rem square print four numbers too small to read, and a cell nobody painted still
// looked unpainted. **A cell a rule allows is filled with that rule's colour** — one target
// fills it whole, two split it in half, three in thirds, always left to right in the reading
// order below. The corner is gone, and with it `Corner`, `cornerOf` and `cornerAt`: the width
// of a band says how many targets want that cell, which the corners never could.

/** The order the targets are read in, and therefore the order the bands are laid out in. */
const targetOrder: readonly TargetView[] = [
  TargetView.Secret,
  TargetView.SuperSecret,
  TargetView.UltraSecret,
]

/**
 * Three steps, and the third is where the scale stops. The rules order candidates; they do
 * **not** say how much less likely the fourth is than the third, so neither does the ramp.
 */
export const RankStep = {
  First: 'first',
  Second: 'second',
  Third: 'third',
} as const
export type RankStep = (typeof RankStep)[keyof typeof RankStep]

export const rankStep = (rank: number): RankStep => {
  if (rank <= 0) return RankStep.First
  if (rank === 1) return RankStep.Second
  return RankStep.Third
}

/** One target's claim on one cell: the slice of it that target colours, and what it prints. */
export interface Band {
  readonly target: TargetView
  readonly step: RankStep
  /** One-based: the number the cell prints, and the one the legend explains. */
  readonly rank: number
}

/**
 * What a cell shows of the rules, for the targets currently switched on. The bands share the
 * cell equally, so an empty list is a cell the rules say nothing about and a list of one is a
 * cell filled edge to edge.
 *
 * The order is the reading order above, never the order the solutions arrived in: a cell two
 * targets claim has to look the same whichever answer landed first.
 */
export const bandsFor = (
  cell: number,
  solutions: readonly FloorSolutionView[],
  shown: readonly TargetView[],
): Band[] => {
  const bands: Band[] = []
  for (const target of targetOrder) {
    if (!shown.includes(target)) continue
    const solution = solutions.find((one) => one.target === target)
    const candidate = solution?.candidates.find((one) => one.cell === cell)
    if (candidate === undefined) continue
    bands.push({
      target,
      step: rankStep(candidate.rank),
      rank: candidate.rank + 1,
    })
  }
  return bands
}

export interface CellPosition {
  readonly row: number
  readonly column: number
}

/**
 * A cell index is not a position. "Cell 97" connects to nothing on a 13x13 grid, which is why
 * reading a candidate row used to light nothing up; row 8, column 7 does.
 */
export const cellPosition = (cell: number): CellPosition => {
  const { x, y } = xy(cell)
  return { row: y + 1, column: x + 1 }
}

/** The cells of one row, left to right — what a row of the grid is made of. */
export const rowOf = (cell: number): number[] => {
  const start = Math.floor(cell / WIDTH) * WIDTH
  return Array.from({ length: WIDTH }, (_, i) => start + i)
}

/**
 * A target switched on or off, as a new list in the reading order.
 *
 * The order is kept because the list is read as well as used: the filters are drawn from it,
 * and a target that jumped to the end of the row every time it was switched back on would make
 * the toolbar move under the hand that is using it.
 */
export const toggled = (
  shown: readonly TargetView[],
  target: TargetView,
): TargetView[] =>
  shown.includes(target)
    ? shown.filter((one) => one !== target)
    : targetOrder.filter((one) => one === target || shown.includes(one))
