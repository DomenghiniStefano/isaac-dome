import type { Faceting } from '@/lib/facets/faceting'
import type { WikiPageRef } from '@/lib/ipc/types'
import type { WikiCategory } from '@/router/routeTable'
import type { WikiFacet, WikiListFilter } from './listFacets'

// A category's pages, matched against the title, the edition, the save's state and the kind's
// own facets (design decision 4) — the faceting the screen built over its own `progressFor`
// (`listFacets.ts`'s `wikiFaceting`) does the matching; this narrows to the one category first.
//
// Reads each page's own `category` — settled once, in Rust, from its entry's infobox — never
// `categoryOf(page.target)`: a `Target::Entity` alone cannot say boss from common enemy, nor
// a `Target::Article` say which of four landing tiles it belongs to (design decisions 2
// and 5 of `2026-09-26-wiki-complete-design.md`).
export const filterPages = (
  pages: WikiPageRef[],
  category: WikiCategory,
  filter: WikiListFilter,
  faceting: Faceting<WikiPageRef, WikiFacet>,
): WikiPageRef[] =>
  pages.filter(
    (page) => page.category === category && faceting.matches(page, filter),
  )
