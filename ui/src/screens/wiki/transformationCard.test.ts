import { describe, expect, it } from 'vitest'
import type { Infobox } from '@/lib/ipc/types'
import { hasRows } from './transformationCard'

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
