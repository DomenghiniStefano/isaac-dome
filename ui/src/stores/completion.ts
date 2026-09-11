import { defineStore } from 'pinia'
import { ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { isIpcError } from '@/lib/ipc/errors'
import { completion } from '@/lib/ipc/save'
import type { IpcError, MarksMatrix } from '@/lib/ipc/types'
import { LoadStatus } from './profile'

// The active profile's completion matrix. A matrix belongs to one profile: loading clears it
// first, so it is never shown under a profile it wasn't read from.
export const useCompletionStore = defineStore(StoreId.Completion, () => {
  const matrix = ref<MarksMatrix | null>(null)
  const status = ref<LoadStatus>(LoadStatus.Idle)
  const error = ref<IpcError | null>(null)

  const load = async (): Promise<void> => {
    matrix.value = null
    status.value = LoadStatus.Loading
    error.value = null
    try {
      matrix.value = await completion()
      status.value = LoadStatus.Ready
    } catch (e) {
      error.value = isIpcError(e) ? e : null
      status.value = LoadStatus.Failed
    }
  }

  return { matrix, status, error, load }
})
