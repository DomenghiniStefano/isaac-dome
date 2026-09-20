import { TargetView } from '@/lib/ipc/types'
import type { FloorSolutionView } from '@/lib/ipc/types'
import { WIDTH, xy } from './painting'

// What one cell of the grid is allowed to say (B64).
//
// Before this, a cell said one thing: a rank, or "painted", or "empty" — so fourteen room
// kinds came out as one grey square, and of the three targets only the Secret Room ever
// reached the map. It now says two: **what you drew** (a colour and a drawing, both in the
// component) and **what the rules make of it**, which is this file.

/**
 * A corner belongs to a target and to nothing else. That is the whole idea: the **position**
 * says which target, so the colour is a reinforcement rather than the only signal — it still
 * reads when two hues sit close, on a bad screen, or for someone who tells them apart poorly.
 *
 * Bottom-right is deliberately left empty. Its absence is visible, and a fourth target would
 * have somewhere to go without moving the other three.
 */
export const Corner = {
  TopLeft: 'topLeft',
  TopRight: 'topRight',
  BottomLeft: 'bottomLeft',
} as const
export type Corner = (typeof Corner)[keyof typeof Corner]

export const cornerOf: Record<TargetView, Corner> = {
  [TargetView.Secret]: Corner.TopLeft,
  [TargetView.SuperSecret]: Corner.TopRight,
  [TargetView.UltraSecret]: Corner.BottomLeft,
}

/** The order the corners are drawn in, and therefore the order the pips come back in. */
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

export interface Pip {
  readonly target: TargetView
  readonly corner: Corner
  readonly step: RankStep
  /** One-based: the number the cell prints, and the one the legend explains. */
  readonly rank: number
}

/**
 * What a cell shows of the rules, for the targets currently switched on.
 *
 * The order is the corners' own, never the order the solutions arrived in: a cell lit by two
 * targets has to look the same whichever answer landed first.
 */
export const pipsFor = (
  cell: number,
  solutions: readonly FloorSolutionView[],
  shown: readonly TargetView[],
): Pip[] => {
  const pips: Pip[] = []
  for (const target of targetOrder) {
    if (!shown.includes(target)) continue
    const solution = solutions.find((one) => one.target === target)
    const candidate = solution?.candidates.find((one) => one.cell === cell)
    if (candidate === undefined) continue
    pips.push({
      target,
      corner: cornerOf[target],
      step: rankStep(candidate.rank),
      rank: candidate.rank + 1,
    })
  }
  return pips
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
 * A target switched on or off, as a new list in the corners' own order.
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
