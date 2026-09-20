import { defineStore } from 'pinia'
import { ref, shallowRef } from 'vue'
import { floorCandidates, roomIcons } from '@/lib/ipc/floor'
import {
  emptyCells,
  paintStroke,
  type PaintedCells,
} from '@/lib/floor/painting'
import { RoomKindView, TargetView, type FloorView } from '@/lib/ipc/types'
import { StoreId } from '@/lib/constants/stores'

// The painted floor is a scratchpad, not a document: it lives here and nowhere else, and it is
// gone when the app closes. Persisting it would outlive the floor it describes.
export const useFloorStore = defineStore(StoreId.Floor, () => {
  const cells = ref<PaintedCells>(emptyCells())
  // The Normal Room to begin with, and never nothing: the palette has no eraser on it, so a
  // brush that paints nothing would be a state the screen cannot show and cannot leave.
  // Rubbing out is the right button on the grid.
  const brush = ref<RoomKindView>(RoomKindView.Normal)
  const view = shallowRef<FloorView | null>(null)
  const failed = ref(false)

  // The game's own picture per room kind, asked once. Empty until it answers and empty for
  // good on a machine without the game: the grid draws its own symbols either way, so there
  // is nothing here to wait for and nothing to report when it stays empty.
  const icons = ref<Map<RoomKindView, string>>(new Map())

  const loadIcons = async (): Promise<void> => {
    try {
      const rows = await roomIcons()
      icons.value = new Map(
        rows
          .filter((row) => row.iconUrl !== null)
          .map((row) => [row.kind, row.iconUrl as string]),
      )
    } catch {
      icons.value = new Map()
    }
  }

  // One target at a time, and never none: three answers laid over one 2rem cell could only be
  // drawn too small to read, and a screen showing no answer at all would hide nothing of the
  // drawing — a candidate only ever sits on a cell nobody painted. It opens on the Secret
  // Room, the one a player looks for on every floor.
  const shown = ref<TargetView>(TargetView.Secret)

  const solve = async (): Promise<void> => {
    try {
      view.value = await floorCandidates(cells.value)
      failed.value = false
    } catch {
      view.value = null
      failed.value = true
    }
  }

  const stroke = async (path: number[]): Promise<void> => {
    cells.value = paintStroke(cells.value, path, brush.value)
    await solve()
  }

  // Rubbing out is painting with no brush: one path through the grid, not a second one that
  // could disagree with it.
  const erase = async (cell: number): Promise<void> => {
    cells.value = paintStroke(cells.value, [cell], null)
    await solve()
  }

  const clear = async (): Promise<void> => {
    cells.value = emptyCells()
    await solve()
  }

  const show = (target: TargetView): void => {
    shown.value = target
  }

  return {
    cells,
    brush,
    view,
    failed,
    shown,
    icons,
    loadIcons,
    stroke,
    erase,
    clear,
    solve,
    show,
  }
})
