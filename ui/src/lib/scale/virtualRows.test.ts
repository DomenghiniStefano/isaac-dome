import { describe, expect, it } from 'vitest'
import spacing from '@/assets/theme/spacing.css?raw'
import virtualRows from '@/components/ui/virtual/VirtualRows.vue?raw'
import {
  listScrollTop,
  pageScrollTop,
  totalHeightPx,
  visibleRows,
} from './virtualRows'

// Unlock, the Collection, Search and the wiki's category list each repeated the same pairing:
// a virtualizer's items carry an index and an offset, the row itself lives in a plain array at
// that index, and a filter can shrink that array after the virtualizer already measured the
// old count — so an index past the end is dropped, not drawn as `undefined`.
describe('visibleRows', () => {
  it('pairs each virtual item with the row at its index, carrying the offset as a style', () => {
    const items = [
      { index: 0, start: 0 },
      { index: 2, start: 80 },
    ]
    const rows = ['a', 'b', 'c']
    expect(visibleRows(items, rows)).toEqual([
      { index: 0, start: 0, style: { '--row-start': '0px' }, row: 'a' },
      { index: 2, start: 80, style: { '--row-start': '80px' }, row: 'c' },
    ])
  })

  it('drops an item whose index no longer has a row', () => {
    const items = [
      { index: 0, start: 0 },
      { index: 5, start: 200 },
    ]
    const rows = ['a']
    expect(visibleRows(items, rows)).toEqual([
      { index: 0, start: 0, style: { '--row-start': '0px' }, row: 'a' },
    ])
  })

  it('places a row from the top of the list, not of the page, when the page scrolls', () => {
    // With the page as the scroller, the virtualizer counts every offset from the top of the
    // page: the hero and the filters sit above the list, and the row must not land that far
    // below its own place.
    const items = [{ index: 3, start: 420 }]
    expect(visibleRows(items, ['a', 'b', 'c', 'd'], 300)).toEqual([
      { index: 3, start: 120, style: { '--row-start': '120px' }, row: 'd' },
    ])
  })

  it('is empty when there are no virtual items', () => {
    expect(visibleRows([], ['a', 'b'])).toEqual([])
  })
})

describe('totalHeightPx', () => {
  it('formats the virtualizer total as a CSS pixel length', () => {
    expect(totalHeightPx(0)).toBe('0px')
    expect(totalHeightPx(1234)).toBe('1234px')
  })
})

// **The scroll body's height stopped being a number** (spec 3.13a §4). It was
// `--spacing-virtual-rows-body: 35rem`, a token read by every virtualized list — B43's point was
// that its *name* had to say so, or `max-h-unlock-body` on the Collection read as a mistake. The
// name is moot now: the body takes the height that is left, so a tall window shows a long list and
// a short one a short list, which no token could express.
//
// What is pinned here is the replacement, because the failure is silent either way: a `max-h-`
// creeping back caps every list again at one number, and it would look like a design choice.
describe('the virtualized list body', () => {
  it('takes the height that is left, and no token pins it', () => {
    expect(virtualRows).toMatch(/['"]min-h-0 flex-1 overflow-auto['"]/)
    expect(virtualRows).not.toMatch(/\bmax-h-/)
  })

  it('leaves no body-height token behind in the theme', () => {
    expect(spacing).not.toMatch(/--spacing-virtual-rows-body/)
    expect(spacing).not.toMatch(/--spacing-unlock-body/)
  })
})

describe('the list offset when the page scrolls', () => {
  it('is how far the page scrolled past the top of the list, never below zero', () => {
    expect(listScrollTop(500, 300)).toBe(200)
    expect(listScrollTop(100, 300)).toBe(0)
  })

  it('puts the page back where the list was left', () => {
    expect(pageScrollTop(200, 300)).toBe(500)
    expect(listScrollTop(pageScrollTop(200, 300), 300)).toBe(200)
  })
})
