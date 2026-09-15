import { describe, expect, it } from 'vitest'
import { RouteName, WikiCategory } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import {
  HistoryDepth,
  backTab,
  canDetach,
  canGoBack,
  canGoForward,
  closeTab,
  detachTab,
  entryView,
  firstState,
  forwardTab,
  insertTab,
  moveTab,
  navigateTab,
  openTab,
  refineTab,
  removeTab,
  seedState,
  setEntryView,
  sessionOf,
  selectTab,
  tabLabel,
  tabLocation,
  tabSeed,
} from './tabModel'
import type { Entry, Tab, TabsState } from './tabModel'

const at = (name: TabLocation['name']): TabLocation => ({ name })
const entry = (name: TabLocation['name']): Entry => ({ location: at(name) })
const one = (id: string, name: TabLocation['name']): Tab => ({
  id,
  entries: [entry(name)],
  index: 0,
})
const three = (): TabsState => ({
  tabs: [
    one('a', RouteName.Goals),
    one('b', RouteName.Unlock),
    one('c', RouteName.Plan),
  ],
  activeId: 'b',
})
const ids = (s: TabsState) => s.tabs.map((t) => t.id)
const fresh = (): Tab => one('new', RouteName.Goals)
const activeTab = (s: TabsState): Tab =>
  s.tabs.filter((t) => t.id === s.activeId)[0]
const where = (s: TabsState) => tabLocation(activeTab(s)).name

describe('tabModel', () => {
  it('starts with one active tab', () => {
    expect(firstState('a', at(RouteName.Goals))).toEqual({
      tabs: [one('a', RouteName.Goals)],
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
      RouteName.Goals,
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
      RouteName.Goals,
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

// A tab that travels between windows carries everything but its identity: these say so without
// naming a single field of a tab, so what a tab is made of can change under them.
const seedOf = (tab: Tab) => tabSeed(tab)
const someTab = (id: string): Tab => ({ id, ...seedOf(three().tabs[0] as Tab) })

describe('a tab that leaves, and one that arrives', () => {
  it('inserts an arriving tab at the index and selects it: a dropped tab is the one you want', () => {
    const after = insertTab(three(), 1, someTab('d'))
    expect(ids(after)).toEqual(['a', 'd', 'b', 'c'])
    expect(after.activeId).toBe('d')
  })

  it('clamps an index past either end rather than dropping the tab', () => {
    expect(ids(insertTab(three(), 99, someTab('d')))).toEqual([
      'a',
      'b',
      'c',
      'd',
    ])
    expect(ids(insertTab(three(), -3, someTab('d')))).toEqual([
      'd',
      'a',
      'b',
      'c',
    ])
  })

  it('detaching hands back the tab and the state without it', () => {
    const out = detachTab(three(), 'b')
    expect(out?.tab.id).toBe('b')
    expect(out ? ids(out.state) : null).toEqual(['a', 'c'])
    // The active tab left, so its right neighbour takes over, exactly as closing does.
    expect(out?.state.activeId).toBe('c')
  })

  it('detaching a tab that is not there answers null and changes nothing', () => {
    expect(detachTab(three(), 'zzz')).toBeNull()
  })

  it('the last tab does not detach: that window already is that tab', () => {
    const one: TabsState = { tabs: [someTab('a')], activeId: 'a' }
    expect(canDetach(one)).toBe(false)
    expect(detachTab(one, 'a')).toBeNull()
    expect(canDetach(three())).toBe(true)
  })

  it('a seeded window holds what it was given, active where it was told', () => {
    const seeds = three().tabs.map(seedOf)
    const state = seedState(seeds, 1, (n) => `tab-${n}`)
    expect(state.tabs).toHaveLength(3)
    expect(state.activeId).toBe(state.tabs[1]?.id)
    // The seeds carried everything but the identity, and the identity is this window's.
    expect(state.tabs.map(seedOf)).toEqual(seeds)
    expect(ids(state)).toEqual(['tab-0', 'tab-1', 'tab-2'])
  })

  it('drops the identity from the seed, so a fresh id cannot be overwritten by an old one', () => {
    const seed = tabSeed(someTab('a'))
    expect('id' in seed).toBe(false)
    expect({ id: 'b', ...seed }.id).toBe('b')
  })

  it('lifting a tab out leaves the bar without it, and says where it was', () => {
    const out = removeTab(three(), 'b')
    expect(out?.index).toBe(1)
    expect(out ? ids(out.state) : null).toEqual(['a', 'c'])
    expect(out?.state.activeId).toBe('c')
    expect(out?.seed).toEqual(tabSeed(three().tabs[1] as Tab))
  })

  // The difference from `detachTab`: a tab in flight has left the strip but has not landed
  // anywhere, so the bar it left **may be empty** — the window stays open because the tab can
  // still come back. What happens to an empty window is decided when the drag ends, not here.
  it('lets the last tab leave, and leaves the bar empty', () => {
    const one: TabsState = { tabs: [someTab('a')], activeId: 'a' }
    const out = removeTab(one, 'a')
    expect(out?.state.tabs).toEqual([])
    expect(out?.state.activeId).toBe('')
    expect(out?.index).toBe(0)
  })

  it('lifting a tab that is not there answers null', () => {
    expect(removeTab(three(), 'zzz')).toBeNull()
  })

  it('a seed with nothing in it still leaves a bar with one tab', () => {
    const state = seedState([], 0, (n) => `tab-${n}`)
    expect(state.tabs).toHaveLength(1)
    expect(state.activeId).toBe(state.tabs[0]?.id)
  })

  it('an active index past the seeds still selects a tab that exists', () => {
    const state = seedState(three().tabs.map(seedOf), 9, (n) => `tab-${n}`)
    expect(state.activeId).toBe('tab-2')
  })
})

describe('what a window would save of itself', () => {
  it('is its tabs without their ids, and which one is active', () => {
    const session = sessionOf(three())
    expect(session.activeIndex).toBe(1)
    expect(session.tabs).toHaveLength(3)
    expect(JSON.stringify(session)).not.toContain('"id"')
  })

  it('is what seedState reads back: the same tabs, the same one active', () => {
    const session = sessionOf(three())
    const back = seedState(session.tabs, session.activeIndex, (n) => `t-${n}`)
    expect(back.tabs.map((t) => t.entries)).toEqual(
      three().tabs.map((t) => t.entries),
    )
    expect(back.activeId).toBe('t-1')
  })

  it('answers 0 rather than -1 for a window with no tabs', () => {
    // A window mid-tear-off holds nothing for an instant. -1 in a stored document would be a
    // number nothing means.
    expect(sessionOf({ tabs: [], activeId: '' })).toEqual({
      tabs: [],
      activeIndex: 0,
    })
  })
})

describe('the view a tab is holding', () => {
  it('starts with no view at all', () => {
    const state = firstState('a', at(RouteName.Unlock))
    expect(entryView(activeTab(state))).toBeUndefined()
  })

  it('writes the view into the entry the tab is showing', () => {
    const state = setEntryView(
      firstState('a', at(RouteName.Unlock)),
      at(RouteName.Unlock),
      { sort: 'name' },
    )
    expect(entryView(activeTab(state))).toEqual({ sort: 'name' })
    // The location is untouched: a view is not a navigation.
    expect(tabLocation(activeTab(state))).toEqual(at(RouteName.Unlock))
  })

  // The same guard `refineTab` has, and for the same reason: a debounced write can land after
  // the user has gone back or switched tab, and it must reach no tab rather than the wrong one.
  it('refuses a view whose location is not the one the tab is showing', () => {
    const state = firstState('a', at(RouteName.Unlock))
    expect(
      setEntryView(state, at(RouteName.Collection), { sort: 'name' }),
    ).toBe(state)
  })

  it('leaves the views of the other entries alone when it writes', () => {
    const start = navigateTab(
      setEntryView(
        firstState('a', at(RouteName.Unlock)),
        at(RouteName.Unlock),
        {
          sort: 'name',
        },
      ),
      at(RouteName.Collection),
    )
    const state = setEntryView(start, at(RouteName.Collection), { sort: 'id' })
    const tab = activeTab(state)
    expect(tab.entries[0]?.view).toEqual({ sort: 'name' })
    expect(tab.entries[1]?.view).toEqual({ sort: 'id' })
  })

  // Going back is going back to what you were looking at, which is the whole reason the record
  // sits on the entry and not on the tab.
  it('gives back the view of the entry it returns to', () => {
    const start = navigateTab(
      setEntryView(
        firstState('a', at(RouteName.Unlock)),
        at(RouteName.Unlock),
        {
          sort: 'name',
        },
      ),
      at(RouteName.Collection),
    )
    expect(entryView(activeTab(backTab(start)))).toEqual({ sort: 'name' })
  })

  // `tabSeed` is written by subtraction, so this holds without a line being added for it. The
  // test exists because that is the property the tear-off rests on.
  it('carries the view across a tear-off', () => {
    const state = setEntryView(
      firstState('a', at(RouteName.Unlock)),
      at(RouteName.Unlock),
      { sort: 'name' },
    )
    expect(tabSeed(activeTab(state)).entries[0]?.view).toEqual({ sort: 'name' })
  })

  // A refinement replaces the entry in place, and the view belongs to the view, not to the
  // query string that refined it.
  it('keeps the view when the same view is refined', () => {
    const start = setEntryView(
      firstState('a', at(RouteName.Unlock)),
      at(RouteName.Unlock),
      { sort: 'name' },
    )
    const state = refineTab(start, {
      name: RouteName.Unlock,
      query: { q: 'brim' },
    })
    expect(entryView(activeTab(state))).toEqual({ sort: 'name' })
    expect(tabLocation(activeTab(state)).query?.q).toBe('brim')
  })
})
