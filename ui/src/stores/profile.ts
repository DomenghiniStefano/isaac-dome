import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { isIpcError } from '@/lib/ipc/errors'
import { saveSummary } from '@/lib/ipc/save'
import { selectProfile, setupState } from '@/lib/ipc/setup'
import type { IpcError, SaveSummary, SetupState } from '@/lib/ipc/types'

export const LoadStatus = {
  Idle: 'idle',
  Loading: 'loading',
  Ready: 'ready',
  Failed: 'failed',
} as const
export type LoadStatus = (typeof LoadStatus)[keyof typeof LoadStatus]

// The active profile is the window's, never a tab's (DESIGN-BRIEF.md §4.1, §4.2): one store,
// read by the indicator, the gate and the profile screen alike.
export const useProfileStore = defineStore(StoreId.Profile, () => {
  const setup = ref<SetupState | null>(null)
  const summary = ref<SaveSummary | null>(null)
  const status = ref<LoadStatus>(LoadStatus.Idle)
  const error = ref<IpcError | null>(null)

  const isActive = computed(() => setup.value?.active.kind === 'active')

  // No active profile means no summary, not an error.
  const readSummary = async (): Promise<void> => {
    if (!isActive.value) {
      summary.value = null
      return
    }
    try {
      summary.value = await saveSummary()
    } catch (e) {
      if (isIpcError(e) && e.kind === 'noActiveProfile') summary.value = null
      else throw e
    }
  }

  const run = async (read: () => Promise<SetupState>): Promise<void> => {
    status.value = LoadStatus.Loading
    error.value = null
    try {
      setup.value = await read()
      await readSummary()
      status.value = LoadStatus.Ready
    } catch (e) {
      error.value = isIpcError(e) ? e : null
      status.value = LoadStatus.Failed
    }
  }

  const load = (): Promise<void> => run(setupState)
  const choose = (id: string): Promise<void> => run(() => selectProfile(id))

  return { setup, summary, status, error, isActive, load, choose }
})
