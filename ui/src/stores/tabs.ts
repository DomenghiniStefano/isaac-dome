import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import type { Point } from '@/lib/drag/dragList'
import { oweSeed } from '@/lib/window/seeds'
import { newWindowLabel, windowPort } from '@/lib/window/windowPort'
import { defaultLocation } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import type { IncomingHover } from '@/components/shell/tabs'
import { WindowMessageKind } from '@/lib/window/messages'
import {
  closeTab,
  firstState,
  insertTab,
  moveTab,
  navigateTab,
  openTab,
  removeTab,
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

  // A tab that has left the strip and has not landed yet. It leaves the moment the drag tears
  // it off, not at the release (owner, 2026-09-13): what you are dragging is no longer in the
  // bar, which is what a browser does and what the owner asked for. Until the drag ends it
  // belongs to nobody, and the window it left stays open — even empty — because it can still
  // come back.
  const inFlight = ref<{ seed: TabSeed; index: number } | null>(null)

  const liftOut = (id: string): boolean => {
    const out = removeTab(state.value, id)
    if (!out) return false
    inFlight.value = { seed: out.seed, index: out.index }
    state.value = out.state
    return true
  }

  // The drag was called off, or the pointer was lost: the tab goes back where it sat.
  const putBack = (): void => {
    const flight = inFlight.value
    if (!flight) return
    inFlight.value = null
    state.value = insertTab(state.value, flight.index, {
      id: nextId(),
      ...flight.seed,
    })
  }

  // A window with nothing left in it. The first window keeps its landing tab, exactly as
  // closing its last tab already does; any other has nothing left to be, and closes.
  const closeIfEmpty = async (): Promise<void> => {
    if (state.value.tabs.length > 0) return
    if (!windowPort.isMain()) {
      await windowPort.closeSelf()
      return
    }
    const tab = fresh()
    state.value = { tabs: [tab], activeId: tab.id }
  }

  // The tab landed on a strip — possibly this window's own, which is a landing like any other
  // now that the tab has already left it.
  const settleTo = async (target: string, at: Point): Promise<void> => {
    const flight = inFlight.value
    if (!flight) return
    inFlight.value = null
    await windowPort.send(target, {
      kind: WindowMessageKind.Docked,
      tab: flight.seed,
      at,
    })
    if (target !== windowPort.label()) await windowPort.focus(target)
    await closeIfEmpty()
  }

  // The tab landed on the bare desktop: a window of its own, there.
  const settleInNewWindow = async (at: Point, size: Point): Promise<void> => {
    const flight = inFlight.value
    if (!flight) return
    inFlight.value = null
    await openWindowWith([flight.seed], at, size)
    await closeIfEmpty()
  }

  // A tab from another window, hovering over this strip. The point arrives in desktop pixels
  // because the sender cannot know our scale factor; the geometry to convert it is ours, read
  // once per hover and not per frame — a window does not move while a tab is over it.
  const incoming = ref<IncomingHover | null>(null)
  // Where the strip says it would land. The strip owns the rectangles, so it owns the answer,
  // and the gap the marker is drawn in is the gap the tab is docked into: one computation.
  const aimed = ref<number | null>(null)

  const aimIncoming = async (at: Point): Promise<void> => {
    const window = incoming.value?.window ?? (await windowPort.self())
    incoming.value = { at, window }
  }

  const clearIncoming = (): void => {
    incoming.value = null
    aimed.value = null
  }

  const aim = (index: number | null): void => {
    aimed.value = index
  }

  // A tab arriving from another window. It lands where the marker said — the strip aimed it
  // while the tab hovered — and at the end of the strip when it was never aimed, which is a
  // drop from a window that never passed over this one.
  const dock = (seed: TabSeed): void => {
    const at = aimed.value ?? state.value.tabs.length
    state.value = insertTab(state.value, at, { id: nextId(), ...seed })
    clearIncoming()
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
    incoming,
    aimIncoming,
    clearIncoming,
    aim,
    dock,
    liftOut,
    putBack,
    settleTo,
    settleInNewWindow,
    openWindowWith,
  }
})
