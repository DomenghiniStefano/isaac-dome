import { describe, expect, it } from 'vitest'
import {
  HistoryAction,
  historyAction,
  pointerHistoryAction,
} from './navigation'

const press = (key: string, altKey = true, ctrlKey = false) =>
  historyAction({ key, altKey, ctrlKey, shiftKey: false } as KeyboardEvent)
const click = (button: number) => pointerHistoryAction({ button } as MouseEvent)

describe('historyAction', () => {
  it("reads the browser's own back and forward keys", () => {
    expect(press('ArrowLeft')).toBe(HistoryAction.Back)
    expect(press('ArrowRight')).toBe(HistoryAction.Forward)
  })

  it('leaves a bare arrow to the table or list that is listening for it', () => {
    expect(press('ArrowLeft', false)).toBeNull()
    expect(press('ArrowRight', false)).toBeNull()
  })

  it('leaves everything else alone', () => {
    expect(press('ArrowUp')).toBeNull()
    expect(press('ArrowLeft', true, true)).toBeNull()
    expect(press('k', false)).toBeNull()
  })
})

describe('pointerHistoryAction', () => {
  it('reads the two side buttons of the mouse', () => {
    expect(click(3)).toBe(HistoryAction.Back)
    expect(click(4)).toBe(HistoryAction.Forward)
  })

  it('leaves the three buttons the app already uses alone', () => {
    expect(click(0)).toBeNull()
    expect(click(1)).toBeNull()
    expect(click(2)).toBeNull()
  })
})
