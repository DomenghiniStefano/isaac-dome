import { describe, expect, it } from 'vitest'
import { CELLS, emptyCells } from '@/lib/floor/painting'
import { RoomKindView, TargetView } from '@/lib/ipc/types'
import { floorView } from './tabView'

const drawn = () => {
  const cells = emptyCells()
  cells[84] = RoomKindView.Start
  cells[85] = RoomKindView.Boss
  return cells
}

describe("Floor's reading", () => {
  it('is an empty grid on the Secret Room with the Normal brush when there is nothing to read', () => {
    expect(floorView.empty()).toEqual({
      cells: emptyCells(),
      shown: TargetView.Secret,
      brush: RoomKindView.Normal,
    })
  })

  // The whole point: the drawing travels with the tab, so what was written is what comes back —
  // through JSON, because that is the shape a torn-off tab and a saved session carry it in.
  it('reads back the drawing, the target and the brush it was written with', () => {
    const written = {
      cells: drawn(),
      shown: TargetView.UltraSecret,
      brush: RoomKindView.Shop,
    }
    expect(floorView.read(JSON.parse(JSON.stringify(written)))).toEqual(written)
  })

  it('reads anything that is not a drawing as nothing', () => {
    expect(floorView.read(null)).toBeNull()
    expect(floorView.read('grid')).toBeNull()
    expect(floorView.read({ cells: 'grid' })).toBeNull()
  })

  // A grid of another size is another grid: laying it on this one would put every room in the
  // wrong place, which is worse than starting over.
  it('reads a grid that is not 13 by 13 as nothing', () => {
    expect(floorView.read({ cells: emptyCells().slice(1) })).toBeNull()
    expect(
      floorView.read({ cells: [...emptyCells(), null] })?.cells,
    ).toBeUndefined()
  })

  // One room this version does not know costs that cell, not the floor around it.
  it('reads a room it does not know as an empty cell and keeps the rest', () => {
    const cells: unknown[] = drawn()
    cells[86] = 'vault'
    const read = floorView.read({ cells })
    expect(read?.cells).toHaveLength(CELLS)
    expect(read?.cells[84]).toBe(RoomKindView.Start)
    expect(read?.cells[85]).toBe(RoomKindView.Boss)
    expect(read?.cells[86]).toBeNull()
  })

  it('reads an unknown target or brush as the ones the screen opens on', () => {
    const read = floorView.read({ cells: drawn(), shown: 'vault', brush: 7 })
    expect(read?.shown).toBe(TargetView.Secret)
    expect(read?.brush).toBe(RoomKindView.Normal)
  })
})
