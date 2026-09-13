import { describe, expect, it } from 'vitest'
import { RouteName, WikiCategory } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import {
  HistoryDepth,
  backTab,
  canGoBack,
  canGoForward,
  closeTab,
  firstState,
  forwardTab,
  moveTab,
  navigateTab,
  openTab,
  refineTab,
  selectTab,
  tabLabel,
  tabLocation,
} from './tabModel'
import type { Tab, TabsState } from './tabModel'

const at = (name: TabLocation['name']): TabLocation => ({ name })
const one = (id: string, name: TabLocation['name']): Tab => ({
  id,
  entries: [at(name)],
  index: 0,
})
const three = (): TabsState => ({
  tabs: [
    one('a', RouteName.NextSteps),
    one('b', RouteName.Unlock),
    one('c', RouteName.Plan),
  ],
  activeId: 'b',
})
const ids = (s: TabsState) => s.tabs.map((t) => t.id)
const fresh = (): Tab => one('new', RouteName.NextSteps)
const activeTab = (s: TabsState): Tab =>
  s.tabs.filter((t) => t.id === s.activeId)[0]
const where = (s: TabsState) => tabLocation(activeTab(s)).name

describe('tabModel', () => {
  it('starts with one active tab', () => {
    expect(firstState('a', at(RouteName.NextSteps))).toEqual({
      tabs: [one('a', RouteName.NextSteps)],
      activeId: 'a',
    })
  })

  it('opens a tab right after the active one and activates it', () => {
    const s = openTab(three(), 'd', at(RouteName.Wiki))
    expect(ids(s)).toEqual(['a', 'b', 'd', 'c'])
    expect(s.activeId).toBe('d')
  })

  it('selects an existing tab and ignores an unknown id', () => {
    expect(selectTab(three(), 'c').activeId).toBe('c')
    expect(selectTab(three(), 'zz').activeId).toBe('b')
  })

  it('closing the active tab activates its right neighbour', () => {
    const s = closeTab(three(), 'b', fresh)
    expect(ids(s)).toEqual(['a', 'c'])
    expect(s.activeId).toBe('c')
  })

  it('closing the active last tab activates its left neighbour', () => {
    const s = closeTab({ ...three(), activeId: 'c' }, 'c', fresh)
    expect(s.activeId).toBe('b')
  })

  it('closing an inactive tab leaves the active one alone', () => {
    const s = closeTab(three(), 'a', fresh)
    expect(ids(s)).toEqual(['b', 'c'])
    expect(s.activeId).toBe('b')
  })

  it('never leaves the bar empty: closing the only tab opens a fresh one', () => {
    const s = closeTab(firstState('a', at(RouteName.Plan)), 'a', fresh)
    expect(s).toEqual({ tabs: [fresh()], activeId: 'new' })
  })

  it('moves a tab to its final index', () => {
    expect(ids(moveTab(three(), 0, 2))).toEqual(['b', 'c', 'a'])
    expect(ids(moveTab(three(), 2, 0))).toEqual(['c', 'a', 'b'])
  })

  it('navigating moves the active tab and nothing else', () => {
    const s = navigateTab(three(), at(RouteName.Profile))
    expect(s.tabs.map((t) => tabLocation(t).name)).toEqual([
      RouteName.NextSteps,
      RouteName.Profile,
      RouteName.Plan,
    ])
    expect(s.activeId).toBe('b')
  })
})

describe('tab history', () => {
  it('has nothing behind or ahead of a tab that just opened', () => {
    const s = openTab(three(), 'd', at(RouteName.Wiki))
    expect(canGoBack(activeTab(s))).toBe(false)
    expect(canGoForward(activeTab(s))).toBe(false)
  })

  it('stacks a navigation, and back returns the location it left', () => {
    const s = navigateTab(three(), at(RouteName.Profile))
    expect(canGoBack(activeTab(s))).toBe(true)
    expect(where(backTab(s))).toBe(RouteName.Unlock)
  })

  it('forward returns to the location back left behind', () => {
    const s = backTab(navigateTab(three(), at(RouteName.Profile)))
    expect(canGoForward(activeTab(s))).toBe(true)
    expect(where(forwardTab(s))).toBe(RouteName.Profile)
  })

  it('navigating after a back drops what was ahead', () => {
    const s = navigateTab(
      backTab(navigateTab(three(), at(RouteName.Profile))),
      at(RouteName.Collection),
    )
    expect(canGoForward(activeTab(s))).toBe(false)
    expect(where(backTab(s))).toBe(RouteName.Unlock)
  })

  it('does nothing at either end of the history', () => {
    const s = three()
    expect(canGoBack(activeTab(s))).toBe(false)
    expect(backTab(s)).toEqual(s)
    const ahead = navigateTab(s, at(RouteName.Profile))
    expect(forwardTab(ahead)).toEqual(ahead)
  })

  it('moves only the active tab', () => {
    const s = backTab(navigateTab(three(), at(RouteName.Profile)))
    expect(s.tabs.map((t) => tabLocation(t).name)).toEqual([
      RouteName.NextSteps,
      RouteName.Unlock,
      RouteName.Plan,
    ])
  })

  // What the user typed is state inside the search, not a location of its own: one back
  // from a search leaves it for the route you were on, never for a previous keystroke.
  it('replaces the entry when only the search query changes', () => {
    const typed = (q: string): TabLocation => ({
      name: RouteName.Search,
      query: { q },
    })
    const s = ['g', 'gi', 'gim'].reduce(
      (state, q) => navigateTab(state, typed(q)),
      three(),
    )
    expect(tabLocation(activeTab(s)).query).toEqual({ q: 'gim' })
    expect(where(backTab(s))).toBe(RouteName.Unlock)
  })

  // Two wiki pages are two locations even though they share a route: the page is what the
  // tab is, the query `q` is not.
  it('stacks two pages of the same wiki category', () => {
    const page = (key: string): TabLocation => ({
      name: RouteName.Wiki,
      query: { category: WikiCategory.Items, page: key },
    })
    const s = navigateTab(
      navigateTab(three(), page('item:105')),
      page('item:9'),
    )
    expect(tabLocation(activeTab(backTab(s))).query?.page).toBe('item:105')
  })

  // The search navigates on a debounce: the keystroke can land after the user has already
  // gone back, and a tab dragged to a screen the user has left would make back unusable.
  it('refines the current entry only while the tab is still on that view', () => {
    const typed = (q: string): TabLocation => ({
      name: RouteName.Search,
      query: { q },
    })
    const searching = navigateTab(three(), typed('gim'))
    expect(
      tabLocation(activeTab(refineTab(searching, typed('gimpy')))).query,
    ).toEqual({ q: 'gimpy' })
    const left = backTab(searching)
    expect(refineTab(left, typed('gimpy'))).toEqual(left)
  })

  it('opening a tab does not inherit the history of the one it came from', () => {
    const s = openTab(
      navigateTab(three(), at(RouteName.Profile)),
      'd',
      at(RouteName.Wiki),
    )
    expect(canGoBack(activeTab(s))).toBe(false)
  })

  it('keeps the history bounded, dropping the oldest and never the current', () => {
    const names = [RouteName.Unlock, RouteName.Plan]
    const s = Array.from({ length: HistoryDepth + 5 }).reduce<TabsState>(
      (state, _, i) => navigateTab(state, at(names[i % 2])),
      three(),
    )
    const tab = activeTab(s)
    expect(tab.entries).toHaveLength(HistoryDepth)
    expect(tabLocation(tab).name).toBe(names[(HistoryDepth + 4) % 2])
    expect(canGoForward(tab)).toBe(false)
  })
})

describe('tabLabel', () => {
  const titles: Record<string, string> = { 'item:105': 'The D6' }
  const titleOf = (key: string) => titles[key] ?? null
  const page = (key: string): TabLocation => ({
    name: RouteName.Wiki,
    query: { category: WikiCategory.Items, page: key },
  })

  it("is the page's title when the index knows it", () => {
    expect(tabLabel(page('item:105'), titleOf)).toEqual({ text: 'The D6' })
  })

  it("is the category's message until the index knows the page", () => {
    expect(tabLabel(page('item:9'), titleOf)).toBe('wikiCategories.items')
  })

  it("is the route's message on any other location", () => {
    expect(tabLabel(at(RouteName.Plan), titleOf)).toBe('routes.plan')
    expect(
      tabLabel(
        { name: RouteName.Wiki, query: { category: WikiCategory.Bosses } },
        titleOf,
      ),
    ).toBe('wikiCategories.bosses')
  })
})
