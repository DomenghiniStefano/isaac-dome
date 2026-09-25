import { computed, nextTick } from 'vue'
import type { Ref } from 'vue'
import { findWithCurrent, findWithQuery, openFind } from '@/lib/find/findState'
import type { FindState } from '@/lib/find/findState'
import { opensFind } from '@/lib/find/keyboard'
import { useShortcut } from './useShortcut'

// Find-in-page (B67) for a screen whose find bar lives in its tab's reading (#79): open or
// closed, its words and the match it is on come back after a tab switch, a back or a tear-off.
// The screen brings where the state is kept and how to bring a match into view; the moves on the
// state are `lib/find/findState.ts`'s.
export const useFind = (
  state: Ref<FindState | null>,
  scrollTo: (index: number) => void,
) => {
  const open = computed(() => state.value !== null)
  const query = computed({
    get: () => state.value?.query ?? '',
    set: (next: string) => {
      state.value = findWithQuery(state.value, next)
    },
  })
  const current = computed({
    get: () => state.value?.current ?? null,
    set: (next: string | null) => {
      state.value = findWithCurrent(state.value, next)
    },
  })

  useShortcut((event) => {
    if (!opensFind(event)) return false
    state.value = openFind(state.value)
    return true
  })

  const close = () => {
    state.value = null
  }

  // The row exists in the model before it exists as a node, so the scroll waits a tick for the
  // virtualizer to have been told the list it is scrolling in.
  const goTo = async (index: number) => {
    await nextTick()
    scrollTo(index)
  }

  return { open, query, current, close, goTo }
}
