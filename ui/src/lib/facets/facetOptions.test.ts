import { describe, expect, it } from 'vitest'
import { createFaceting, emptyFilter } from './faceting'
import { facetOptions, foldStartsOpen, stateRowCounts } from './facetOptions'
import type { FacetSlot } from './facetOptions'

// Two facets over three rows, which is the smallest shape that has a value nobody can reach:
// `kind: 'trinket'` exists only on a row the pool filter is hiding.
interface Row {
  name: string
  pool: string
  kind: string
}

const Facet = { Pool: 'pool', Kind: 'kind' } as const
type Facet = (typeof Facet)[keyof typeof Facet]

const rows: Row[] = [
  { name: 'sad onion', pool: 'treasure', kind: 'passive' },
  { name: 'the inner eye', pool: 'treasure', kind: 'passive' },
  { name: 'swallowed penny', pool: 'shop', kind: 'trinket' },
]

const faceting = createFaceting<Row, Facet>({
  order: [Facet.Pool, Facet.Kind],
  values: (row, facet) => [row[facet]],
  text: (row) => row.name,
  options: (all, facet) => [...new Set(all.map((row) => row[facet]))],
})

const label = (_facet: Facet, value: string) => `«${value}»`
const empty = () => emptyFilter<Facet>([Facet.Pool, Facet.Kind])

describe('facetOptions', () => {
  it('gives every value its count and its words', () => {
    expect(facetOptions(faceting, rows, empty(), Facet.Kind, label)).toEqual([
      { value: 'passive', label: '«passive»', count: 2, picked: false },
      { value: 'trinket', label: '«trinket»', count: 1, picked: false },
    ])
  })

  // The rule that has never had a test: a value the other facets have emptied is not offered,
  // because picking it could only give nothing (`docs/BACKLOG.md` B29).
  it('drops a value nothing is left behind', () => {
    const filter = { ...empty(), picks: { pool: ['treasure'], kind: [] } }
    expect(
      facetOptions(faceting, rows, filter, Facet.Kind, label).map(
        (o) => o.value,
      ),
    ).toEqual(['passive'])
  })

  // …unless it is the one currently picked: dropping it would remove the only control that
  // undoes it, and the list would stay filtered by something invisible.
  it('keeps a picked value even when its count is zero', () => {
    const filter = { ...empty(), picks: { pool: ['shop'], kind: ['passive'] } }
    const options = facetOptions(faceting, rows, filter, Facet.Kind, label)
    expect(options).toContainEqual({
      value: 'passive',
      label: '«passive»',
      count: 0,
      picked: true,
    })
    // And the value that *does* have rows behind it is still offered: the rule drops what is
    // empty, never what the reader could still reach.
    expect(options.map((o) => o.value)).toEqual(['passive', 'trinket'])
  })
})

describe('stateRowCounts', () => {
  const order = ['passive', 'trinket', 'familiar']

  // Every value of the row always has a number, including one no row answers: a control whose
  // squares appear and disappear with the data moves under the reader's cursor.
  it('gives a value nobody has a zero rather than leaving it out', () => {
    expect(stateRowCounts(faceting, rows, empty(), Facet.Kind, order)).toEqual({
      passive: 2,
      trinket: 1,
      familiar: 0,
    })
  })

  // The state row sits in the same bar as the dropdowns and has to say the same kind of thing:
  // what is left once the rest of the filter is applied (spec 3.10 §5, corrected 2026-09-17).
  it('counts what the other facets leave', () => {
    const filter = { ...empty(), picks: { pool: ['shop'], kind: [] } }
    expect(stateRowCounts(faceting, rows, filter, Facet.Kind, order)).toEqual({
      passive: 0,
      trinket: 1,
      familiar: 0,
    })
  })

  // …and never counts itself: a row where picking one value zeroed the others could never be
  // used to pick a second one, which is the whole point of a multiple state control.
  it('ignores its own picks, so a second value can still be reached', () => {
    const filter = { ...empty(), picks: { pool: [], kind: ['passive'] } }
    expect(stateRowCounts(faceting, rows, filter, Facet.Kind, order)).toEqual({
      passive: 2,
      trinket: 1,
      familiar: 0,
    })
  })
})

describe('foldStartsOpen', () => {
  const slots: FacetSlot<Facet>[] = [
    { facet: Facet.Pool, inView: true },
    { facet: Facet.Kind, inView: false },
  ]

  it('stays closed when nothing behind it is picked', () => {
    expect(foldStartsOpen(slots, empty())).toBe(false)
  })

  it('opens when a folded facet holds a pick, so no filter is hidden', () => {
    const filter = { ...empty(), picks: { pool: [], kind: ['trinket'] } }
    expect(foldStartsOpen(slots, filter)).toBe(true)
  })

  // A pick on a control that is already on screen is not a reason to unfold: the reader can
  // see it.
  it('stays closed when only a facet in view is picked', () => {
    const filter = { ...empty(), picks: { pool: ['shop'], kind: [] } }
    expect(foldStartsOpen(slots, filter)).toBe(false)
  })
})
