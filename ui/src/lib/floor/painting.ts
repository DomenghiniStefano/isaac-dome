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
