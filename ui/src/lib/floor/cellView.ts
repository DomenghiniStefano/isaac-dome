import { TargetView } from '@/lib/ipc/types'
import type { FloorSolutionView } from '@/lib/ipc/types'
import { xy } from './painting'

// What one cell of the grid is allowed to say (B64).
//
// Before this, a cell said one thing: a rank, or "painted", or "empty" — so fourteen room
// kinds came out as one grey square, and of the three targets only the Secret Room ever
// reached the map. It now says two: **what you drew** (a colour and a drawing, both in the
// component) and **what the rules make of it**, which is this file.
//
// Two shapes were tried and thrown away in front of a real window, and both failed for the
// same reason — a 2rem square is not enough room to print a number in. First four pips in the
// corners, one per target, each with its rank inside it. Then the cell divided into a band per
// target, the rank still printed. **The screen shows one target at a time now**, and the rank
// is not written at all: it is how full the square is. Nothing has to be read.

/** The order the targets are read in: the switch's order, and the rules' order below it. */
export const TARGET_ORDER: readonly TargetView[] = [
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

/** What the rules make of one cell, for the one target being shown. */
export interface Candidate {
  readonly step: RankStep
  /** One-based: the place in the order, for the cell's own label. */
  readonly rank: number
}

/**
 * What a cell shows of the rules, or `null` when the rules say nothing about it.
 *
 * One target, not three. Showing them together was the first design and it is gone: three
 * answers laid over one 2rem square could only be drawn small enough to be unreadable,
 * whether as pips in the corners or as bands side by side.
 */
export const candidateFor = (
  cell: number,
  solutions: readonly FloorSolutionView[],
  target: TargetView,
): Candidate | null => {
  const solution = solutions.find((one) => one.target === target)
  const candidate = solution?.candidates.find((one) => one.cell === cell)
  if (candidate === undefined) return null
  return { step: rankStep(candidate.rank), rank: candidate.rank + 1 }
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
