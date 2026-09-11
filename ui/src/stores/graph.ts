import { defineStore } from 'pinia'
import { ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { isIpcError } from '@/lib/ipc/errors'
import {
  nextSteps as readNextSteps,
  unlock as readUnlock,
} from '@/lib/ipc/graph'
import type { IpcError, NextSteps, UnlockView } from '@/lib/ipc/types'
import { LoadStatus } from './profile'

// The active profile's unlock graph, read by Next steps and Unlock alike. Both answers belong
// to one profile: loading clears them first, so they're never shown under another.
export const useGraphStore = defineStore(StoreId.Graph, () => {
  const unlock = ref<UnlockView | null>(null)
  const steps = ref<NextSteps | null>(null)
  const status = ref<LoadStatus>(LoadStatus.Idle)
  const error = ref<IpcError | null>(null)

  const load = async (): Promise<void> => {
    unlock.value = null
    steps.value = null
    status.value = LoadStatus.Loading
    error.value = null
    try {
      const [view, next] = await Promise.all([readUnlock(), readNextSteps()])
      unlock.value = view
      steps.value = next
      status.value = LoadStatus.Ready
    } catch (e) {
      error.value = isIpcError(e) ? e : null
      status.value = LoadStatus.Failed
    }
  }

  return { unlock, steps, status, error, load }
})
