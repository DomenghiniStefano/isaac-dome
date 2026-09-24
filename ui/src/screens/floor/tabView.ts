import { readString } from '@/lib/tabs/tabView'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { CELLS, emptyCells } from '@/lib/floor/painting'
import type { PaintedCells } from '@/lib/floor/painting'
import { RoomKindView, TargetView } from '@/lib/ipc/types'

// The floor a tab is drawing, kept on the tab.
//
// **It was a store, and a store belongs to a window, not to a tab.** Tear a Floor tab off and
// the new window opened on an empty grid, because the drawing stayed in the old window's store;
// put it back and the old window showed what *its* store still held, which was the drawing from
// before. Two Floor tabs in one window were one drawing. What a tab carries across a tear-off is
// its history entries and nothing else, so the drawing lives there, like every other screen's
// reading — and like them it comes back when the session is restored.
//
// The target and the brush travel with it: they are how this drawing was being worked on, and a
// torn-off tab that forgot which of the three answers it was showing would be a tab that changed
// under the hand that moved it.
export interface FloorReading {
  cells: PaintedCells
  shown: TargetView
  brush: RoomKindView
}

const rooms: readonly string[] = Object.values(RoomKindView)
const targets: readonly string[] = Object.values(TargetView)

// A cell this version does not know is an empty cell: one room lost, never the floor around it.
const readCells = (value: unknown): PaintedCells | null => {
  if (!Array.isArray(value) || value.length !== CELLS) return null
  return value.map((cell) => readString(cell, rooms) as RoomKindView | null)
}

export const floorView: TabViewSpec<FloorReading> = {
  empty: () => ({
    cells: emptyCells(),
    shown: TargetView.Secret,
    brush: RoomKindView.Normal,
  }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const { cells, shown, brush } = value as {
      cells?: unknown
      shown?: unknown
      brush?: unknown
    }
    const read = readCells(cells)
    if (read === null) return null
    return {
      cells: read,
      shown:
        (readString(shown, targets) as TargetView | null) ?? TargetView.Secret,
      brush:
        (readString(brush, rooms) as RoomKindView | null) ??
        RoomKindView.Normal,
    }
  },
}
