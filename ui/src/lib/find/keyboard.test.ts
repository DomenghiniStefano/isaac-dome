import { describe, expect, it } from 'vitest'
import { FindAction, findAction, opensFind } from './keyboard'

// Positional, the way `shell/navigation.test.ts` writes the same helper: key, then ctrl,
// then shift, then alt.
const press = (
  key: string,
  ctrlKey = false,
  shiftKey = false,
  altKey = false,
) => ({ key, ctrlKey, shiftKey, altKey }) as KeyboardEvent

describe('opensFind', () => {
  it('opens on Ctrl+F, the gesture every reader already knows', () => {
    expect(opensFind(press('f', true))).toBe(true)
  })

  it('opens on Ctrl+F with caps lock on, because the key is the same key', () => {
    expect(opensFind(press('F', true))).toBe(true)
  })

  it('leaves a bare f alone, or nobody could type the letter', () => {
    expect(opensFind(press('f'))).toBe(false)
  })

  it('leaves Ctrl+Shift+F alone, a combination this bar has no claim on', () => {
    expect(opensFind(press('f', true, true))).toBe(false)
  })

  it('leaves Ctrl+Alt+F alone', () => {
    expect(opensFind(press('f', true, false, true))).toBe(false)
  })
})

describe('findAction', () => {
  it('walks forward on Enter', () => {
    expect(findAction(press('Enter'))).toBe(FindAction.Next)
  })

  it('walks back on Shift+Enter', () => {
    expect(findAction(press('Enter', false, true))).toBe(FindAction.Previous)
  })

  it('closes on Escape', () => {
    expect(findAction(press('Escape'))).toBe(FindAction.Close)
  })

  it('lets every other key through, because the bar is a text field first', () => {
    expect(findAction(press('a'))).toBeNull()
    expect(findAction(press('ArrowDown'))).toBeNull()
  })

  it('does not claim Ctrl+Enter, which is the palette gesture and not this one', () => {
    // B65: two things reading the same key in one screen is how Ctrl+Enter got eaten.
    expect(findAction(press('Enter', true))).toBeNull()
  })

  it('does not claim Alt+Enter either', () => {
    expect(findAction(press('Enter', false, false, true))).toBeNull()
  })
})
