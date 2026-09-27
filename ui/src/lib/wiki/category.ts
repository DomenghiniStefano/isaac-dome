import { assertNever } from '@/lib/assertNever'
import type { Target } from '@/lib/ipc/types'
import { RouteName, WikiCategory } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import { pageKey } from './pageKey'

// Which sidebar category a page belongs to, from its identity alone: what keeps the
// category's entry lit while a page tab is active.
//
// An approximation for two kinds, by construction: `entity` covers both a boss and a common
// enemy (design decision 2) and `article` covers four landing tiles or none (decision 5),
// and neither distinction survives in `Target` alone — it is `entry.infobox`'s variant that
// says which, and only the wiki index still has the entry when it is built
// (`WikiPageRef.category`, `crates/ipc/src/wiki.rs`). `filterPages` (`listFilter.ts`) reads
// that precise, per-page value instead of this function for exactly that reason. Here, an
// entity reads as a boss (the pre-existing reading) and an article reads as `null`: neither
// answer gates whether the page opens (`pageLocation` below), only which sidebar entry a
// stray cross-link lights up.
export const categoryOf = (target: Target): WikiCategory | null => {
  switch (target.kind) {
    case 'item':
      return WikiCategory.Items
    case 'trinket':
      return WikiCategory.Trinkets
    case 'achievement':
      return WikiCategory.Achievements
    case 'entity':
      return WikiCategory.Bosses
    case 'challenge':
      return WikiCategory.Challenges
    case 'character':
      return WikiCategory.Characters
    case 'transformation':
      return WikiCategory.Transformations
    case 'article':
    case 'stage':
    case 'room':
    case 'concept':
      return null
    default:
      return assertNever(target)
  }
}

// A page as a tab location: the route, its page key, and its category when one can be told
// from the target alone. Category is optional in `TabLocation`'s own query on purpose: a
// page whose category `categoryOf` cannot tell (an article, decision 5) still has to open —
// it only loses the sidebar entry lighting up, never the navigation itself.
export const pageLocation = (target: Target): TabLocation | null => {
  const page = pageKey(target)
  if (page === null) return null
  const category = categoryOf(target)
  return {
    name: RouteName.Wiki,
    query: category === null ? { page } : { category, page },
  }
}

// The same for a reference that may have no page at all: no page, no location.
export const pageLocationOf = (
  page: Target | null | undefined,
): TabLocation | null => (page ? pageLocation(page) : null)
