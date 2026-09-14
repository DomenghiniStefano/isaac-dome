import { countBy, sumBy } from 'lodash-es'

// One faceted list, for every screen that has one.
//
// A screen brings what is genuinely its own — which facets, how to read a row's values, what
// text the search reads, which options are offered — and gets back the part that was written
// twice word for word before this module existed: matching, the counts that leave their own
// facet out, and how many values are picked.

export interface FacetFilter<Facet extends string> {
  query: string
  picks: Record<Facet, string[]>
}

export interface FacetSpec<Row, Facet extends string> {
  order: Facet[]
  values: (row: Row, facet: Facet) => string[]
  text: (row: Row) => string
  options: (rows: Row[], facet: Facet) => string[]
}

export interface Faceting<Row, Facet extends string> {
  empty: () => FacetFilter<Facet>
  matches: (row: Row, filter: FacetFilter<Facet>) => boolean
  counts: (
    rows: Row[],
    filter: FacetFilter<Facet>,
    facet: Facet,
  ) => Map<string, number>
  options: (rows: Row[], facet: Facet) => string[]
  activeCount: (filter: FacetFilter<Facet>) => number
}

export const createFaceting = <Row, Facet extends string>(
  spec: FacetSpec<Row, Facet>,
): Faceting<Row, Facet> => {
  // A facet nobody picked a value in holds nothing back; a facet with picks takes a row that
  // holds any one of them, because one row sits in several pools and unlocks several kinds.
  const matchesFacet = (row: Row, facet: Facet, picked: string[]): boolean =>
    picked.length === 0 ||
    spec.values(row, facet).some((value) => picked.includes(value))

  // The engine lowercases what the spec hands it: a screen reading several fields joins them
  // and says nothing about case, and one that reads a single name should not have to either.
  const matchesQuery = (row: Row, query: string): boolean => {
    const wanted = query.trim().toLowerCase()
    return wanted === '' || spec.text(row).toLowerCase().includes(wanted)
  }

  const matchesFacets = (
    row: Row,
    filter: FacetFilter<Facet>,
    facets: Facet[],
  ): boolean =>
    matchesQuery(row, filter.query) &&
    facets.every((facet) => matchesFacet(row, facet, filter.picks[facet]))

  return {
    // Built fresh each call: one shared object would carry a screen's picks into the next
    // list opened on the same engine.
    // `order` is what the facets *are*: one left out of it is not part of the faceting, so a
    // key per entry is the whole record and the cast says only what the map already built.
    empty: () => ({
      query: '',
      picks: Object.fromEntries(
        spec.order.map((facet) => [facet, [] as string[]]),
      ) as Record<Facet, string[]>,
    }),

    matches: (row, filter) => matchesFacets(row, filter, spec.order),

    // A value's count leaves its own facet out: it says how many rows picking it would give,
    // which is not the same number as how many it gives now.
    counts: (rows, filter, facet) => {
      const others = spec.order.filter((f) => f !== facet)
      const values = rows
        .filter((row) => matchesFacets(row, filter, others))
        .flatMap((row) => spec.values(row, facet))
      return new Map(Object.entries(countBy(values)))
    },

    options: (rows, facet) => spec.options(rows, facet),

    // The search is shown on its own, so it is not one of the values picked.
    activeCount: (filter) =>
      sumBy(spec.order, (facet) => filter.picks[facet].length),
  }
}
