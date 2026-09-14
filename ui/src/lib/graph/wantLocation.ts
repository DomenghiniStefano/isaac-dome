import { assertNever } from '@/lib/assertNever'
import type { Target } from '@/lib/ipc/types'
import { pageKey, parsePageKey } from '@/lib/wiki/pageKey'
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'

// Whether the graph can be asked for this thing at all.
//
// Until 2026-09-14 this question was not asked: a want travelled as a page key, and
// `pageKey` answered `null` for exactly the kinds nothing unlocks, so "has a wiki page" and
// "something in the graph can grant it" were the same set and one codec answered both. **B46
// ended that coincidence**: a transformation has a page now, and nothing unlocks one — you
// collect three items. `want_view` would answer `nothingUnlocks`, which is true and is a
// question the app should never have offered, so the two questions are two functions now.
//
// A switch and not a list: adding a `Target` variant has to break the build here, which is
// how the transformation would have been caught the day it gained a page.
const canBeWanted = (target: Target): boolean => {
  switch (target.kind) {
    case 'item':
    case 'trinket':
    case 'achievement':
    case 'challenge':
    case 'character':
    case 'entity':
      return true
    case 'transformation':
    case 'stage':
    case 'room':
    case 'concept':
      return false
    default:
      return assertNever(target)
  }
}

// A want still travels as the key a page does (`item:105`): one codec, already tested. What
// it no longer decides is *whether* there is a want to travel.
export const wantLocation = (target: Target): TabLocation | null => {
  const want = canBeWanted(target) ? pageKey(target) : null
  return want === null ? null : { name: RouteName.Goals, query: { want } }
}

// A key we never wrote is not an error: the screen draws the recommendations, as if no want
// had been named. A key we write for a page but never for a want — `transformation:1` — is
// the same case, and is read as no want rather than as a question with no answer.
export const wantOf = (query: TabLocation['query']): Target | null => {
  if (query?.want === undefined) return null
  const target = parsePageKey(query.want)
  return target !== null && canBeWanted(target) ? target : null
}

// The vocabulary, as a filter: a hit you can ask about is one the graph can grant.
export const wantable = <T extends { target: Target }>(hits: T[]): T[] =>
  hits.filter((hit) => canBeWanted(hit.target))
