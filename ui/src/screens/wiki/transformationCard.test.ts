import { describe, expect, it } from 'vitest'
import type { Entry, Infobox } from '@/lib/ipc/types'
import { hasCard, hasRows } from './transformationCard'

type Transformation = Extract<Infobox, { kind: 'transformation' }>

const infobox = (
  fields: Partial<Omit<Transformation, 'kind'>>,
): Transformation => ({
  kind: 'transformation',
  requires: null,
  contributors: [],
  target: [],
  ...fields,
})

describe('hasRows', () => {
  it('draws the card when the page says how many contributors are needed', () => {
    expect(hasRows(infobox({ requires: 3 }))).toBe(true)
  })

  it('draws the card when the page lists contributors', () => {
    expect(hasRows(infobox({ contributors: [{ kind: 'item', id: 1 }] }))).toBe(
      true,
    )
  })

  it('draws the card when the page says what the transformation acts on', () => {
    expect(
      hasRows(
        infobox({ target: [{ kind: 'text', text: 'Isaac', style: 'plain' }] }),
      ),
    ).toBe(true)
  })

  // Adult, the one page of sixteen with no count: its transformation is taking three pills,
  // not picking up items, so the dataset has nothing for any of the three rows. A card there
  // would draw "nessuno" twice and read as a claim about the page, which is what
  // `wiki::transformation::requires` refuses to do one layer down.
  it('draws no card when the page filled none of the three', () => {
    expect(hasRows(infobox({}))).toBe(false)
  })
})

// The card carries the two facts every kind declares — the description and what unlocks it
// — above the three that belong to a transformation. Suppressing it by the three alone was
// a defect of a few hours: Adult has a description and lost it along with the rows it does
// not have.
describe('hasCard', () => {
  const entry = (fields: Partial<Entry>): Entry => ({
    title: 'Adult',
    revid: 1,
    description: [],
    dlc: [],
    unlockedBy: null,
    infobox: infobox({}),
    sections: [],
    ...fields,
  })

  it('draws the card for a description even with none of the three rows', () => {
    expect(
      hasCard(
        infobox({}),
        entry({
          description: [{ kind: 'text', text: 'three pills', style: 'plain' }],
        }),
      ),
    ).toBe(true)
  })

  it('draws the card for what unlocks it even with none of the three rows', () => {
    expect(
      hasCard(
        infobox({}),
        entry({ unlockedBy: { kind: 'achievement', id: 1 } }),
      ),
    ).toBe(true)
  })

  it('draws no card when the page filled nothing at all', () => {
    expect(hasCard(infobox({}), entry({}))).toBe(false)
  })
})
