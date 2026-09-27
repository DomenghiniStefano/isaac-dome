import { Dlc } from '@/lib/ipc/types'
import type { WikiPageRef } from '@/lib/ipc/types'
import type { WikiCategory } from '@/router/routeTable'
import { editionAdded } from './edition'
import { factColumns } from './factChips'
import { WikiSort } from './listFacets'
import type { WikiSortKey } from './listFacets'
import { pageId } from './wikiLabels'

export const SortDirection = { Asc: 'asc', Desc: 'desc' } as const
export type SortDirection = (typeof SortDirection)[keyof typeof SortDirection]

export interface WikiSortSpec {
  key: WikiSortKey
  direction: SortDirection
}

const releaseOrder: Dlc[] = Object.values(Dlc)

// Unrestricted is present since Rebirth — the same reading `editionAdded` gives the badge — so
// it ranks first, exactly where Rebirth itself would.
const editionRank = (dlc: Dlc[]): number => {
  const added = editionAdded(dlc)
  return added === null ? 0 : releaseOrder.indexOf(added)
}

const numericId = (id: string): string | number =>
  /^\d+$/.test(id) ? Number(id) : id

const valueFor = (
  page: WikiPageRef,
  category: WikiCategory,
  key: WikiSortKey,
): string | number | null => {
  switch (key) {
    case WikiSort.Name:
      return page.title
    case WikiSort.Id: {
      const id = pageId(page.target)
      return id === null ? null : numericId(id)
    }
    case WikiSort.Edition:
      return editionRank(page.dlc)
    default: {
      const column = factColumns(category).find((c) => c.key === key)
      return column ? column.value(page) : null
    }
  }
}

// `null` always sorts last, in either direction (Review Focus 5): a page the sorted fact does
// not name is not "less than everything", it is "nothing to compare".
const compare = (
  a: string | number | null,
  b: string | number | null,
  direction: SortDirection,
): number => {
  if (a === null || b === null) {
    if (a === null && b === null) return 0
    return a === null ? 1 : -1
  }
  const cmp =
    typeof a === 'number' && typeof b === 'number'
      ? a - b
      : String(a).localeCompare(String(b))
  return direction === SortDirection.Desc ? -cmp : cmp
}

// A category's pages, by one sort key: its name, its id, the edition it was added in, or one
// of its own fact columns (`listFacets.ts`'s `wikiSortOrder`). Every order ends on the name, so
// equal rows never swap between two renders (the same rule `sortNodes` and `sortItems` keep).
export const sortPages = (
  pages: WikiPageRef[],
  category: WikiCategory,
  sort: WikiSortSpec,
): WikiPageRef[] =>
  [...pages].sort((a, b) => {
    const cmp = compare(
      valueFor(a, category, sort.key),
      valueFor(b, category, sort.key),
      sort.direction,
    )
    return cmp !== 0 ? cmp : a.title.localeCompare(b.title)
  })
