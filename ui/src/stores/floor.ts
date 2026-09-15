import { defineStore } from 'pinia'
import { ref, shallowRef } from 'vue'
import { floorCandidates } from '@/lib/ipc/floor'
import {
  emptyCells,
  paintStroke,
  type PaintedCells,
} from '@/lib/floor/painting'
import type { FloorView, RoomKindView } from '@/lib/ipc/types'
import { StoreId } from '@/lib/constants/stores'

// The painted floor is a scratchpad, not a document: it lives here and nowhere else, and it is
// gone when the app closes. Persisting it would outlive the floor it describes.
export const useFloorStore = defineStore(StoreId.Floor, () => {
  const cells = ref<PaintedCells>(emptyCells())
  const brush = ref<RoomKindView | null>(null)
  const view = shallowRef<FloorView | null>(null)
  const failed = ref(false)

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

  const clear = async (): Promise<void> => {
    cells.value = emptyCells()
    await solve()
  }

  return { cells, brush, view, failed, stroke, clear, solve }
})
