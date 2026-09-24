import { describe, expect, it } from 'vitest'
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import {
  backTab,
  entryScroll,
  navigateTab,
  selectTab,
  setEntryScroll,
  setEntryView,
  tabSeed,
} from './tabModel'
import type { TabsState } from './tabModel'

const at = (name: TabLocation['name']): TabLocation => ({ name })

// Two tabs, `a` on Floor and `b` on Unlock, `a` active.
const two = (): TabsState => ({
  tabs: [
    { id: 'a', entries: [{ location: at(RouteName.Floor) }], index: 0 },
    { id: 'b', entries: [{ location: at(RouteName.Unlock) }], index: 0 },
  ],
  activeId: 'a',
})
const tab = (state: TabsState, id: string) =>
  state.tabs.find((each) => each.id === id)!

describe('where a tab was scrolled to', () => {
  it('starts scrolled nowhere', () => {
    expect(entryScroll(tab(two(), 'a'))).toEqual({})
  })

  it('keeps one position per named region of the entry', () => {
    const once = setEntryScroll(two(), 'a', 0, at(RouteName.Floor), 'page', 480)
    const twice = setEntryScroll(once, 'a', 0, at(RouteName.Floor), 'pane', 90)
    expect(entryScroll(tab(twice, 'a'))).toEqual({ page: 480, pane: 90 })
  })

  // The write is addressed, not aimed at "the active tab": a screen saves its last position as it
  // is taken down, and by then the tab it belongs to is no longer the one showing.
  it('writes into the tab it names even when another tab is active', () => {
    const switched = selectTab(two(), 'b')
    const state = setEntryScroll(
      switched,
      'a',
      0,
      at(RouteName.Floor),
      'page',
      300,
    )
    expect(entryScroll(tab(state, 'a'))).toEqual({ page: 300 })
    expect(entryScroll(tab(state, 'b'))).toEqual({})
  })

  // Back and forward are the reason this exists: each entry keeps its own position, so going back
  // lands where that page was left and not where the next one was.
  it('keeps a position per history entry', () => {
    const first = setEntryScroll(
      two(),
      'a',
      0,
      at(RouteName.Floor),
      'page',
      700,
    )
    const moved = navigateTab(first, at(RouteName.Collection))
    const second = setEntryScroll(
      moved,
      'a',
      1,
      at(RouteName.Collection),
      'page',
      40,
    )
    const back = backTab(second)
    expect(entryScroll(tab(back, 'a'))).toEqual({ page: 700 })
  })

  // The same guard the view has: a write that lands late must reach no entry rather than the wrong
  // one — the tab gone, the entry gone, or a different page at that place in the history.
  it('refuses a write whose tab, entry or location is no longer there', () => {
    const state = two()
    expect(
      setEntryScroll(state, 'gone', 0, at(RouteName.Floor), 'page', 1),
    ).toBe(state)
    expect(setEntryScroll(state, 'a', 3, at(RouteName.Floor), 'page', 1)).toBe(
      state,
    )
    expect(setEntryScroll(state, 'a', 0, at(RouteName.Unlock), 'page', 1)).toBe(
      state,
    )
  })

  it('refuses a position that is not a distance', () => {
    const state = two()
    expect(setEntryScroll(state, 'a', 0, at(RouteName.Floor), 'page', -5)).toBe(
      state,
    )
    expect(
      setEntryScroll(state, 'a', 0, at(RouteName.Floor), 'page', Number.NaN),
    ).toBe(state)
  })

  // The screen's own reading and the shell's positions are two fields of one entry, and writing one
  // must not erase the other.
  it('is kept apart from the view, and the view from it', () => {
    const scrolled = setEntryScroll(
      two(),
      'a',
      0,
      at(RouteName.Floor),
      'page',
      250,
    )
    const viewed = setEntryView(scrolled, at(RouteName.Floor), {
      shown: 'secret',
    })
    expect(entryScroll(tab(viewed, 'a'))).toEqual({ page: 250 })
    const rescrolled = setEntryScroll(
      viewed,
      'a',
      0,
      at(RouteName.Floor),
      'page',
      260,
    )
    expect(tab(rescrolled, 'a').entries[0]?.view).toEqual({ shown: 'secret' })
  })

  it('travels with a torn-off tab', () => {
    const state = setEntryScroll(
      two(),
      'a',
      0,
      at(RouteName.Floor),
      'page',
      510,
    )
    expect(tabSeed(tab(state, 'a')).entries[0]?.scroll).toEqual({ page: 510 })
  })
})
