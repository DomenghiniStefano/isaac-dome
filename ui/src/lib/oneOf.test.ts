import { describe, expect, it } from 'vitest'
import { oneOf } from './oneOf'

const Colour = { Red: 'red', Blue: 'blue' } as const
type Colour = (typeof Colour)[keyof typeof Colour]

describe('oneOf', () => {
  it('narrows a string that is one of the set', () => {
    const found: Colour | undefined = oneOf(Colour, 'blue')
    expect(found).toBe(Colour.Blue)
  })

  // The caller's fallback depends on telling "not in the set" from a value: a filter hands back
  // strings, and one outside the set is shown as it came rather than dropped.
  it('answers undefined for a string that is not', () => {
    expect(oneOf(Colour, 'green')).toBeUndefined()
  })

  it('reads the values and not the keys', () => {
    expect(oneOf(Colour, 'Red')).toBeUndefined()
  })
})
