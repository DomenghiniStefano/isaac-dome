import { describe, expect, it } from 'vitest'
import { goalsView } from './tabView'

describe("Goals' reading", () => {
  it('is nothing typed when there is nothing to read', () => {
    expect(goalsView.empty()).toEqual({ typed: '' })
  })

  // What was being typed into the want bar comes back with the tab (#79): a switch, a back or a
  // tear-off halfway through a word used to clear it.
  it('reads back what was being typed', () => {
    const written = { typed: 'mom' }
    expect(goalsView.read(JSON.parse(JSON.stringify(written)))).toEqual(written)
  })

  it('reads anything that is not a reading as nothing', () => {
    expect(goalsView.read(null)).toBeNull()
    expect(goalsView.read('mom')).toBeNull()
  })

  it('reads words that are not a string as nothing typed', () => {
    expect(goalsView.read({ typed: 7 })).toEqual({ typed: '' })
  })
})
