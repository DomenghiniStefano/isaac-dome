import type { WikiPageRef } from '@/lib/ipc/types'
import type { WikiCategory } from '@/router/routeTable'

// A category's pages, by title, filtered on the title the way Unlock and the Collection
// filter on a name: trimmed, case-insensitive, anywhere in it.
//
// Reads each page's own `category` — settled once, in Rust, from its entry's infobox — never
// `categoryOf(page.target)`: a `Target::Entity` alone cannot say boss from common enemy, nor
// a `Target::Article` say which of four landing tiles it belongs to (design decisions 2
// and 5). `page.category` and `WikiCategory` are the same values under two names (see
// `WikiPageCategory`'s own doc comment), so the comparison needs no translation.
export const filterPages = (
  pages: WikiPageRef[],
  category: WikiCategory,
  query: string,
): WikiPageRef[] => {
  const wanted = query.trim().toLowerCase()
  return pages
    .filter(
      (page) =>
        page.category === category &&
        (wanted === '' || page.title.toLowerCase().includes(wanted)),
    )
    .sort((a, b) => a.title.localeCompare(b.title))
}
