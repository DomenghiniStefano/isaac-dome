import type { RoomKindView } from '@/lib/ipc/types'

// The grid the game uses: 13 wide, 13 tall, a cell's index is y * WIDTH + x. The start room
// is 84, which is what the game prints on every floor.
export const WIDTH = 13
export const HEIGHT = 13
export const CELLS = WIDTH * HEIGHT
export const START = 84

export type PaintedCells = (RoomKindView | null)[]

export const emptyCells = (): PaintedCells =>
  Array<RoomKindView | null>(CELLS).fill(null)

export const xy = (cell: number): { x: number; y: number } => ({
  x: cell % WIDTH,
  y: Math.floor(cell / WIDTH),
})

/** The four ways the drawing can be nudged, under the grid. */
export const Direction = {
  Left: 'left',
  Up: 'up',
  Down: 'down',
  Right: 'right',
} as const
export type Direction = (typeof Direction)[keyof typeof Direction]

const step: Record<Direction, { x: number; y: number }> = {
  [Direction.Left]: { x: -1, y: 0 },
  [Direction.Up]: { x: 0, y: -1 },
  [Direction.Down]: { x: 0, y: 1 },
  [Direction.Right]: { x: 1, y: 0 },
}

/**
 * The whole drawing, one cell over. `null` when a room would be pushed off the grid.
 *
 * **A move that would lose a room is not made.** There is no undo on this screen, so the
 * alternative — letting what crosses the edge fall off — is one press too many deleting work
 * with nothing left to say so. The arrows are disabled by this `null`, which is what makes a
 * grey arrow readable: it means one thing only, that something is against that edge.
 *
 * All of it moves or none of it does. Keeping the middle and dropping the one room touching
 * the wall would be the screen editing what you drew.
 *
 * An untouched grid moves to another untouched grid rather than refusing: there is nothing to
 * lose, so there is nothing to refuse, and a disabled arrow has to keep meaning one thing.
 */
export const shift = (
  cells: PaintedCells,
  direction: Direction,
): PaintedCells | null => {
  const by = step[direction]
  const next = emptyCells()
  for (let cell = 0; cell < CELLS; cell += 1) {
    const kind = cells[cell]
    if (kind === null || kind === undefined) continue
    const { x, y } = xy(cell)
    const to = { x: x + by.x, y: y + by.y }
    if (to.x < 0 || to.x >= WIDTH || to.y < 0 || to.y >= HEIGHT) return null
    next[to.y * WIDTH + to.x] = kind
  }
  return next
}

// A stroke is the cells the pointer crossed, in order and with repeats; a null brush erases.
// It answers a new array, because the store's state is replaced rather than mutated.
export const paintStroke = (
  cells: PaintedCells,
  stroke: number[],
  brush: RoomKindView | null,
): PaintedCells => {
  const next = [...cells]
  for (const cell of stroke) {
    if (!Number.isInteger(cell) || cell < 0 || cell >= CELLS) continue
    next[cell] = brush
  }
  return next
}
