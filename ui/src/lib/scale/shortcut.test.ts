import { describe, expect, it } from 'vitest'
import { ScaleAction, shortcutAction } from './shortcut'

const press = (key: string, ctrlKey = true) =>
  shortcutAction({ key, ctrlKey } as KeyboardEvent)

describe('shortcutAction', () => {
  it('reads the browser shortcuts the app takes over', () => {
    // Both keys that produce a plus: the main row needs Shift, the numpad doesn't.
    expect(press('+')).toBe(ScaleAction.In)
    expect(press('=')).toBe(ScaleAction.In)
    expect(press('-')).toBe(ScaleAction.Out)
    expect(press('0')).toBe(ScaleAction.Reset)
  })

  it('leaves everything else alone', () => {
    expect(press('+', false)).toBeNull()
    expect(press('k')).toBeNull()
    expect(press('1')).toBeNull()
  })
})
