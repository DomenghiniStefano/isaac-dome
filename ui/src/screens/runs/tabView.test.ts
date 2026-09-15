import { describe, expect, it } from 'vitest'
import { emptyFilter } from '@/lib/facets/faceting'
import { RunFacet } from '@/lib/runs/runFacets'
import { runsView } from './tabView'

const order = Object.values(RunFacet)

describe("Runs' reading", () => {
  it('is the empty filter and nothing selected when there is nothing to read', () => {
    expect(runsView.empty()).toEqual({
      filter: emptyFilter<RunFacet>(order),
      selected: null,
      offset: null,
    })
  })

  // The selection is the run's key and never the run: a view-model is the archive's answer of
  // the moment, and storing one would restore a row that no longer describes anything.
  it('reads back the key of the selected run', () => {
    expect(
      runsView.read({
        filter: emptyFilter<RunFacet>(order),
        selected: 'session:09_14#2',
      })?.selected,
    ).toBe('session:09_14#2')
  })

  it('reads a selection that is not a string as nothing selected', () => {
    expect(
      runsView.read({ filter: emptyFilter<RunFacet>(order), selected: 7 })
        ?.selected,
    ).toBeNull()
  })
})
