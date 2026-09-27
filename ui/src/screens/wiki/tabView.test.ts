import { describe, expect, it } from 'vitest'
import { WikiFacet, emptyWikiListFilter } from '@/lib/wiki/listFacets'
import { wikiView } from './tabView'

describe("Wiki's reading", () => {
  it('is no filter, the default sort and no position when there is nothing to read', () => {
    expect(wikiView.empty()).toEqual({
      filter: emptyWikiListFilter(),
      sort: { key: 'name', direction: 'asc' },
      offset: null,
    })
  })

  // A category's list comes back as it was left: what it was filtered by, how it was
  // sorted, and how far down it was — through JSON, the shape a torn-off tab and a saved
  // session carry it in.
  it("reads back a list's filter, sort and position", () => {
    const filter = {
      ...emptyWikiListFilter(),
      query: 'brimstone',
      picks: {
        ...emptyWikiListFilter().picks,
        [WikiFacet.Quality]: ['4'],
      },
    }
    const written = {
      filter,
      sort: { key: 'quality', direction: 'desc' },
      offset: { top: 480, rows: 12 },
    }
    expect(wikiView.read(JSON.parse(JSON.stringify(written)))).toEqual(written)
  })

  it('reads anything that is not an object as nothing', () => {
    expect(wikiView.read(null)).toBeNull()
    expect(wikiView.read('brimstone')).toBeNull()
  })

  // Each field stands on its own: a position that no longer reads costs the position, never the
  // filter someone typed.
  it('reads a field that does not read as its default and keeps the others', () => {
    const filter = { ...emptyWikiListFilter(), query: 'sacred' }
    expect(wikiView.read({ filter, offset: 'far' })).toEqual({
      filter,
      sort: { key: 'name', direction: 'asc' },
      offset: null,
    })
    expect(wikiView.read({ sort: 7, offset: { top: 40, rows: 3 } })).toEqual({
      filter: emptyWikiListFilter(),
      sort: { key: 'name', direction: 'asc' },
      offset: { top: 40, rows: 3 },
    })
  })

  it('falls back to ascending on a direction it does not know', () => {
    expect(
      wikiView.read({ sort: { key: 'id', direction: 'sideways' } })?.sort,
    ).toEqual({ key: 'id', direction: 'asc' })
  })
})
