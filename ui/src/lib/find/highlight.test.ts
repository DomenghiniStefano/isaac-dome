import { describe, expect, it } from 'vitest'
import { segments } from './highlight'

const joined = (text: string, query: string) =>
  segments(text, query)
    .map((piece) => piece.text)
    .join('')

describe('segments', () => {
  it('splits the text around the match', () => {
    expect(segments('Sacred Heart', 'heart')).toEqual([
      { text: 'Sacred ', match: false },
      { text: 'Heart', match: true },
    ])
  })

  it('keeps the text exactly as the row wrote it, matching without case', () => {
    // The match is case-insensitive, but what is drawn is the original: a row that
    // rendered "Heart" as "heart" would be the find bar rewriting the page.
    expect(segments('Sacred Heart', 'HEART')[1]).toEqual({
      text: 'Heart',
      match: true,
    })
  })

  it('handles a match that closes the text, with nothing after it', () => {
    expect(segments('Heart of Gold', 'gold')).toEqual([
      { text: 'Heart of ', match: false },
      { text: 'Gold', match: true },
    ])
  })

  it('marks two occurrences that touch as two', () => {
    expect(segments('abab', 'ab')).toEqual([
      { text: 'ab', match: true },
      { text: 'ab', match: true },
    ])
  })

  it('marks every occurrence, not only the first', () => {
    expect(segments('Brim Brim', 'brim')).toEqual([
      { text: 'Brim', match: true },
      { text: ' ', match: false },
      { text: 'Brim', match: true },
    ])
  })

  it('handles a match that opens the text', () => {
    expect(segments('Heart of Gold', 'heart')).toEqual([
      { text: 'Heart', match: true },
      { text: ' of Gold', match: false },
    ])
  })

  it('gives the whole text back when nothing matches', () => {
    expect(segments('Brimstone', 'zzz')).toEqual([
      { text: 'Brimstone', match: false },
    ])
  })

  it('gives the whole text back on an empty query, so a closed bar paints nothing', () => {
    expect(segments('Brimstone', '')).toEqual([
      { text: 'Brimstone', match: false },
    ])
    expect(segments('Brimstone', '   ')).toEqual([
      { text: 'Brimstone', match: false },
    ])
  })

  it('never loses or invents a character, whatever the query', () => {
    // The property that matters more than any single split: what is drawn is what the row
    // said. If this holds, the highlight cannot corrupt the page it is painting on.
    const text = 'The Brim of the Brimstone Hat'
    for (const query of ['brim', 'the', 'x', '', 'Brimstone Hat', 'T']) {
      expect(joined(text, query)).toBe(text)
    }
  })

  it('answers on an empty text without inventing a segment to draw', () => {
    expect(segments('', 'brim')).toEqual([])
  })
})
