import { describe, expect, it } from 'vitest'
import { singleQuery } from './queryParam'

describe('singleQuery', () => {
  it('takes one string and nothing else', () => {
    expect(singleQuery('brimstone')).toBe('brimstone')
    // The router hands an array when a key repeats, and null when it has no value.
    expect(singleQuery(['a', 'b'])).toBeNull()
    expect(singleQuery(null)).toBeNull()
    expect(singleQuery(undefined)).toBeNull()
  })
})
