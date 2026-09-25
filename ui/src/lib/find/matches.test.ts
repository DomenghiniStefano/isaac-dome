import { describe, expect, it } from 'vitest'
import { FindDirection, findMatches, stepMatch } from './matches'

const rows = [
  { key: 'a', text: "Mom's Heart" },
  { key: 'b', text: 'Brimstone' },
  { key: 'c', text: 'The Brim of the Hat' },
  { key: 'd', text: 'Sacred Heart' },
]

describe('findMatches', () => {
  it('keeps the rows whose text contains the query, in the order the screen gave them', () => {
    const state = findMatches(rows, 'brim', null)
    expect(state.matches).toEqual(['b', 'c'])
  })

  it('ignores case, because case is not an option this bar offers', () => {
    expect(findMatches(rows, 'BRIMSTONE', null).matches).toEqual(['b'])
  })

  it('matches nothing on an empty query, because a bar that matches everything says nothing', () => {
    const state = findMatches(rows, '', null)
    expect(state.matches).toEqual([])
    expect(state.current).toBeNull()
  })

  it('matches nothing on a query that is only spaces', () => {
    expect(findMatches(rows, '   ', null).matches).toEqual([])
  })

  it('ignores space around the query, so a stray keystroke does not empty the bar', () => {
    expect(findMatches(rows, '  heart  ', null).matches).toEqual(['a', 'd'])
  })

  it('starts on the first match when nothing was current', () => {
    expect(findMatches(rows, 'heart', null).current).toBe('a')
  })

  it('keeps the current match when one more letter leaves it standing', () => {
    // The lesson B65 paid for: recomputing on every keystroke must not drag the
    // user back to the top of a list they had already walked into.
    const state = findMatches(rows, 'brim', 'c')
    expect(state.current).toBe('c')
  })

  it('falls back to the first match when the current one stops matching', () => {
    expect(findMatches(rows, 'brimstone', 'c').current).toBe('b')
  })

  it('counts as "3 di 17" does: the position is one-based and the total is the matches', () => {
    const state = findMatches(rows, 'heart', 'd')
    expect(state.position).toBe(2)
    expect(state.total).toBe(2)
  })

  it('has a position of zero when nothing matches, so no row can be called current', () => {
    const state = findMatches(rows, 'zzz', null)
    expect(state.position).toBe(0)
    expect(state.total).toBe(0)
    expect(state.current).toBeNull()
  })

  it('answers on a screen with no rows at all, instead of pretending it has some', () => {
    const state = findMatches([], 'heart', null)
    expect(state.total).toBe(0)
    expect(state.current).toBeNull()
  })
})

describe('stepMatch', () => {
  it('walks forward, which is what Enter asks for', () => {
    const state = findMatches(rows, 'heart', null)
    expect(stepMatch(state, FindDirection.Next).current).toBe('d')
  })

  it('wraps from the last match to the first, because the entry says the last wraps', () => {
    const state = findMatches(rows, 'heart', 'd')
    expect(stepMatch(state, FindDirection.Next).current).toBe('a')
  })

  it('walks backwards, which is what Shift+Enter asks for', () => {
    const state = findMatches(rows, 'heart', 'd')
    expect(stepMatch(state, FindDirection.Previous).current).toBe('a')
  })

  it('wraps from the first match back to the last', () => {
    const state = findMatches(rows, 'heart', 'a')
    expect(stepMatch(state, FindDirection.Previous).current).toBe('d')
  })

  it('moves the position with the current row', () => {
    const state = findMatches(rows, 'heart', 'a')
    expect(stepMatch(state, FindDirection.Next).position).toBe(2)
  })

  it('stays empty when there is nothing to step through', () => {
    const state = findMatches(rows, 'zzz', null)
    expect(stepMatch(state, FindDirection.Next).current).toBeNull()
    expect(stepMatch(state, FindDirection.Previous).position).toBe(0)
  })
})
