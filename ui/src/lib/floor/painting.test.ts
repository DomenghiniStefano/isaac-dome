import { describe, expect, it } from 'vitest'
import {
  CELLS,
  Direction,
  WIDTH,
  emptyCells,
  paintStroke,
  shift,
  xy,
} from './painting'
import type { PaintedCells } from './painting'
import { RoomKindView } from '@/lib/ipc/types'

describe('the painted grid', () => {
  it('is 13 by 13', () => {
    expect(WIDTH).toBe(13)
    expect(CELLS).toBe(169)
    expect(emptyCells()).toHaveLength(CELLS)
    expect(emptyCells().every((c) => c === null)).toBe(true)
  })

  it('places a cell at the row and column its index says', () => {
    expect(xy(84)).toEqual({ x: 6, y: 6 })
    expect(xy(0)).toEqual({ x: 0, y: 0 })
    expect(xy(168)).toEqual({ x: 12, y: 12 })
  })

  it('paints every cell a stroke crosses, once each', () => {
    const before = emptyCells()
    const after = paintStroke(before, [1, 2, 2, 3], RoomKindView.Normal)
    expect(after[1]).toBe(RoomKindView.Normal)
    expect(after[2]).toBe(RoomKindView.Normal)
    expect(after[3]).toBe(RoomKindView.Normal)
    expect(after[0]).toBeNull()
  })

  it('does not change the array it was given', () => {
    const before = emptyCells()
    paintStroke(before, [5], RoomKindView.Boss)
    expect(before[5]).toBeNull()
  })

  it('erases with a null brush instead of a second function', () => {
    const painted = paintStroke(emptyCells(), [7], RoomKindView.Shop)
    expect(paintStroke(painted, [7], null)[7]).toBeNull()
  })

  it('ignores a cell outside the grid rather than growing the array', () => {
    const after = paintStroke(
      emptyCells(),
      [-1, CELLS, 999],
      RoomKindView.Normal,
    )
    expect(after).toHaveLength(CELLS)
    expect(after.every((c) => c === null)).toBe(true)
  })
})

// Moving what you drew, one cell at a time (the arrows under the grid).
//
// **A move that would lose a room is not made.** There is no undo on this screen, so the
// alternative — letting what crosses the edge fall off — is a press too many deleting work
// with nothing to say so afterwards. `null` is the answer the arrows are disabled by, and it
// means one thing only: something is against that edge.
describe('shifting the drawing', () => {
  const at = (x: number, y: number): number => y * WIDTH + x

  const painted = (...cells: number[]): PaintedCells =>
    paintStroke(emptyCells(), cells, RoomKindView.Normal)

  it('moves every painted cell one step, and leaves what it left behind empty', () => {
    const before = painted(at(5, 5), at(6, 5))
    const after = shift(before, Direction.Right)
    expect(after).not.toBeNull()
    expect(after![at(6, 5)]).toBe(RoomKindView.Normal)
    expect(after![at(7, 5)]).toBe(RoomKindView.Normal)
    expect(after![at(5, 5)]).toBeNull()
  })

  it('carries every room its own kind, not one kind for the whole drawing', () => {
    const before = paintStroke(
      paintStroke(emptyCells(), [at(1, 1)], RoomKindView.Boss),
      [at(2, 1)],
      RoomKindView.Shop,
    )
    const after = shift(before, Direction.Down)
    expect(after![at(1, 2)]).toBe(RoomKindView.Boss)
    expect(after![at(2, 2)]).toBe(RoomKindView.Shop)
  })

  it('goes each of the four ways', () => {
    const before = painted(at(6, 6))
    expect(shift(before, Direction.Left)![at(5, 6)]).toBe(RoomKindView.Normal)
    expect(shift(before, Direction.Right)![at(7, 6)]).toBe(RoomKindView.Normal)
    expect(shift(before, Direction.Up)![at(6, 5)]).toBe(RoomKindView.Normal)
    expect(shift(before, Direction.Down)![at(6, 7)]).toBe(RoomKindView.Normal)
  })

  it('refuses the one direction that would push a room off the grid', () => {
    // A room against the left edge: every other direction is still open, which is what makes
    // a disabled arrow readable as "not that way" rather than as "not any more".
    const before = painted(at(0, 6))
    expect(shift(before, Direction.Left)).toBeNull()
    expect(shift(before, Direction.Right)).not.toBeNull()
    expect(shift(before, Direction.Up)).not.toBeNull()
    expect(shift(before, Direction.Down)).not.toBeNull()
  })

  it('refuses on any edge, not only the first one it looks at', () => {
    expect(shift(painted(at(6, 0)), Direction.Up)).toBeNull()
    expect(shift(painted(at(12, 6)), Direction.Right)).toBeNull()
    expect(shift(painted(at(6, 12)), Direction.Down)).toBeNull()
  })

  it('refuses because of a room on the edge even when most of the drawing is not', () => {
    // The whole drawing moves or none of it does: a move that kept the middle and dropped the
    // one room touching the wall would be the screen editing what you drew.
    const before = painted(at(0, 0), at(5, 5), at(6, 5), at(7, 5))
    expect(shift(before, Direction.Left)).toBeNull()
  })

  it('moves an untouched grid to another untouched grid rather than refusing', () => {
    // Nothing to lose, so nothing to refuse. A disabled arrow has to mean one thing only.
    const after = shift(emptyCells(), Direction.Left)
    expect(after).not.toBeNull()
    expect(after!.every((c) => c === null)).toBe(true)
  })

  it('answers a new grid and never the one it was given', () => {
    const before = painted(at(5, 5))
    const after = shift(before, Direction.Right)
    expect(after).not.toBe(before)
    expect(before[at(5, 5)]).toBe(RoomKindView.Normal)
  })
})
