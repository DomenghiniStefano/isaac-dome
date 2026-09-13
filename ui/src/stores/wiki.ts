import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { isIpcError } from '@/lib/ipc/errors'
import { wikiEntry, wikiIndex } from '@/lib/ipc/wiki'
import type {
  Entry,
  IpcError,
  Target,
  WikiIndex,
  WikiPageRef,
} from '@/lib/ipc/types'
import { pageKey } from '@/lib/wiki/pageKey'
import { LoadStatus } from './loadStatus'
import { tracked } from './tracked'

// The wiki, window-wide and profile-free (DESIGN-BRIEF.md §4.2): the index once, every page
// once. Nothing here is cleared on a profile change, because nothing here depends on one.
export const useWikiStore = defineStore(StoreId.Wiki, () => {
  const index = ref<WikiIndex | null>(null)
  const status = ref<LoadStatus>(LoadStatus.Idle)
  const error = ref<IpcError | null>(null)
  // A page read once stays read; `null` is an answer too (the dataset lacks it), so a
  // second look doesn't ask again.
  const entries = ref(new Map<string, Entry | null>())
  const pending = new Set<string>()

  const byKey = computed(
    () =>
      new Map<string, WikiPageRef>(
        (index.value?.pages ?? []).flatMap((page) => {
          const key = pageKey(page.target)
          return key === null ? [] : [[key, page] as const]
        }),
      ),
  )

  // The index is read once for the window's life: the dataset is compiled into the binary and
  // cannot change under us, so a second call while one is in flight — or after it landed — is
  // not a refresh, it is the same answer asked for twice.
  const loadIndex = (): Promise<void> => {
    if (
      status.value === LoadStatus.Ready ||
      status.value === LoadStatus.Loading
    )
      return Promise.resolve()
    return tracked(status, error, async () => {
      index.value = await wikiIndex()
    })
  }

  const titleOf = (key: string): string | null =>
    byKey.value.get(key)?.title ?? null

  const iconFor = (target: Target): string | null => {
    const key = pageKey(target)
    return key === null ? null : (byKey.value.get(key)?.iconUrl ?? null)
  }

  // Whether a reference can open a page: a target with no key, or one the index doesn't
  // list, reads as a concept rather than leading to a page that says "unknown".
  const hasPage = (target: Target): boolean => {
    const key = pageKey(target)
    return key !== null && byKey.value.has(key)
  }

  // `undefined`: not read yet; `null`: read, and the dataset doesn't know it.
  const entry = (key: string): Entry | null | undefined =>
    entries.value.get(key)

  // The one read here that is **not** `tracked`, and deliberately: a page that arrives must
  // not set the status to `Ready`, because the status belongs to the index. This reports a
  // failure without ever claiming a success, which is a shape `tracked` cannot express.
  //
  // That a page's failure lands on the index's `error` is inherited, not decided here.
  const loadEntry = async (target: Target): Promise<void> => {
    const key = pageKey(target)
    if (key === null || entries.value.has(key) || pending.has(key)) return
    pending.add(key)
    try {
      entries.value.set(key, await wikiEntry(target))
    } catch (e) {
      error.value = isIpcError(e) ? e : null
      status.value = LoadStatus.Failed
    } finally {
      pending.delete(key)
    }
  }

  return {
    index,
    status,
    error,
    loadIndex,
    titleOf,
    iconFor,
    hasPage,
    entry,
    loadEntry,
  }
})
