import { describe, expect, it } from 'vitest'
import base from '@/assets/base.css?raw'
import spacing from '@/assets/theme/spacing.css?raw'
import { RootFontPx, RowWideRem, remToPx, rowWidePx } from './rows'

// The three virtualized tables (Unlock, the Collection, the Wiki's lists) position their
// rows with a number while the rows are drawn with a token. If the two disagree the rows
// overlap or leave gaps, and nothing else fails.
describe('the virtualized row', () => {
  it('is the row-wide token, read from the CSS', () => {
    expect(spacing).toMatch(
      new RegExp(String.raw`--spacing-row-wide:\s*${RowWideRem}rem;`),
    )
  })

  it('is measured against the same root size base.css uses', () => {
    expect(base).toMatch(
      new RegExp(String.raw`font-size:\s*calc\(${RootFontPx}px \*`),
    )
  })

  it('is 40px at scale 100 and follows every step', () => {
    expect(rowWidePx(100)).toBe(40)
    expect(rowWidePx(50)).toBe(20)
    expect(rowWidePx(200)).toBe(80)
    expect(remToPx(1, 125)).toBe(20)
  })
})
