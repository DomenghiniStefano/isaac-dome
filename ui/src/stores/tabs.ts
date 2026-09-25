import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import type { Point } from '@/lib/drag/dragList'
import { oweSeed } from '@/lib/window/seeds'
import { newWindowLabel, windowPort } from '@/lib/window/windowPort'
import type { Target } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
import { defaultLocation } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import type { IncomingHover } from '@/lib/shell/tabs'
import { WindowMessageKind } from '@/lib/window/messages'
import {
  backTab,
  canGoBack,
  canGoForward,
  closeTab,
  forwardTab,
  insertTab,
  moveTab,
  navigateTab,
  openTab,
  refineTab,
  removeTab,
  seedState,
  setEntryScroll,
  setEntryView,
  sessionOf,
  selectTab,
  tabLocation,
  tabSeed,
} from './tabModel'
import type { EntryAddress, Tab, TabSeed, TabsState } from './tabModel'

// The open tabs, window-wide. The rules are tabModel's; this holds the result. What a window
// holds is saved and restored by `lib/window/session.ts` (part of 3.7, landed with the tray).
export const useTabsStore = defineStore(StoreId.Tabs, () => {
  let counter = 0
  const nextId = (): string => `tab-${++counter}`
  const fresh = (): Tab => ({
    id: nextId(),
    entries: [{ location: defaultLocation }],
    index: 0,
  })

  // **Every window starts empty and waits to be told what it holds** (`lib/window/session.ts`).
  // A window born from a tear-off is told by the window that created it; `main` is told by its
  // own last session. Neither ever travels in a URL.
  //
  // `main` used to start on its landing tab instead. It doesn't any more because the landing
  // tab would then be painted and replaced a moment later by the session — a tab appearing and
  // vanishing, which is worse than a bar that is empty for the length of one read.
  const empty: TabsState = { tabs: [], activeId: '' }
  const state = ref<TabsState>(empty)
  const pending = ref(true)

  const tabs = computed(() => state.value.tabs)
  const activeId = computed(() => state.value.activeId)
  const active = computed(() =>
    state.value.tabs.find((tab) => tab.id === state.value.activeId),
  )
  // Where the active tab is: the entry its history is showing, not the last one it reached.
  const location = computed(() =>
    active.value ? tabLocation(active.value) : undefined,
  )
  const canBack = computed(() => canGoBack(active.value))
  const canForward = computed(() => canGoForward(active.value))

  const open = (location: TabLocation = defaultLocation): void => {
    state.value = openTab(state.value, nextId(), location)
  }
  const select = (id: string): void => {
    state.value = selectTab(state.value, id)
  }
  // Closing the last tab of a secondary window closes the window: that window *is* its tabs,
  // and an empty one has nothing to be (owner, 2026-09-13). The first window keeps its landing
  // tab instead — the app is still running, and its bar is never empty.
  const close = async (id: string): Promise<void> => {
    if (state.value.tabs.length === 1 && !windowPort.isMain()) {
      await windowPort.closeSelf()
      return
    }
    state.value = closeTab(state.value, id, fresh)
  }
  const move = (from: number, to: number): void => {
    state.value = moveTab(state.value, from, to)
  }
  const navigate = (location: TabLocation): void => {
    state.value = navigateTab(state.value, location)
  }
  // The app's one gesture: a click navigates the active tab, Ctrl opens the page beside it.
  // Every link, row and entry that goes somewhere goes through here, with the modifier it read.
  // No location is a reference with nowhere to go, and it moves nothing.
  const go = (location: TabLocation | null, newTab: boolean): void => {
    if (location === null) return
    if (newTab) open(location)
    else navigate(location)
  }
  // The same gesture on a wiki reference: its page, here or beside (DESIGN-BRIEF.md §4.2).
  const openPage = (target: Target, newTab: boolean): void => {
    go(pageLocation(target), newTab)
  }
  // How the active tab's current entry is being read. The rule is `tabModel`'s; this only holds
  // the result, as with every other tab rule.
  const setView = (location: TabLocation, view: unknown): void => {
    state.value = setEntryView(state.value, location, view)
  }
  // Where a region of an entry's screen was scrolled to. Addressed by tab and entry rather than
  // aimed at the active one: the rule is `tabModel`'s, and says why.
  const setScroll = (at: EntryAddress, region: string, top: number): void => {
    state.value = setEntryScroll(
      state.value,
      at.tabId,
      at.index,
      at.location,
      region,
      top,
    )
  }
  const refine = (location: TabLocation): void => {
    state.value = refineTab(state.value, location)
  }
  const back = (): void => {
    state.value = backTab(state.value)
  }
  const forward = (): void => {
    state.value = forwardTab(state.value)
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
    const paid = oweSeed(label, seeds, seeds.length - 1)
    await windowPort.create(label, at, size)
    // **The debt is waited for, not just registered.** It lives in this window's memory, and
    // this window may be about to close — it just gave away its last tab. Closing first leaves
    // the newborn asking a window that no longer exists, and it opens with an empty bar: seen
    // on the machine, 2026-09-13. The wait has its own timeout, so a newborn that never asks
    // cannot keep this window alive.
    await paid
  }

  // What this window would be restored from. The rule is `tabModel`'s, as every other rule
  // about tabs is; this only reads it.
  const session = computed(() => sessionOf(state.value))

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

  // This window's geometry for the hover: the one already read, kept without an `await` so a
  // `Hovering` followed by a `HoverLeft` in the same macrotask never finds `incoming` stale from
  // a microtask it did not need — or read now, on its first point.
  const aimIncoming = async (at: Point): Promise<void> => {
    const known = incoming.value?.window
    if (known) {
      incoming.value = { at, window: known }
      return
    }
    const geometry = await windowPort.self()
    incoming.value = { at, window: geometry }
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
    location,
    canBack,
    canForward,
    open,
    select,
    close,
    move,
    navigate,
    go,
    openPage,
    seed,
    seedAt,
    session,
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
    refine,
    setView,
    setScroll,
    back,
    forward,
  }
})
