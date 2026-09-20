import { describe, expect, it } from 'vitest'
import { isTyping } from './typing'

// No DOM here, so the target is described by the two things the answer depends on. That is
// also the honest shape of the question: "is this element one that eats keys", not "is this an
// HTMLInputElement".
const element = (tagName: string, isContentEditable = false) =>
  ({ tagName, isContentEditable }) as unknown as EventTarget

describe('isTyping', () => {
  it('says yes inside the fields a key belongs to', () => {
    expect(isTyping(element('INPUT'))).toBe(true)
    expect(isTyping(element('TEXTAREA'))).toBe(true)
    expect(isTyping(element('SELECT'))).toBe(true)
  })

  it('says yes inside anything editable, which no tag name announces', () => {
    expect(isTyping(element('DIV', true))).toBe(true)
  })

  it('says no everywhere else, or a shortcut would never fire at all', () => {
    expect(isTyping(element('DIV'))).toBe(false)
    expect(isTyping(element('BUTTON'))).toBe(false)
  })

  it('says no when there is no target: a key with nowhere to go is not typing', () => {
    expect(isTyping(null)).toBe(false)
  })
})
