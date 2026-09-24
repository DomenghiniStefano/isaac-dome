import { describe, expect, it } from 'vitest'
import { togglesSidebar } from './sidebarToggle'

const press = (
  key: string,
  { ctrlKey = true, shiftKey = false, altKey = false, repeat = false } = {},
) =>
  togglesSidebar({ key, ctrlKey, shiftKey, altKey, repeat } as KeyboardEvent)

describe('togglesSidebar', () => {
  it('is Ctrl+B', () => {
    expect(press('b')).toBe(true)
  })

  // Caps Lock reports the capital, and the gesture is the same one.
  it('is Ctrl+B with Caps Lock on', () => {
    expect(press('B')).toBe(true)
  })

  it('is not a bare b, which somebody is typing', () => {
    expect(press('b', { ctrlKey: false })).toBe(false)
  })

  it('leaves the other combinations on B to whoever claims them', () => {
    expect(press('b', { shiftKey: true })).toBe(false)
    expect(press('b', { altKey: true })).toBe(false)
  })

  // Held down, the key repeats: the sidebar would fold and unfold for as long as the finger stayed
  // there, and land wherever it happened to be when it lifted.
  it('ignores the repeats of a key held down', () => {
    expect(press('b', { repeat: true })).toBe(false)
  })

  it('is not any other Ctrl shortcut', () => {
    expect(press('k')).toBe(false)
    expect(press('f')).toBe(false)
  })
})
