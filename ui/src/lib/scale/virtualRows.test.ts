import { describe, expect, it } from 'vitest'
import spacing from '@/assets/theme/spacing.css?raw'
import { totalHeightPx, visibleRows } from './virtualRows'

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

// The scroll body's height is a token read by every virtualized list, not by Unlock alone
// (`docs/BACKLOG.md` B43): the name has to say so, or `max-h-unlock-body` on the Collection
// reads as a mistake to anyone who hasn't been told it isn't.
describe('the virtualized list body token', () => {
  it('is named for what reads it, not for Unlock alone', () => {
    expect(spacing).toMatch(/--spacing-virtual-rows-body:\s*35rem;/)
    expect(spacing).not.toMatch(/--spacing-unlock-body/)
  })
})
