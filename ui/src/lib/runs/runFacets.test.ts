import { describe, expect, it } from 'vitest'
import type { RunView } from '@/lib/ipc/types'
import { emptyFilter } from '@/lib/facets/faceting'
import { RunFacet, outcomeCounts, runFaceting } from './runFacets'

const run = (fields: Partial<RunView>): RunView => ({
  source: { kind: 'session', name: '09_12_2026__13_34_26' },
  ordinal: 1,
  character: 'Cain',
  characterId: 2,
  seedWords: 'FYQ8 QQ8G',
  online: false,
  outcome: { kind: 'abandoned' },
  floors: 3,
  startingItems: [],
  collected: [],
  heldActive: null,
  achievements: [],
  ...fields,
})

const filter = (
  picks: Partial<Record<RunFacet, string[]>> = {},
  query = '',
) => ({
  ...emptyFilter<RunFacet>(Object.values(RunFacet)),
  query,
  picks: {
    ...emptyFilter<RunFacet>(Object.values(RunFacet)).picks,
    ...picks,
  },
})

describe('the run facets', () => {
  it('keeps an abandoned run as a value like any other', () => {
    const rows = [
      run({ outcome: { kind: 'abandoned' }, seedWords: 'left' }),
      run({ outcome: { kind: 'won', ending: 'Mother' }, seedWords: 'won' }),
    ]
    const picked = rows.filter((r) =>
      runFaceting.matches(r, filter({ [RunFacet.Outcome]: ['abandoned'] })),
    )
    expect(picked.map((r) => r.seedWords)).toEqual(['left'])
  })

  it('tells a run that was online from one that was not', () => {
    const rows = [
      run({ online: true, seedWords: 'co-op' }),
      run({ online: false, seedWords: 'alone' }),
    ]
    const picked = rows.filter((r) =>
      runFaceting.matches(r, filter({ [RunFacet.Online]: ['online'] })),
    )
    expect(picked.map((r) => r.seedWords)).toEqual(['co-op'])
  })

  // `character: null` is "no item line ever named it", which is not a character called
  // "unknown": offering it as a value would invent a row the archive does not have.
  it('offers no character for a run that never named one', () => {
    const rows = [run({ character: null }), run({ character: 'Cain' })]
    expect(runFaceting.options(rows, RunFacet.Character)).toEqual(['Cain'])
  })

  it('finds a run by its seed and by its character', () => {
    const rows = [
      run({ seedWords: 'FYQ8 QQ8G', character: 'Cain' }),
      run({ seedWords: 'AB12 CD34', character: 'Judas' }),
    ]
    expect(
      rows
        .filter((r) => runFaceting.matches(r, filter({}, 'qq8g')))
        .map((r) => r.character),
    ).toEqual(['Cain'])
    expect(
      rows
        .filter((r) => runFaceting.matches(r, filter({}, 'judas')))
        .map((r) => r.seedWords),
    ).toEqual(['AB12 CD34'])
  })

  // The engine's rule, pinned on this screen's rows: a facet's own picks do not narrow its
  // own counts, or picking one value would hide the others it could have been.
  it('counts the outcomes a pick on outcome could have had', () => {
    const rows = [
      run({ outcome: { kind: 'abandoned' } }),
      run({ outcome: { kind: 'won', ending: 'Mother' } }),
      run({ outcome: { kind: 'died', killer: 'Monstro' } }),
    ]
    const counts = runFaceting.counts(
      rows,
      filter({ [RunFacet.Outcome]: ['abandoned'] }),
      RunFacet.Outcome,
    )
    expect(counts.get('won')).toBe(1)
    expect(counts.get('died')).toBe(1)
  })

  it('names a session by its folder and the watched launch by itself', () => {
    const rows = [
      run({ source: { kind: 'live' } }),
      run({ source: { kind: 'session', name: '09_12_2026__13_34_26' } }),
    ]
    expect(runFaceting.options(rows, RunFacet.Source)).toEqual([
      'live',
      'session',
    ])
  })
})

describe('outcomeCounts', () => {
  it('counts every outcome, and says zero for one nothing reached', () => {
    const runs = [
      run({ outcome: { kind: 'won', ending: 'The Void' } }),
      run({ outcome: { kind: 'won', ending: 'Mother' } }),
      run({ outcome: { kind: 'abandoned' } }),
    ]
    expect(outcomeCounts(runs)).toEqual({
      won: 2,
      died: 0,
      abandoned: 1,
      open: 0,
    })
  })

  // An empty archive still draws four controls: a state row whose values come and go with the
  // data would move under the reader's cursor.
  it('answers for an archive with nothing in it', () => {
    expect(outcomeCounts([])).toEqual({
      won: 0,
      died: 0,
      abandoned: 0,
      open: 0,
    })
  })
})
