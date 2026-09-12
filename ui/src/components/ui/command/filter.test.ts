import { describe, expect, it } from 'vitest'
import { scoreItems } from './filter'

const items = new Map([
  ['a', 'Brimstone'],
  ['b', 'Blood Bomb'],
])
const groups = new Map([
  ['g1', new Set(['a'])],
  ['g2', new Set(['b'])],
])
const contains = (text: string, query: string) =>
  text.toLowerCase().includes(query.toLowerCase())

describe('scoreItems', () => {
  it('scores nothing when the list is already filtered', () => {
    // The palette's rows come from the backend: scoring them again would hide rows whose
    // text doesn't repeat the query — a section fragment, a screen's name.
    const filtered = scoreItems(items, groups, 'zzz', false, contains)
    expect(filtered.count).toBe(2)
    expect(filtered.items.size).toBe(0)
    expect([...filtered.groups]).toEqual(['g1', 'g2'])
  })

  it('scores every item when it does filter', () => {
    const filtered = scoreItems(items, groups, 'blood', true, contains)
    expect(filtered.count).toBe(1)
    expect(filtered.items.get('a')).toBe(0)
    expect(filtered.items.get('b')).toBe(1)
    expect([...filtered.groups]).toEqual(['g2'])
  })

  it('shows everything on an empty search', () => {
    const filtered = scoreItems(items, groups, '', true, contains)
    expect(filtered.count).toBe(2)
    expect(filtered.items.size).toBe(0)
  })
})
