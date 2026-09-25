import { computed } from 'vue'
import type { FacetFilter } from '@/lib/facets/faceting'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { useTabView } from './useTabView'

// How a faceted list reads its tab's reading: the filter, and the moves its filter bar makes on
// it — the pick of one facet, the query, a reset.
//
// The filter belongs to the tab, not to the screen: leaving and coming back — through a
// tear-off, a restart, or the back button — finds it where it was left (B39). `update` is how
// the rest of the reading moves (a sort, a scroll offset, a selection), so it goes the same way.
//
// `emptyFilter` is what a reset returns to, and it is the screen's: the Collection opens, and
// resets, on what has not been found yet.
export const useFacetedReading = <
  Facet extends string,
  Reading extends { filter: FacetFilter<Facet> },
>(
  view: TabViewSpec<Reading>,
  emptyFilter: () => FacetFilter<Facet>,
) => {
  const { reading, update } = useTabView(view)
  const filter = computed({
    get: (): FacetFilter<Facet> => reading.value.filter,
    set: (value: FacetFilter<Facet>) =>
      update({ filter: value } as Partial<Reading>),
  })

  const setPicks = (facet: Facet, picked: string[]) => {
    filter.value = {
      ...filter.value,
      picks: { ...filter.value.picks, [facet]: picked },
    }
  }
  const setQuery = (query: string) => {
    filter.value = { ...filter.value, query }
  }
  const reset = () => {
    filter.value = emptyFilter()
  }

  return { reading, update, filter, setPicks, setQuery, reset }
}
