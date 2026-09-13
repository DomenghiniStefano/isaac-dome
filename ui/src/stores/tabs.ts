import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import type { Point } from '@/lib/drag/dragList'
import { oweSeed } from '@/lib/window/seeds'
import { newWindowLabel, windowPort } from '@/lib/window/windowPort'
import { defaultLocation } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import { WindowMessageKind } from '@/lib/window/messages'
import {
  canDetach,
  closeTab,
  detachTab,
  firstState,
  insertTab,
  moveTab,
  navigateTab,
  openTab,
  seedState,
  selectTab,
  tabSeed,
} from './tabModel'
import type { Tab, TabSeed, TabsState } from './tabModel'

// The open tabs, window-wide. The rules are tabModel's; this holds the result. Nothing is
// saved yet: tabs surviving a restart is sub-project 7.
export const useTabsStore = defineStore(StoreId.Tabs, () => {
  let counter = 0
  const nextId = (): string => `tab-${++counter}`
  const fresh = (): Tab => ({ id: nextId(), location: defaultLocation })

  // `main` starts with its landing tab, as it always has. A window born from a tear-off starts
  // empty and waits for its seed (`lib/window/session.ts`): what it holds is decided by the
  // window that created it and never travels in its URL.
  const born = windowPort.isMain()
  const empty: TabsState = { tabs: [], activeId: '' }
  const state = ref<TabsState>(
    born ? firstState(nextId(), defaultLocation) : empty,
  )
  const pending = ref(!born)

  const tabs = computed(() => state.value.tabs)
  const activeId = computed(() => state.value.activeId)
  const active = computed(() =>
    state.value.tabs.find((tab) => tab.id === state.value.activeId),
  )

  const open = (location: TabLocation = defaultLocation): void => {
    state.value = openTab(state.value, nextId(), location)
  }
  const select = (id: string): void => {
    state.value = selectTab(state.value, id)
  }
  const close = (id: string): void => {
    state.value = closeTab(state.value, id, fresh)
  }
  const move = (from: number, to: number): void => {
    state.value = moveTab(state.value, from, to)
  }
  const navigate = (location: TabLocation): void => {
    state.value = navigateTab(state.value, location)
  }

  // What this window was told to hold. Called once, by the session, and never again: the ids
  // are this window's, minted here as every other tab's are.
  const seed = (seeds: TabSeed[], activeIndex: number): void => {
    state.value = seedState(seeds, activeIndex, () => nextId())
    pending.value = false
  }

  // Opens a window holding these tabs, at this place and size, and owes it its seed before it
  // can ask for it.
  const openWindowWith = async (
    seeds: TabSeed[],
    at: Point,
    size: Point,
  ): Promise<void> => {
    const label = newWindowLabel()
    oweSeed(label, seeds, seeds.length - 1)
    await windowPort.create(label, at, size)
  }

  // The tab at that index, as it travels: everything but its identity.
  const seedAt = (index: number): TabSeed | null => {
    const tab = state.value.tabs[index]
    return tab ? tabSeed(tab) : null
  }

  // Whether a tab can leave this window at all. A window holding one tab already *is* that
  // tab: the gesture is refused before it starts.
  const canTear = computed(() => canDetach(state.value))

  // A tab arriving from another window, at an index in this strip.
  const dock = (seed: TabSeed, at: number): void => {
    state.value = insertTab(state.value, at, { id: nextId(), ...seed })
  }

  // A tab leaving for another window. **The last tab may leave this way**: joining a window
  // that exists is not the same gesture as opening one, and a window that gives its last tab
  // away has nothing left to be — so it closes, unless it is `main`, which keeps a fresh tab
  // the way closing the last one already does.
  //
  // The order is load-bearing: the target is told first, and only then does the tab leave
  // here. A target that never answers costs nothing — the tab is still in this window.
  const giveAway = async (
    id: string,
    target: string,
    at: Point,
  ): Promise<boolean> => {
    const tab = state.value.tabs.find((t) => t.id === id)
    if (!tab) return false
    await windowPort.send(target, {
      kind: WindowMessageKind.Docked,
      tab: tabSeed(tab),
      at,
    })
    const last = state.value.tabs.length === 1
    state.value =
      last && !windowPort.isMain() ? empty : closeTab(state.value, id, fresh)
    await windowPort.focus(target)
    if (last && !windowPort.isMain()) await windowPort.closeSelf()
    return true
  }

  // A tab leaving for a window of its own. Here the last tab is refused: that window already
  // *is* that tab alone, and tearing it off would close one window to open the same one. The
  // window is created before the tab leaves, so a creation that fails leaves the tab where it
  // was.
  const tearOffTo = async (
    id: string,
    at: Point,
    size: Point,
  ): Promise<boolean> => {
    const out = detachTab(state.value, id)
    if (!out) return false
    await openWindowWith([tabSeed(out.tab)], at, size)
    state.value = out.state
    return true
  }

  return {
    tabs,
    activeId,
    active,
    pending,
    open,
    select,
    close,
    move,
    navigate,
    seed,
    seedAt,
    canTear,
    dock,
    giveAway,
    tearOffTo,
    openWindowWith,
  }
})
