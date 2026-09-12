import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { locationTitle } from '@/router/routeTable'
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
