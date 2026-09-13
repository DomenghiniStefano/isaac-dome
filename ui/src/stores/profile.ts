import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { LoadStatus } from './loadStatus'
import { tracked } from './tracked'
import { isIpcError } from '@/lib/ipc/errors'
import { saveSummary } from '@/lib/ipc/save'
import { selectProfile, setupState } from '@/lib/ipc/setup'
import type { IpcError, SaveSummary, SetupState } from '@/lib/ipc/types'

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

  // Two reads under one status: the profile is not loaded until its summary is too.
  const run = (read: () => Promise<SetupState>): Promise<void> =>
    tracked(status, error, async () => {
      setup.value = await read()
      await readSummary()
    })

  const load = (): Promise<void> => run(setupState)
  const choose = (id: string): Promise<void> => run(() => selectProfile(id))

  return { setup, summary, status, error, isActive, load, choose }
})
