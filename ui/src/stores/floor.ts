import { defineStore } from 'pinia'
import { ref, shallowRef } from 'vue'
import { floorCandidates, roomIcons } from '@/lib/ipc/floor'
import { toggled } from '@/lib/floor/cellView'
import {
  emptyCells,
  paintStroke,
  type PaintedCells,
} from '@/lib/floor/painting'
import { TargetView, type FloorView, type RoomKindView } from '@/lib/ipc/types'
import { StoreId } from '@/lib/constants/stores'

// The painted floor is a scratchpad, not a document: it lives here and nowhere else, and it is
// gone when the app closes. Persisting it would outlive the floor it describes.
export const useFloorStore = defineStore(StoreId.Floor, () => {
  const cells = ref<PaintedCells>(emptyCells())
  const brush = ref<RoomKindView | null>(null)
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

  // All three on to begin with: the fixed corners are what lets them be read together, and a
  // screen that opens with two of them hidden would teach that they cannot be.
  const shown = ref<TargetView[]>([
    TargetView.Secret,
    TargetView.SuperSecret,
    TargetView.UltraSecret,
  ])

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

  const toggle = (target: TargetView): void => {
    shown.value = toggled(shown.value, target)
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
    toggle,
  }
})
