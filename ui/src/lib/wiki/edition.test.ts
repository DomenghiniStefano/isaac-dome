import { describe, expect, it } from 'vitest'
import { Dlc } from '@/lib/ipc/types'
import { editionAdded, editionRemoved } from './edition'

// `Entry.dlc`'s own doc (`crates/wiki/src/model.rs`): empty is "no restriction stated", and
// a page that writes the restriction out as `n` lists all five — both mean "nothing to flag
// in the header", the same answer for two different shapes of "everywhere".

describe('editionAdded', () => {
  it('says nothing for an unrestricted entry (dlc empty)', () => {
    expect(editionAdded([])).toBeNull()
  })

  it('says nothing for a thing that has been there since Rebirth', () => {
    expect(editionAdded([Dlc.Rebirth])).toBeNull()
    // Explicit "no restriction" (`n`) lists all five: still Rebirth's own case.
    expect(
      editionAdded([
        Dlc.Rebirth,
        Dlc.Afterbirth,
        Dlc.AfterbirthPlus,
        Dlc.Repentance,
        Dlc.RepentancePlus,
      ]),
    ).toBeNull()
  })

  it('names the earliest edition when it is later than Rebirth', () => {
    expect(editionAdded([Dlc.AfterbirthPlus, Dlc.Repentance])).toBe(
      Dlc.AfterbirthPlus,
    )
  })

  it('reads the earliest whatever order the list arrives in', () => {
    expect(editionAdded([Dlc.RepentancePlus, Dlc.Repentance])).toBe(
      Dlc.Repentance,
    )
  })

  it('names a single later edition', () => {
    expect(editionAdded([Dlc.RepentancePlus])).toBe(Dlc.RepentancePlus)
  })
})

describe('editionRemoved', () => {
  it('says nothing for an unrestricted entry (dlc empty)', () => {
    expect(editionRemoved([])).toBeNull()
  })

  it('says nothing when the range reaches the newest edition', () => {
    expect(editionRemoved([Dlc.Rebirth, Dlc.RepentancePlus])).toBeNull()
    expect(editionRemoved([Dlc.RepentancePlus])).toBeNull()
  })

  it('names the edition right after the last one the entry existed in', () => {
    expect(editionRemoved([Dlc.Rebirth, Dlc.Afterbirth])).toBe(
      Dlc.AfterbirthPlus,
    )
  })

  it('names Rebirth+1 for a thing that only ever existed in Rebirth', () => {
    expect(editionRemoved([Dlc.Rebirth])).toBe(Dlc.Afterbirth)
  })

  it('reads the latest whatever order the list arrives in', () => {
    expect(editionRemoved([Dlc.Repentance, Dlc.Rebirth])).toBe(
      Dlc.RepentancePlus,
    )
  })
})
