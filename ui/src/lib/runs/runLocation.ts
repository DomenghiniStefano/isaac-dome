import type { RunView } from '@/lib/ipc/types'
import { singleQuery } from '@/lib/search/queryParam'
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
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
