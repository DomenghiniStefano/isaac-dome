import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { asIpcError } from '@/lib/ipc/errors'
import {
  wikiEntry,
  wikiIndex,
  wikiItemPools,
  wikiProgress,
} from '@/lib/ipc/wiki'
import type {
  Entry,
  IpcError,
  PageProgress,
  PoolMembershipView,
  Target,
  WikiIndex,
  WikiPageRef,
  WikiProgress,
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
  // A page whose read failed, with the error if it is ours (`null` if not). Its own map, not
  // the index's `status`: one page that would not load says nothing about the others.
  const failures = ref(new Map<string, IpcError | null>())
  const pending = new Set<string>()
  // The pools the installed game lists an item in, keyed like `entries` (design decision 4):
  // `null` is a real answer here too — no game, or a target with no pools concept — cached
  // the same way the index's own icon links are, so a page asks once per window.
  const pools = ref(new Map<string, PoolMembershipView[] | null>())
  const pendingPools = new Set<string>()
  // The save's state for every page that has one (design decision 6): `null` is a real
  // answer, "no save chosen", the same as `wikiProgress` itself answers — never the shape a
  // failed read leaves behind, which is why `loadProgress` below never writes it on a
  // rejection.
  const progress = ref<WikiProgress | null>(null)
  const progressStatus = ref<LoadStatus>(LoadStatus.Idle)
  const progressError = ref<IpcError | null>(null)

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

  // Built once per `progress` load, the way `byKey` is built once per index load: a `Map`
  // over the same page keys, so every screen's lookup is O(1) rather than a scan per row.
  const progressByKey = computed(
    () =>
      new Map<string, PageProgress>(
        (progress.value?.pages ?? []).flatMap((entry) => {
          const key = pageKey(entry.target)
          return key === null ? [] : [[key, entry.progress] as const]
        }),
      ),
  )

  // The active save's state for one page, by its own key — never by title, since a Tainted
  // form shares its base form's (CLAUDE.md, "resolve by id first"). `null` covers both "no
  // save chosen" and "this page has no state" (decision 6's table): a screen that only wants
  // to know whether to draw a bar treats the two the same way.
  const progressFor = (target: Target): PageProgress | null => {
    const key = pageKey(target)
    return key === null ? null : (progressByKey.value.get(key) ?? null)
  }

  // Read again whenever the wiki screen mounts or the chosen save changes
  // (`useOnActiveProfile` in the screens that call this). A read that fails leaves `progress`
  // exactly as it was: `wikiProgress()` throwing is never the same fact as it resolving to
  // `null`, and writing `null` for the first would cache a failure as "no save chosen".
  const loadProgress = (): Promise<void> =>
    tracked(progressStatus, progressError, async () => {
      progress.value = await wikiProgress()
    })

  // `undefined`: not read yet; `null`: read, and the dataset doesn't know it.
  const entry = (key: string): Entry | null | undefined =>
    entries.value.get(key)

  const pageFailed = (key: string): boolean => failures.value.has(key)

  const pageError = (key: string): IpcError | null =>
    failures.value.get(key) ?? null

  // Not `tracked`, and deliberately: the status and the error `tracked` writes belong to the
  // index. A page keeps its own failure, and asking again reads it again — `entries` never
  // took the key, so nothing stands in the way.
  const loadEntry = async (target: Target): Promise<void> => {
    const key = pageKey(target)
    if (key === null || entries.value.has(key) || pending.has(key)) return
    pending.add(key)
    failures.value.delete(key)
    try {
      entries.value.set(key, await wikiEntry(target))
    } catch (e) {
      failures.value.set(key, asIpcError(e))
    } finally {
      pending.delete(key)
    }
  }

  // `undefined`: not read yet; `null`: read, and there is nothing to show — no game, or a
  // target with no pools concept at all.
  const poolsFor = (key: string): PoolMembershipView[] | null | undefined =>
    pools.value.get(key)

  // A failed read leaves nothing cached, unlike `loadEntry`: there is no retry affordance for
  // a pools row, so asking again next time it is shown is simpler than a second failure map.
  const loadPools = async (target: Target): Promise<void> => {
    const key = pageKey(target)
    if (key === null || pools.value.has(key) || pendingPools.has(key)) return
    pendingPools.add(key)
    try {
      pools.value.set(key, await wikiItemPools(target))
    } catch {
      // no-op
    } finally {
      pendingPools.delete(key)
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
    pageFailed,
    pageError,
    loadEntry,
    poolsFor,
    loadPools,
    progress,
    progressStatus,
    progressError,
    progressFor,
    loadProgress,
  }
})
