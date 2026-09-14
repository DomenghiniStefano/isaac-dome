import { assertNever } from '@/lib/assertNever'
import type { Target } from '@/lib/ipc/types'
import { RouteName, WikiCategory } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import { pageKey } from './pageKey'

// Which sidebar category a page belongs to, from its identity alone: what keeps the
// category's entry lit while a page tab is active.
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
    case 'stage':
    case 'room':
    case 'concept':
    case 'transformation':
      return null
    default:
      return assertNever(target)
  }
}

// A page as a tab location: the route, its category, and the page key. Nothing else — a
// tab saves identity, never content (B6).
export const pageLocation = (target: Target): TabLocation | null => {
  const category = categoryOf(target)
  const page = pageKey(target)
  return category === null || page === null
    ? null
    : { name: RouteName.Wiki, query: { category, page } }
}
