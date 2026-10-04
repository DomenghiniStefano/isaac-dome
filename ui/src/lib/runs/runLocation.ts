import type { RunView } from '@/lib/ipc/types'
import { singleQuery } from '@/lib/search/queryParam'
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import type { Tab } from '@/stores/tabModel'
import { runKey } from './runKey'

// A run's page is the Runs screen with the run named in the query, as a wiki page is the Wiki
// screen with a `page` one: a tab can hold it, back returns to the list, and no route is added.
export const runLocation = (run: RunView): TabLocation => ({
  name: RouteName.Runs,
  query: { run: runKey(run) },
})

/** The run a query names, or `null` when it names none — the list is shown. */
export const runKeyOf = (query: Record<string, unknown>): string | null => {
  const key = singleQuery(query.run)
  return key === null || key === '' ? null : key
}

/**
 * Whether the run's page was opened from the list, in this tab: the entry before it is the list.
 * Then the way back is back — to the list where its page box kept its place — and not a new entry
 * that opens the list at the top.
 */
export const listIsBehind = (tab: Tab | undefined): boolean => {
  const before = tab?.entries[tab.index - 1]?.location
  return (
    before !== undefined &&
    before.name === RouteName.Runs &&
    runKeyOf(before.query ?? {}) === null
  )
}
