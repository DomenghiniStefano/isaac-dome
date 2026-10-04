import { describe, expect, it } from 'vitest'
import { emptyFilter } from '@/lib/facets/faceting'
import { RunItemView } from '@/lib/runs/itemViews'
import { RunFacet } from '@/lib/runs/runFacets'
import { runsView } from './tabView'

const order = Object.values(RunFacet)

describe("Runs' reading", () => {
  // A run is a page of its own now, so the list selects nothing; and the page box keeps where
  // the list was, so the reading keeps no offset either.
  it('is the empty filter and the items as logged when there is nothing to read', () => {
    expect(runsView.empty()).toEqual({
      filter: emptyFilter<RunFacet>(order),
      itemView: RunItemView.AsLogged,
    })
  })

  it('reads back the way the items were being read', () => {
    expect(
      runsView.read({
        filter: emptyFilter<RunFacet>(order),
        itemView: RunItemView.ByFloor,
      })?.itemView,
    ).toBe(RunItemView.ByFloor)
  })

  it('reads a view it does not know as the items as logged', () => {
    expect(
      runsView.read({
        filter: emptyFilter<RunFacet>(order),
        itemView: 'byMood',
      })?.itemView,
    ).toBe(RunItemView.AsLogged)
  })

  // A reading stored when the list selected a run and kept its offset still reads: the filter
  // is the user's, and the rest is left behind.
  it('reads a reading stored with a selection and an offset, and leaves both behind', () => {
    expect(
      runsView.read({
        filter: emptyFilter<RunFacet>(order),
        selected: 'session:09_14#2',
        offset: { top: 300, rows: 40 },
      }),
    ).toEqual({
      filter: emptyFilter<RunFacet>(order),
      itemView: RunItemView.AsLogged,
    })
  })
})
