import { describe, expect, it } from 'vitest'
import { WikiCategory } from '@/router/routeTable'
import {
  ListViewMode,
  currentWikiListView,
  listViewFor,
  readWikiListView,
  setListViewFor,
  takeWikiListView,
  wikiListView,
} from './listViewMode'

describe('listViewFor', () => {
  it('is the grid by default, for a category never switched', () => {
    wikiListView.value = {}
    expect(listViewFor(WikiCategory.Items)).toBe(ListViewMode.Grid)
  })

  it('is what the category was last switched to', () => {
    wikiListView.value = {}
    setListViewFor(WikiCategory.Items, ListViewMode.Table)
    expect(listViewFor(WikiCategory.Items)).toBe(ListViewMode.Table)
    // A per-category convenience: switching one category never moves another.
    expect(listViewFor(WikiCategory.Trinkets)).toBe(ListViewMode.Grid)
  })
})

describe('readWikiListView', () => {
  const categories = [WikiCategory.Items, WikiCategory.Trinkets]

  it('reads back what a category was stored as', () => {
    expect(
      readWikiListView({ items: 'table', trinkets: 'grid' }, categories),
    ).toEqual({ items: 'table', trinkets: 'grid' })
  })

  it('drops a category the caller does not know', () => {
    expect(
      readWikiListView({ items: 'table', bosses: 'table' }, categories),
    ).toEqual({ items: 'table' })
  })

  it('drops a value that is not a view mode, and keeps the rest', () => {
    expect(
      readWikiListView({ items: 'card', trinkets: 'grid' }, categories),
    ).toEqual({ trinkets: 'grid' })
  })

  it('is empty for anything that is not an object', () => {
    expect(readWikiListView(null, categories)).toEqual({})
    expect(readWikiListView('table', categories)).toEqual({})
    expect(readWikiListView(['table'], categories)).toEqual({})
  })
})

describe('takeWikiListView', () => {
  it('replaces what is held with a validated reading', () => {
    takeWikiListView({ items: 'table', bosses: 'nonsense' }, [
      WikiCategory.Items,
      WikiCategory.Bosses,
    ])
    expect(currentWikiListView()).toEqual({ items: 'table' })
  })
})
