import type { Message, Translate } from '@/i18n/message'
import { unlockKindLabel } from '@/lib/graph/unlockKindText'
import { assertNever } from '@/lib/assertNever'
import { oneOf } from '@/lib/oneOf'
import { originLabel } from '@/lib/facets/labels'
import { StateTone, stateDots } from '@/lib/facets/stateTone'
import type { FilterBarLabels } from '@/lib/facets/labels'
import type { FacetSlot } from '@/lib/facets/facetOptions'
import type { Faceting } from '@/lib/facets/faceting'
import type { FilterBarDescriptor } from '@/lib/facets/filterBar'
import type { CollectionItem } from '@/lib/ipc/types'
import {
  CollectionFacet,
  CollectionSort,
  NoPool,
  QualityValue,
} from '@/lib/collection/collectionFacets'
import { ItemState, itemStateOrder } from '@/lib/collection/itemState'

const collectionFacetTitle: Record<CollectionFacet, Message> = {
  [CollectionFacet.State]: 'collection.facet.state',
  [CollectionFacet.Quality]: 'collection.facet.quality',
  [CollectionFacet.Pool]: 'collection.facet.pool',
  [CollectionFacet.Kind]: 'collection.facet.kind',
  [CollectionFacet.Origin]: 'collection.facet.origin',
}

export const itemStateText: Record<ItemState, Message> = {
  [ItemState.InCollection]: 'collection.state.inCollection',
  [ItemState.Available]: 'collection.state.available',
  [ItemState.Locked]: 'collection.state.locked',
  [ItemState.Unknown]: 'collection.state.unknown',
}

// A facet value in words. Quality numbers and pool names are shown as they are: a number is a
// number, a pool is the game's own word.
export const collectionFacetValueLabel = (
  t: Translate,
  facet: CollectionFacet,
  value: string,
): string => {
  switch (facet) {
    case CollectionFacet.State: {
      const state = oneOf(ItemState, value)
      return state ? t(itemStateText[state]) : value
    }
    case CollectionFacet.Quality:
      return value === QualityValue.Unrated
        ? t('collection.qualityUnrated')
        : value
    case CollectionFacet.Pool:
      return value === NoPool ? t('collection.poolNone') : value
    case CollectionFacet.Kind:
      return unlockKindLabel(t, value)
    case CollectionFacet.Origin:
      return originLabel(t, value)
    default:
      return assertNever(facet)
  }
}

// Unreadable wears the unknown hatch, as its badge does.
const itemStateDot = stateDots<ItemState>({
  [ItemState.InCollection]: StateTone.Done,
  [ItemState.Available]: StateTone.Now,
  [ItemState.Locked]: StateTone.Blocked,
  [ItemState.Unknown]: StateTone.Unknown,
})

// Which filters are on screen at rest and which are behind the fold (spec 3.10 §3). The state
// is not one of these: it has its own row, and reaches the bar as its `state` prop.
const collectionSlots: FacetSlot<CollectionFacet>[] = [
  { facet: CollectionFacet.Quality, inView: true },
  { facet: CollectionFacet.Pool, inView: false },
  { facet: CollectionFacet.Kind, inView: false },
  { facet: CollectionFacet.Origin, inView: false },
]

const sortOrder: CollectionSort[] = [
  CollectionSort.Quality,
  CollectionSort.Id,
  CollectionSort.Name,
]

const sortText: Record<CollectionSort, Message> = {
  [CollectionSort.Quality]: 'collection.sort.quality',
  [CollectionSort.Id]: 'collection.sort.id',
  [CollectionSort.Name]: 'collection.sort.name',
}

const barLabels: FilterBarLabels = {
  rows: 'collection.items',
  search: 'collection.search',
  sortBy: 'collection.sortBy',
}

// The Collection's filter bar. Built from the faceting rather than holding it: the pools arrive
// with the view, so the faceting does too.
export const collectionBar = (
  faceting: Faceting<CollectionItem, CollectionFacet>,
): FilterBarDescriptor<CollectionItem, CollectionFacet, CollectionSort> => ({
  faceting,
  facets: collectionSlots,
  state: {
    facet: CollectionFacet.State,
    order: itemStateOrder,
    dot: itemStateDot,
    text: itemStateText,
  },
  title: collectionFacetTitle,
  labels: barLabels,
  sorts: { order: sortOrder, text: sortText },
})
