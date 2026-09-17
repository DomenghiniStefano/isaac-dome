import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { LoadStatus } from './loadStatus'
import { tracked } from './tracked'
import { isIpcError } from '@/lib/ipc/errors'
import { saveSummary } from '@/lib/ipc/save'
import {
  chooseGameFolder,
  chooseSavesFolder,
  selectProfile,
  setupState,
} from '@/lib/ipc/setup'
import type { IpcError, SaveSummary, SetupState } from '@/lib/ipc/types'

// The active profile is the window's, never a tab's (DESIGN-BRIEF.md §4.1, §4.2): one store,
// read by the indicator, the gate and the profile screen alike.
export const useProfileStore = defineStore(StoreId.Profile, () => {
  const setup = ref<SetupState | null>(null)
  const summary = ref<SaveSummary | null>(null)
  const status = ref<LoadStatus>(LoadStatus.Idle)
  const error = ref<IpcError | null>(null)
  // The welcome, asked for over a profile that is already settled. Not a route and not a
  // tab: it is a state this window is in, and another window's choice takes it out of it —
  // `App.vue` clears it when `ProfileChanged` arrives, because the settled profile is the
  // app's and one answer settles every window.
  const picking = ref(false)

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
  const pick = (): void => {
    picking.value = true
  }
  const stopPicking = (): void => {
    picking.value = false
  }
  // A choice made is a picker closed: the welcome is a question, and it has been answered.
  const choose = async (id: string): Promise<void> => {
    await run(() => selectProfile(id))
    stopPicking()
  }

  // B14: the dialog opens in Rust and the folder never reaches this side. Cancelling answers
  // the state as it already was, so there is no case to tell apart here.
  const pickGameFolder = (): Promise<void> => run(chooseGameFolder)
  const pickSavesFolder = (): Promise<void> => run(chooseSavesFolder)

  return {
    setup,
    summary,
    status,
    error,
    isActive,
    picking,
    load,
    pick,
    stopPicking,
    choose,
    pickGameFolder,
    pickSavesFolder,
  }
})
