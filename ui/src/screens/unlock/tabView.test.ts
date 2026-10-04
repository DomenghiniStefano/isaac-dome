import { describe, expect, it } from 'vitest'
import { UnlockSort, unlockFaceting } from '@/lib/graph/unlockFacets'
import { unlockView } from './tabView'

describe("Unlock's reading", () => {
  it('is the empty faceting and the default sort when there is nothing to read', () => {
    expect(unlockView.empty()).toEqual({
      filter: unlockFaceting.empty(),
      sort: UnlockSort.FanOut,
    })
  })

  it('reads back a filter and a sort it wrote', () => {
    const reading = {
      filter: { ...unlockFaceting.empty(), query: 'brim' },
      sort: UnlockSort.Name,
    }
    expect(unlockView.read(JSON.parse(JSON.stringify(reading)))).toEqual(
      reading,
    )
  })

  // A sort that was legal when it was written and is not a sort any more falls back to the
  // default rather than reaching `sortNodes` as a value it has no branch for.
  it('falls back to the default sort when the stored one is gone', () => {
    expect(
      unlockView.read({ filter: unlockFaceting.empty(), sort: 'byVibes' })
        ?.sort,
    ).toBe(UnlockSort.FanOut)
  })

  // The screen scrolls as a page, and the page box keeps its own position. A reading stored when
  // the list kept one still reads — its filter and sort are the user's — and the offset is left.
  it('reads a reading stored with a list offset, and leaves the offset behind', () => {
    expect(
      unlockView.read({
        filter: unlockFaceting.empty(),
        sort: UnlockSort.Name,
        offset: { top: 900, rows: 641 },
      }),
    ).toEqual({ filter: unlockFaceting.empty(), sort: UnlockSort.Name })
  })

  it('refuses a record that is not a reading', () => {
    expect(unlockView.read({ sort: UnlockSort.Name })).toBeNull()
    expect(unlockView.read(null)).toBeNull()
  })
})
