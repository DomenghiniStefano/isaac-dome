import type { FacetFilter } from './faceting'

// What an empty list is, and what there is to undo — for every list that has both a filter and
// something to say when it comes back with nothing.
//
// **The two halves are independent, and that is the whole point.** The sentence follows the
// total, the button follows the filter, because they answer different questions: a list with
// nothing in it is a fact about the data, and a filter is the only thing a reset can undo. Read
// off the filter alone, a list that was never populated is drawn as a search that failed; read
// off the total alone, a filter typed into an empty list has no way back.
//
// It lives beside the faceting because two of its three callers are faceted, but it takes a
// plain boolean rather than a filter: the wiki's category list has a query and no facets, and
// asking it to build a `FacetFilter` to be told its list is empty would be the wrong way round.

export interface EmptyList<Key extends string> {
  text: Key
  reset: boolean
}

export const emptyList = <Key extends string>(
  total: number,
  filtering: boolean,
  text: { empty: Key; noResults: Key },
): EmptyList<Key> => ({
  text: total === 0 ? text.empty : text.noResults,
  reset: filtering,
})

// A query of spaces is nothing typed, the way `matchesQuery` reads it.
export const queryTyped = (query: string): boolean => query.trim() !== ''

// Whether a faceted filter is holding anything back. It counts the picks a screen **opens**
// with — the Collection starts on `state: [available, locked]` — because a reset undoes those
// too, and a button that undoes something has to be offered.
export const isFiltering = <Facet extends string>(
  filter: FacetFilter<Facet>,
): boolean =>
  queryTyped(filter.query) ||
  Object.values<string[]>(filter.picks).some((picked) => picked.length > 0)
