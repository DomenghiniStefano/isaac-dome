import { describe, expect, it } from 'vitest'
import { CELLS, WIDTH, emptyCells, paintStroke, xy } from './painting'
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
