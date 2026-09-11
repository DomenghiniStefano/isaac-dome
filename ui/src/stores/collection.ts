import { defineStore } from 'pinia'
import { ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { collection } from '@/lib/ipc/collection'
import { isIpcError } from '@/lib/ipc/errors'
import type { CollectionView, IpcError } from '@/lib/ipc/types'
import { LoadStatus } from './profile'

// The active profile's Collection. A view belongs to one profile: loading clears it first, so it
// is never shown under a profile it wasn't read from.
export const useCollectionStore = defineStore(StoreId.Collection, () => {
  const view = ref<CollectionView | null>(null)
  const status = ref<LoadStatus>(LoadStatus.Idle)
  const error = ref<IpcError | null>(null)

  const load = async (): Promise<void> => {
    view.value = null
    status.value = LoadStatus.Loading
    error.value = null
    try {
      view.value = await collection()
      status.value = LoadStatus.Ready
    } catch (e) {
      error.value = isIpcError(e) ? e : null
      status.value = LoadStatus.Failed
    }
  }

  return { view, status, error, load }
})
