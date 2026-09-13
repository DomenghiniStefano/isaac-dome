import type { Target } from '@/lib/ipc/types'
import { pageKey, parsePageKey } from '@/lib/wiki/pageKey'
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'

// A want travels as the same key a page does (`item:105`): one codec, already tested, and it
// answers `null` for exactly the four kinds nothing unlocks — so a location exists only for
// a question the app can actually be asked.
export const wantLocation = (target: Target): TabLocation | null => {
  const want = pageKey(target)
  return want === null ? null : { name: RouteName.Goals, query: { want } }
}

// A key we never wrote is not an error: the screen draws the recommendations, as if no want
// had been named.
export const wantOf = (query: TabLocation['query']): Target | null =>
  query?.want === undefined ? null : parsePageKey(query.want)

// The vocabulary, as a filter: a hit you can ask about is one that has a page key, which is
// exactly the set of kinds something in the graph can grant. A second list of kinds here
// would be a second answer to the same question.
export const wantable = <T extends { target: Target }>(hits: T[]): T[] =>
  hits.filter((hit) => pageKey(hit.target) !== null)
