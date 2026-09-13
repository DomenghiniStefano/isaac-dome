import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { defaultLocation, locationTitle } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'

type Message = MessageKey<MessageSchema>

export interface Tab {
  id: string
  location: TabLocation
}

export interface TabsState {
  tabs: Tab[]
  activeId: string
}

// The tab bar's rules, pure: the store only holds the result.
export const firstState = (id: string, location: TabLocation): TabsState => ({
  tabs: [{ id, location }],
  activeId: id,
})

const indexOf = (state: TabsState, id: string): number =>
  state.tabs.findIndex((tab) => tab.id === id)

export const openTab = (
  state: TabsState,
  id: string,
  location: TabLocation,
): TabsState => {
  const at = indexOf(state, state.activeId) + 1
  return {
    tabs: [
      ...state.tabs.slice(0, at),
      { id, location },
      ...state.tabs.slice(at),
    ],
    activeId: id,
  }
}

export const selectTab = (state: TabsState, id: string): TabsState =>
  indexOf(state, id) < 0 ? state : { ...state, activeId: id }

// Closing the active tab moves to its right neighbour, or the left one when it was last;
// the bar is never empty, so closing the only tab puts a fresh one in its place.
export const closeTab = (
  state: TabsState,
  id: string,
  fresh: () => Tab,
): TabsState => {
  const index = indexOf(state, id)
  if (index < 0) return state
  const tabs = state.tabs.filter((tab) => tab.id !== id)
  if (tabs.length === 0) {
    const tab = fresh()
    return { tabs: [tab], activeId: tab.id }
  }
  if (id !== state.activeId) return { tabs, activeId: state.activeId }
  const next = tabs[Math.min(index, tabs.length - 1)]
  return next ? { tabs, activeId: next.id } : state
}

// A tab arriving from another window lands at an index and takes the focus: you dropped it
// where you wanted to look at it. The index is clamped, never rejected — a drop a pixel past
// the last tab is a drop on the end, not a lost tab.
export const insertTab = (
  state: TabsState,
  at: number,
  tab: Tab,
): TabsState => {
  const index = Math.max(0, Math.min(at, state.tabs.length))
  const tabs = [...state.tabs]
  tabs.splice(index, 0, tab)
  return { tabs, activeId: tab.id }
}

// A tab lifted out of the strip and not yet anywhere: what it was, and where it sat. The bar it
// left **may be empty**, and that is the difference from every other rule here — the window
// stays open because the tab can still come back. What becomes of an empty window is decided
// when the drag ends (`stores/tabs.ts`), not here.
export interface Removed {
  seed: TabSeed
  index: number
  state: TabsState
}

export const removeTab = (state: TabsState, id: string): Removed | null => {
  const index = state.tabs.findIndex((t) => t.id === id)
  const tab = state.tabs[index]
  if (!tab) return null
  const tabs = state.tabs.filter((t) => t.id !== id)
  // The neighbour rule of `closeTab`, without its floor: no tabs means no active tab.
  const activeId =
    id === state.activeId
      ? (tabs[Math.min(index, tabs.length - 1)]?.id ?? '')
      : state.activeId
  return { seed: tabSeed(tab), index, state: { tabs, activeId } }
}

// A window holding one tab *is* that tab: taking it out would leave a bar with nothing in it.
// Kept for the gestures that must refuse it; **tearing off is no longer one of them** (owner,
// 2026-09-13): a tab can be dragged out of a window that holds only it.
export const canDetach = (state: TabsState): boolean => state.tabs.length > 1

export interface Detached {
  tab: Tab
  state: TabsState
}

export const detachTab = (state: TabsState, id: string): Detached | null => {
  if (!canDetach(state)) return null
  const tab = state.tabs.find((t) => t.id === id)
  if (!tab) return null
  // As far as what is left behind is concerned, leaving is closing: the same neighbour rule.
  // `fresh` is never reached — `canDetach` has already refused the last tab.
  return { tab, state: closeTab(state, id, () => tab) }
}

// Everything a tab is except its identity: what crosses to another window, which mints ids of
// its own. Written by subtraction so that a tab gaining a field needs no line here.
export type TabSeed = Omit<Tab, 'id'>

// The id is **removed**, not merely left out of the type: `seedState` spreads the seed over a
// fresh id, and a seed still carrying the old one would overwrite it — two windows holding the
// same tab id, with nothing to say so.
export const tabSeed = (tab: Tab): TabSeed => {
  const seed = { ...tab }
  delete (seed as { id?: string }).id
  return seed
}

// The state a window born from a tear-off starts in. An empty seed would leave a bar with no
// tabs, so it becomes one default tab: a window showing the landing page beats a window
// showing nothing.
export const seedState = (
  seeds: TabSeed[],
  activeIndex: number,
  id: (n: number) => string,
): TabsState => {
  if (seeds.length === 0) return firstState(id(0), defaultLocation)
  const tabs = seeds.map((seed, n) => ({ id: id(n), ...seed }))
  const active = tabs[Math.max(0, Math.min(activeIndex, tabs.length - 1))]
  return { tabs, activeId: active?.id ?? '' }
}

// `to` is the tab's final index, as TabStrip computes it with moveIndex.
export const moveTab = (
  state: TabsState,
  from: number,
  to: number,
): TabsState => {
  const tabs = [...state.tabs]
  const [tab] = tabs.splice(from, 1)
  if (!tab) return state
  tabs.splice(to, 0, tab)
  return { ...state, tabs }
}

export const navigateTab = (
  state: TabsState,
  location: TabLocation,
): TabsState => ({
  ...state,
  tabs: state.tabs.map((tab) =>
    tab.id === state.activeId ? { ...tab, location } : tab,
  ),
})

// A tab's label. A page's title is data (English, from the index), every other label a
// message: the caller translates one and shows the other. Until the index knows the page,
// the tab reads as its category.
export const tabLabel = (
  location: TabLocation,
  titleOf: (key: string) => string | null,
): Message | { text: string } => {
  const key = location.query?.page
  const title = key === undefined ? null : titleOf(key)
  return title === null ? locationTitle(location) : { text: title }
}
