import { describe, expect, it } from 'vitest'
import { rememberFocus } from './focusOrder'

describe('rememberFocus', () => {
  it('puts the window that just took the focus first', () => {
    expect(rememberFocus(['a', 'b'], 'b')).toEqual(['b', 'a'])
  })

  it('never holds the same window twice', () => {
    expect(rememberFocus(['a', 'b', 'a'], 'a')).toEqual(['a', 'b'])
  })

  it('learns a window it had never seen', () => {
    expect(rememberFocus([], 'win-1')).toEqual(['win-1'])
  })

  it('leaves the order of the others alone', () => {
    expect(rememberFocus(['a', 'b', 'c'], 'c')).toEqual(['c', 'a', 'b'])
  })
})
