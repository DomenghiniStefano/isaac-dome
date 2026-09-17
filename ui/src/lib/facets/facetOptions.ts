import type { FacetFilter, Faceting } from './faceting'

// What a filter control needs to draw one facet, and the one judgment in the whole bar: which
// values are worth offering. It lives here rather than in the component because it is the part
// worth checking — the frontend's reading of "if a return value is worth checking, it lives in
// a pure crate".

export interface FacetOption {
  value: string
  label: string
  count: number
  picked: boolean
}

// A facet and where the bar draws it: in view at rest, or behind the fold. The screen decides,
// because which filter matters is a fact about the screen and not about the control.
export interface FacetSlot<Facet extends string> {
  facet: Facet
  inView: boolean
}

// The count is over the rows every *other* facet and the search leave: it says what picking the
// value would give, which is not how many rows it gives now.
//
// A value with nothing behind it is not offered at all: it could not be picked, and reading it
// with a 0 beside it is noise. A value that *is* picked stays whatever its count — it is the
// only control that undoes itself, and dropping it would leave the list filtered by something
// the reader cannot see.
export const facetOptions = <Row, Facet extends string>(
  faceting: Faceting<Row, Facet>,
  rows: Row[],
  filter: FacetFilter<Facet>,
  facet: Facet,
  valueLabel: (facet: Facet, value: string) => string,
): FacetOption[] => {
  const counts = faceting.counts(rows, filter, facet)
  const picked = filter.picks[facet]
  return faceting
    .options(rows, facet)
    .map((value) => ({
      value,
      label: valueLabel(facet, value),
      count: counts.get(value) ?? 0,
      picked: picked.includes(value),
    }))
    .filter((option) => option.count > 0 || option.picked)
}

// The state row's numbers, one per value it draws — including a zero for a value no row
// answers, because a control whose squares come and go with the data moves under the reader's
// cursor.
//
// It is the same count as a dropdown's and for the same reason: the row sits in the same bar,
// and two kinds of number side by side read as a mistake. So it says what the *rest* of the
// filter leaves, and never counts its own picks — a row where picking one value zeroed the
// others could never be used to pick a second one. Until 2026-09-17 each screen counted every
// row instead, which made the row sum to more than the list under it.
export const stateRowCounts = <Row, Facet extends string>(
  faceting: Faceting<Row, Facet>,
  rows: Row[],
  filter: FacetFilter<Facet>,
  facet: Facet,
  order: string[],
): Record<string, number> => {
  const counts = faceting.counts(rows, filter, facet)
  return Object.fromEntries(
    order.map((value) => [value, counts.get(value) ?? 0]),
  )
}

// The fold opens by itself when something behind it is picked. Nothing is stored: a filter that
// is on has to be reachable, and that is a property of the current picks, not of what the
// reader did last time.
export const foldStartsOpen = <Facet extends string>(
  slots: FacetSlot<Facet>[],
  filter: FacetFilter<Facet>,
): boolean =>
  slots.some((slot) => !slot.inView && filter.picks[slot.facet].length > 0)
