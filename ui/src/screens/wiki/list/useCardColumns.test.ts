import { describe, expect, it } from 'vitest'
import { columnsForWidth } from './useCardColumns'

describe('columnsForWidth', () => {
  it('is one card when the box holds less than two', () => {
    expect(columnsForWidth(100, 192, 12)).toBe(1)
    expect(columnsForWidth(0, 192, 12)).toBe(1)
  })

  it('is as many cards as fit, gap included', () => {
    // Two 192px cards and one 12px gap between them need 396px.
    expect(columnsForWidth(396, 192, 12)).toBe(2)
    expect(columnsForWidth(395, 192, 12)).toBe(1)
  })

  it('grows with the box', () => {
    expect(columnsForWidth(1280, 192, 12)).toBeGreaterThan(
      columnsForWidth(640, 192, 12),
    )
  })
})
