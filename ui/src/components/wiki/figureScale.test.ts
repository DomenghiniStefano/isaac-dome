import { describe, expect, it } from 'vitest'
import { FigureBoxPx, integerScale } from './figureScale'
import { FigureSize } from './figureSize'

describe('integerScale', () => {
  it.each<[number, number, number]>([
    [32, 64, 2],
    [32, 70, 2],
    [48, 32, 1],
    [16, 192, 12],
  ])('a %ipx sprite in a %ipx box scales to %i', (native, box, expected) => {
    expect(integerScale(native, box)).toBe(expected)
  })

  it('never returns less than 1, even when the sprite is bigger than the box', () => {
    expect(integerScale(64, 1)).toBe(1)
  })
})

describe('FigureBoxPx', () => {
  it('is a whole multiple of the native sprite for every size, so the sprite fills its box', () => {
    for (const size of Object.values(FigureSize)) {
      const boxPx = FigureBoxPx[size]
      expect(boxPx % 32).toBe(0)
    }
  })
})
