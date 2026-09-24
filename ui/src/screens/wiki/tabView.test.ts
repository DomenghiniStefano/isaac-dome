import { describe, expect, it } from 'vitest'
import { wikiView } from './tabView'

describe("Wiki's reading", () => {
  it('is no filter and no position when there is nothing to read', () => {
    expect(wikiView.empty()).toEqual({ query: '', offset: null })
  })

  // A category's list comes back as it was left: what it was filtered by and how far down it
  // was — through JSON, the shape a torn-off tab and a saved session carry it in.
  it("reads back a list's filter and position", () => {
    const written = { query: 'brimstone', offset: { top: 480, rows: 12 } }
    expect(wikiView.read(JSON.parse(JSON.stringify(written)))).toEqual(written)
  })

  it('reads anything that is not an object as nothing', () => {
    expect(wikiView.read(null)).toBeNull()
    expect(wikiView.read('brimstone')).toBeNull()
  })

  // Each field stands on its own: a position that no longer reads costs the position, never the
  // filter someone typed.
  it('reads a field that does not read as its empty value and keeps the other', () => {
    expect(wikiView.read({ query: 'sacred', offset: 'far' })).toEqual({
      query: 'sacred',
      offset: null,
    })
    expect(wikiView.read({ query: 7, offset: { top: 40, rows: 3 } })).toEqual({
      query: '',
      offset: { top: 40, rows: 3 },
    })
  })
})
