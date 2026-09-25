import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { effectScope } from 'vue'
import { emptyFilter } from '@/lib/facets/faceting'
import type { FacetFilter } from '@/lib/facets/faceting'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { RouteName } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import { useFacetedReading } from './useFacetedReading'

const Facet = { Pool: 'pool', Kind: 'kind' } as const
type Facet = (typeof Facet)[keyof typeof Facet]
const order: Facet[] = [Facet.Pool, Facet.Kind]

interface Reading {
  filter: FacetFilter<Facet>
  sort: string
}

// A screen that opens, and resets, on a pick — the Collection's shape — so a reset is seen to
// return to the screen's filter and not to an empty one.
const opening = (): FacetFilter<Facet> => ({
  ...emptyFilter(order),
  picks: { pool: ['treasure'], kind: [] },
})

const spec: TabViewSpec<Reading> = {
  empty: () => ({ filter: opening(), sort: 'name' }),
  read: () => null,
}

const inScope = () => {
  const faceted = effectScope().run(() => useFacetedReading(spec, opening))
  if (!faceted) throw new Error('the scope ran nothing')
  return faceted
}

beforeEach(() => {
  setActivePinia(createPinia())
  useTabsStore().seed(
    [{ entries: [{ location: { name: RouteName.Collection } }], index: 0 }],
    0,
  )
})

describe('a faceted list reading its tab', () => {
  it('opens on the reading the view starts with', () => {
    const { filter } = inScope()
    expect(filter.value).toEqual(opening())
  })

  it('moves one facet and leaves the others and the search alone', () => {
    const { filter, setQuery, setPicks } = inScope()
    setQuery('onion')
    setPicks(Facet.Kind, ['passive'])
    expect(filter.value).toEqual({
      query: 'onion',
      picks: { pool: ['treasure'], kind: ['passive'] },
    })
  })

  it('resets to the filter the screen opens on, keeping the rest of the reading', () => {
    const { reading, filter, setQuery, setPicks, update, reset } = inScope()
    setQuery('onion')
    setPicks(Facet.Pool, [])
    update({ sort: 'quality' })
    reset()
    expect(filter.value).toEqual(opening())
    expect(reading.value.sort).toBe('quality')
  })
})
