import { describe, expect, it } from 'vitest'
import { keyAfterAnswer } from './highlight'

describe('keyAfterAnswer', () => {
  it('keeps the row the user moved to, because the new answer still has it', () => {
    expect(keyAfterAnswer([{ key: 'a' }, { key: 'b' }], 'b')).toBe('b')
  })

  it('falls back to the first row, because the one it was on has gone', () => {
    expect(keyAfterAnswer([{ key: 'a' }, { key: 'b' }], 'z')).toBe('a')
  })

  it('takes the first row when nothing was highlighted yet', () => {
    expect(keyAfterAnswer([{ key: 'a' }], null)).toBe('a')
  })

  it('has nothing to highlight in an empty answer', () => {
    expect(keyAfterAnswer([], 'a')).toBeNull()
  })
})
