import { describe, expect, it } from 'vitest'
import { RouteName, WikiCategory } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import {
  canDetach,
  closeTab,
  detachTab,
  firstState,
  insertTab,
  moveTab,
  navigateTab,
  openTab,
  removeTab,
  seedState,
  selectTab,
  tabLabel,
  tabSeed,
} from './tabModel'
import type { Tab, TabsState } from './tabModel'

const at = (name: TabLocation['name']): TabLocation => ({ name })
const three = (): TabsState => ({
  tabs: [
    { id: 'a', location: at(RouteName.NextSteps) },
    { id: 'b', location: at(RouteName.Unlock) },
    { id: 'c', location: at(RouteName.Plan) },
  ],
  activeId: 'b',
})
const ids = (s: TabsState) => s.tabs.map((t) => t.id)
const fresh = (): Tab => ({ id: 'new', location: at(RouteName.NextSteps) })

describe('tabModel', () => {
  it('starts with one active tab', () => {
    expect(firstState('a', at(RouteName.NextSteps))).toEqual({
      tabs: [{ id: 'a', location: at(RouteName.NextSteps) }],
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

  it('navigating replaces the active tab location and nothing else', () => {
    const s = navigateTab(three(), at(RouteName.Profile))
    expect(s.tabs.map((t) => t.location.name)).toEqual([
      RouteName.NextSteps,
      RouteName.Profile,
      RouteName.Plan,
    ])
    expect(s.activeId).toBe('b')
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
