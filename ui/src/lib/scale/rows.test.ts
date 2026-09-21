import { describe, expect, it } from 'vitest'
import base from '@/assets/base.css?raw'
import spacing from '@/assets/theme/spacing.css?raw'
import {
  RootFontPx,
  RowResultRem,
  RowWideRem,
  RowWikiRem,
  remToPx,
  rowResultPx,
  rowWidePx,
  rowWikiPx,
} from './rows'

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

// A search result is two lines — the title and why it matched — so it is taller than a table
// row, and it needs its own token for the same reason the others do.
describe('the search result row', () => {
  it('is the row-result token, read from the CSS', () => {
    expect(spacing).toMatch(
      new RegExp(String.raw`--spacing-row-result:\s*${RowResultRem}rem;`),
    )
  })

  it('is 56px at scale 100 and follows every step', () => {
    expect(rowResultPx(100)).toBe(56)
    expect(rowResultPx(200)).toBe(112)
  })
})

// A wiki list row carries the framed figure, the page's name and the line under it: taller
// again than a search result, and pinned to its own token for the same reason.
describe('the wiki list row', () => {
  it('is the row-wiki token, read from the CSS', () => {
    expect(spacing).toMatch(
      new RegExp(String.raw`--spacing-row-wiki:\s*${RowWikiRem}rem;`),
    )
  })

  it('is 60px at scale 100 and follows every step', () => {
    expect(rowWikiPx(100)).toBe(60)
    expect(rowWikiPx(200)).toBe(120)
  })
})
