import type { WikiPageRef } from '@/lib/ipc/types'
import type { WikiCategory } from '@/router/routeTable'
import { categoryOf } from './category'

// A category's pages, by title, filtered on the title the way Unlock and the Collection
// filter on a name: trimmed, case-insensitive, anywhere in it.
export const filterPages = (
  pages: WikiPageRef[],
  category: WikiCategory,
  query: string,
): WikiPageRef[] => {
  const wanted = query.trim().toLowerCase()
  return pages
    .filter(
      (page) =>
        categoryOf(page.target) === category &&
        (wanted === '' || page.title.toLowerCase().includes(wanted)),
    )
    .sort((a, b) => a.title.localeCompare(b.title))
}
