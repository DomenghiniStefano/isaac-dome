import { defineStore } from 'pinia'
import { ref } from 'vue'
import { roomIcons } from '@/lib/ipc/floor'
import type { RoomKindView } from '@/lib/ipc/types'
import { StoreId } from '@/lib/constants/stores'

// What the Floor screen shares across a window: the game's pictures, and nothing else.
//
// **The drawing is not here any more**, and that is a fix (`screens/floor/tabView.ts` has the
// story): a store is one per window, a drawing is one per tab, and keeping it here lost it on
// every tear-off. The pictures are the other way round — they are the installed game's, the
// same for every tab — so they are asked once per window and kept.
export const useFloorStore = defineStore(StoreId.Floor, () => {
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

  return { icons, loadIcons }
})
