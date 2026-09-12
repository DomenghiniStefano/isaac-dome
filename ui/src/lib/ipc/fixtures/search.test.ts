import { describe, expect, it } from 'vitest'
import { rankFixture } from './search'
import type { FixtureDoc } from './search'

const doc = (
  id: number,
  title: string,
  extra: Partial<FixtureDoc> = {},
): FixtureDoc => ({
  target: { kind: 'item', id },
  title,
  condition: null,
  sections: [],
  hasPage: true,
  iconUrl: null,
  ...extra,
})

describe('the fixture search', () => {
  it('orders by the same six tiers the backend uses', () => {
    const docs = [
      doc(1, 'Mother'),
      doc(2, 'The'),
      doc(3, 'The Bible'),
      doc(4, 'Of the'),
    ]
    const view = rankFixture(docs, 'the', 10)
    expect(view.hits.map((h) => h.title)).toEqual([
      'The',
      'The Bible',
      'Of the',
      'Mother',
    ])
    expect(view.total).toBe(4)
  })

  it('names the first field that holds every word, once per target', () => {
    const docs = [
      doc(1, 'Brimstone', {
        sections: [{ section: 'effects', text: 'Fires a blood laser' }],
      }),
      doc(2, 'Blood Bomb', { condition: 'Blow up 10 tinted rocks' }),
    ]
    expect(rankFixture(docs, 'brimstone laser', 10).hits).toHaveLength(0)
    expect(rankFixture(docs, 'blood laser', 10).hits[0]?.match).toEqual({
      kind: 'section',
      section: 'effects',
      before: 'Fires a ',
      matched: 'blood',
      after: ' laser',
    })
    expect(rankFixture(docs, 'tinted', 10).hits[0]?.match).toEqual({
      kind: 'condition',
      text: 'Blow up 10 tinted rocks',
    })
  })

  it('cuts to the limit and still says how many matched', () => {
    const docs = [1, 2, 3, 4, 5].map((id) => doc(id, `Bomb ${id}`))
    const view = rankFixture(docs, 'bomb', 2)
    expect(view.hits).toHaveLength(2)
    expect(view.total).toBe(5)
  })

  it('answers an empty query with nothing at all', () => {
    const view = rankFixture([doc(1, 'Bomb')], '   ', 10)
    expect(view.hits).toEqual([])
    expect(view.total).toBe(0)
    expect(view.diagnostics).toEqual([])
  })
})
