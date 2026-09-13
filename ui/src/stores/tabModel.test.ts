import { describe, expect, it } from 'vitest'
import { RouteName, WikiCategory } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import {
  closeTab,
  firstState,
  moveTab,
  navigateTab,
  openTab,
  selectTab,
  tabLabel,
} from './tabModel'
import type { Tab, TabsState } from './tabModel'

const at = (name: TabLocation['name']): TabLocation => ({ name })
const three = (): TabsState => ({
  tabs: [
    { id: 'a', location: at(RouteName.Goals) },
    { id: 'b', location: at(RouteName.Unlock) },
    { id: 'c', location: at(RouteName.Plan) },
  ],
  activeId: 'b',
})
const ids = (s: TabsState) => s.tabs.map((t) => t.id)
const fresh = (): Tab => ({ id: 'new', location: at(RouteName.Goals) })

describe('tabModel', () => {
  it('starts with one active tab', () => {
    expect(firstState('a', at(RouteName.Goals))).toEqual({
      tabs: [{ id: 'a', location: at(RouteName.Goals) }],
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
      RouteName.Goals,
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
