import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { collection } from '@/lib/ipc/collection'
import { tracked } from './tracked'
import { graphViews } from '@/lib/ipc/graph'
import { completion } from '@/lib/ipc/save'
import { live, runs } from '@/lib/ipc/runs'
import { LoadStatus } from './loadStatus'
import type {
  IpcError,
  MarksMatrix,
  GraphViews,
  LiveView,
  RunsView,
} from '@/lib/ipc/types'
import type { CollectionView } from '@/lib/ipc/types'

export interface ViewStore<T> {
  view: Ref<T | null>
  status: Ref<LoadStatus>
  error: Ref<IpcError | null>
  load: () => Promise<void>
}

// Three stores wrote the same `view` / `status` / `error` triad and the same `try` / `catch`;
// only the call in the middle changed. This is the middle, taken as an argument.
//
// The view is cleared **before** the read, not after it: a view belongs to one profile, and
// showing the old one for the length of a command would show it under a profile it was never
// read from. The status and the error are written by `tracked`, which is the half the stores
// with a richer shape use too.
export const defineViewStore = <T>(id: StoreId, read: () => Promise<T>) =>
  defineStore(id, (): ViewStore<T> => {
    const view = ref<T | null>(null) as Ref<T | null>
    const status = ref<LoadStatus>(LoadStatus.Idle)
    const error = ref<IpcError | null>(null)

    const load = (): Promise<void> => {
      view.value = null
      return tracked(status, error, async () => {
        view.value = await read()
      })
    }

    return { view, status, error, load }
  })

// The active profile's Collection.
export const useCollectionStore = defineViewStore<CollectionView>(
  StoreId.Collection,
  collection,
)

// The active profile's completion matrix.
export const useCompletionStore = defineViewStore<MarksMatrix>(
  StoreId.Completion,
  completion,
)

// The run archive. It is not a view of the profile: it fills itself in the background and
// says so with `runs-changed`, so the screen listens rather than polls.
export const useRunsStore = defineViewStore<RunsView>(StoreId.Runs, runs)

// What the run being watched would open. Its own store: the two screens are two questions,
// and Live'''s answer changes while Run'''s does not.
export const useLiveStore = defineViewStore<LiveView>(StoreId.Live, live)

// The active profile's unlock graph, read by Next steps and Unlock alike. One read, and
// since N8 one *command*: the promise this comment makes — that both answers belong to the
// same profile — is one a pair of commands could not keep.
export const useGraphStore = defineViewStore<GraphViews>(
  StoreId.Graph,
  graphViews,
)
