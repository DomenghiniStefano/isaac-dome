import { describe, expect, it } from 'vitest'
import { createFaceting, emptyFilter } from './faceting'
import { facetOptions } from './facetOptions'
import { optionsBySlot } from './filterBar'

interface Row {
  name: string
  pool: string
  kind: string
  tag: string
}

const Facet = { Pool: 'pool', Kind: 'kind', Tag: 'tag' } as const
type Facet = (typeof Facet)[keyof typeof Facet]

const rows: Row[] = [
  { name: 'sad onion', pool: 'treasure', kind: 'passive', tag: 'tears' },
  { name: 'the inner eye', pool: 'treasure', kind: 'passive', tag: 'tears' },
  { name: 'swallowed penny', pool: 'shop', kind: 'trinket', tag: 'coins' },
]

const faceting = createFaceting<Row, Facet>({
  order: [Facet.Pool, Facet.Kind, Facet.Tag],
  values: (row, facet) => [row[facet]],
  text: (row) => row.name,
  options: (all, facet) => [...new Set(all.map((row) => row[facet]))],
})

const label = (_facet: Facet, value: string) => `«${value}»`
const filter = {
  ...emptyFilter<Facet>([Facet.Pool, Facet.Kind, Facet.Tag]),
  picks: { pool: ['treasure'], kind: [], tag: [] },
}

// The bar draws each facet's dropdown from one table, computed once per change of the rows or
// the filter rather than once per dropdown per render.
describe('the options of every slot the bar draws', () => {
  const slots = [
    { facet: Facet.Kind, inView: true },
    { facet: Facet.Tag, inView: false },
  ]

  it('holds, for each slot, what facetOptions answers for its facet', () => {
    const table = optionsBySlot(faceting, rows, filter, slots, label)
    expect(table.get(Facet.Kind)).toEqual(
      facetOptions(faceting, rows, filter, Facet.Kind, label),
    )
    expect(table.get(Facet.Tag)).toEqual(
      facetOptions(faceting, rows, filter, Facet.Tag, label),
    )
  })

  // The state facet has its own row, and a facet the screen did not slot is not drawn: neither
  // is worth counting.
  it('counts only the facets that are slotted', () => {
    const table = optionsBySlot(faceting, rows, filter, slots, label)
    expect([...table.keys()]).toEqual([Facet.Kind, Facet.Tag])
  })
})
