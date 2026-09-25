import type { FacetFilter, Faceting } from './faceting'
import { facetOptions } from './facetOptions'
import type { FacetOption, FacetSlot } from './facetOptions'
import type { Message } from '@/i18n/message'
import type { FilterBarLabels } from './labels'

// Everything a screen's filter bar is made of that does not change while the screen is read:
// the faceting, where each facet sits, the state row, the words. One object per screen, beside
// its labels, instead of eleven props repeated at every call site.

// The filter that matters more than the others (DESIGN-BRIEF.md §6). The screen brings what its
// states are, what they are called and what colour each square carries; the numbers are the
// bar's, counted exactly like a dropdown's — see `stateRowCounts`.
export interface StateRow<Facet extends string> {
  facet: Facet
  order: string[]
  dot: Record<string, string>
  text: Record<string, Message>
}

// A list with nothing to choose between has no sort: the Run diary's order is decided by the
// archive (`runOrder.ts`), not by the reader, and a single fake option would be a control that
// changes nothing.
export interface SortChoice<Sort extends string> {
  order: Sort[]
  text: Record<Sort, Message>
}

export interface FilterBarDescriptor<
  Row,
  Facet extends string,
  Sort extends string,
> {
  faceting: Faceting<Row, Facet>
  // The facets, in order, each marked as in view at rest or behind the fold.
  facets: FacetSlot<Facet>[]
  state: StateRow<Facet>
  title: Record<Facet, Message>
  labels: FilterBarLabels
  sorts?: SortChoice<Sort>
}

// Each slotted facet's options, keyed by facet. One pass per change of the rows or the filter:
// the template asks for a facet's options once per dropdown, and a function call there counted
// every facet again on every render.
export const optionsBySlot = <Row, Facet extends string>(
  faceting: Faceting<Row, Facet>,
  rows: Row[],
  filter: FacetFilter<Facet>,
  slots: FacetSlot<Facet>[],
  valueLabel: (facet: Facet, value: string) => string,
): Map<Facet, FacetOption[]> =>
  new Map(
    slots.map(({ facet }) => [
      facet,
      facetOptions(faceting, rows, filter, facet, valueLabel),
    ]),
  )
