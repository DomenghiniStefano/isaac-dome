import { useDebounceFn } from '@vueuse/core'
import { ref } from 'vue'
import type { Ref } from 'vue'
import { Timing } from '@/lib/constants/timing'
import { search } from '@/lib/ipc/search'
import type { SearchView } from '@/lib/ipc/types'
import { latest } from '@/lib/search/latest'

export interface SearchState {
  view: Ref<SearchView | null>
  run: (query: string) => Promise<void>
}

// One search in flight per caller, with the call passed in so the guard can be tested without
// a backend. The palette and the Search screen each have their own state: they ask different
// questions at the same time, and one shared store would have each overwrite the other.
export const searchFrom = (
  call: (query: string, limit: number) => Promise<SearchView>,
  limit: number,
): SearchState => {
  const view = ref<SearchView | null>(null)
  const guard = latest()
  const run = async (query: string): Promise<void> => {
    const token = guard.next()
    if (query.trim() === '') {
      view.value = null
      return
    }
    try {
      const answer = await call(query, limit)
      // An answer to an older question than the one typed is dropped.
      if (guard.isCurrent(token)) view.value = answer
    } catch {
      // The last answer was to another question, and must not stand for this one. No error
      // is kept beside it (card #80, P10): nothing ever read one, and the command itself
      // never answers `Err` — what it cannot find travels as the view's diagnostics — so
      // only the transport can fail here.
      if (guard.isCurrent(token)) view.value = null
    }
  }
  return { view, run }
}

export const useSearch = (
  limit: number,
): SearchState & { ask: (query: string) => void } => {
  const state = searchFrom(search, limit)
  const debounced = useDebounceFn(state.run, Timing.SearchDebounce)
  return { ...state, ask: (query: string) => void debounced(query) }
}
