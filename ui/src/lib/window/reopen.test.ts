import { describe, expect, it } from 'vitest'
import { cascadeBox } from './reopen'

describe('cascadeBox', () => {
  const first = { left: 100, top: 50, width: 1280, height: 800 }

  it("keeps the first window's size, one step down and right per window", () => {
    expect(cascadeBox(first, 1)).toEqual({
      left: 132,
      top: 82,
      width: 1280,
      height: 800,
    })
    expect(cascadeBox(first, 3)).toEqual({
      left: 196,
      top: 146,
      width: 1280,
      height: 800,
    })
  })

  it('copies the four numbers of a box and nothing else it carries', () => {
    const measured = { ...first, label: 'main', scaleFactor: 1.5 }
    expect(cascadeBox(measured, 0)).toStrictEqual(first)
  })
})
