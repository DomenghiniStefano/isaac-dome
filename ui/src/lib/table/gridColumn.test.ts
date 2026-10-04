import { describe, expect, it } from 'vitest'
import { rowWidePx, rowWikiPx } from '@/lib/scale/rows'
import {
  ACTIONS_KEY,
  ColumnAlign,
  ColumnFold,
  RowHeight,
  cellClass,
  cellStyle,
  fixed,
  grow,
  rowHeightClass,
  rowHeightPx,
  withActions,
} from './gridColumn'
import type { GridColumn } from './gridColumn'

const name: GridColumn = {
  key: 'name',
  header: 'challenges.columns.challenge',
  width: grow(1.4),
  fold: ColumnFold.Never,
  align: ColumnAlign.Start,
}
const state: GridColumn = {
  key: 'state',
  header: 'challenges.columns.state',
  width: fixed('w-challenge-state'),
  fold: ColumnFold.Compact,
  align: ColumnAlign.End,
}

describe('the Actions column', () => {
  it('is appended last, once', () => {
    const all = withActions([name, state])
    expect(all.map((c) => c.key)).toEqual(['name', 'state', ACTIONS_KEY])
  })

  it('is a fixed column that never folds', () => {
    const actions = withActions([])[0]
    expect(actions.fold).toBe(ColumnFold.Never)
    expect(actions.width).toEqual(fixed('w-actions'))
    expect(actions.header).toBe('table.actions')
  })
})

describe('a cell', () => {
  it('always carries the containment', () => {
    for (const column of [name, state])
      expect(cellClass(column)).toEqual(
        expect.arrayContaining(['grid-cell', 'min-w-0']),
      )
  })

  it('takes its fixed width from the token class, and does not shrink', () => {
    expect(cellClass(state)).toEqual(
      expect.arrayContaining(['w-challenge-state', 'shrink-0']),
    )
    expect(cellStyle(state)).toBeUndefined()
  })

  it('grows by its weight through a variable', () => {
    expect(cellClass(name)).toContain('cell-grow')
    expect(cellStyle(name)).toEqual({ '--cell-grow': '1.4' })
  })

  it('folds at compact only when its column says so', () => {
    expect(cellClass(state)).toContain('@max-compact/page:hidden')
    expect(cellClass(name)).not.toContain('@max-compact/page:hidden')
  })

  it('aligns as its column says', () => {
    expect(cellClass(state)).toContain('justify-end')
    expect(cellClass(name)).toContain('justify-start')
  })
})

describe('a row height', () => {
  it('names the token and the number the virtualizer counts with', () => {
    expect(rowHeightClass[RowHeight.Wide]).toBe('h-row-wide')
    expect(rowHeightClass[RowHeight.Wiki]).toBe('h-row-wiki')
    expect(rowHeightPx[RowHeight.Wide]).toBe(rowWidePx)
    expect(rowHeightPx[RowHeight.Wiki]).toBe(rowWikiPx)
  })
})
