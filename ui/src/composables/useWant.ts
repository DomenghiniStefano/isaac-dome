import { ref, watch } from 'vue'
import type { Ref } from 'vue'
import { want as readWant } from '@/lib/ipc/graph'
import type { IpcError, Target, WantView } from '@/lib/ipc/types'
import { LoadStatus } from '@/stores/loadStatus'
import { tracked } from '@/stores/tracked'

export interface WantState {
  view: Ref<WantView | null>
  status: Ref<LoadStatus>
  error: Ref<IpcError | null>
  load: () => Promise<void>
}

// A want is read *by target*, which is why this is a composable and not a view store:
// `defineViewStore`'s `load()` takes no argument. The triad is the same one every screen
// draws, so the loading and failed states are the ones the app already has.
//
// The call is passed in so the late-answer guard can be tested without a backend.
export const wantFrom = (
  call: (target: Target) => Promise<WantView>,
  target: Ref<Target | null>,
): WantState => {
  const view = ref<WantView | null>(null) as Ref<WantView | null>
  const status = ref<LoadStatus>(LoadStatus.Idle)
  const error = ref<IpcError | null>(null)

  const load = (): Promise<void> => {
    const asked = target.value
    view.value = null
    if (asked === null) {
      status.value = LoadStatus.Idle
      error.value = null
      return Promise.resolve()
    }
    return tracked(status, error, async () => {
      const answer = await call(asked)
      // The want may have changed while the command was in flight: a late answer to an older
      // question is worse than no answer at all.
      if (target.value === asked) view.value = answer
    })
  }

  watch(target, () => void load(), { immediate: true })
  return { view, status, error, load }
}

export const useWant = (target: Ref<Target | null>): WantState =>
  wantFrom(readWant, target)
