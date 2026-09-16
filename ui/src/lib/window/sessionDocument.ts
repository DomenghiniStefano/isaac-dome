import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import type { Entry, TabSeed } from '@/stores/tabModel'

// The document's version. It is bumped when an older app could read the new shape and be wrong
// about it — never for a part it can simply ignore. An entry gaining a `view` is such a part, so
// 3.7a did not move it, and neither will 3.7c's sidebar width and table sizes. **3.7b did**: the
// top level stopped saying `tabs` and started saying `windows`, and an older app reading a new
// document finds no `tabs` and answers the landing page — which is exactly the "could read it
// and be wrong" a version number exists for.
const Version = 2

// **Version 1 is still read**, as one window with no box. The alternative is that everybody who
// updates the app loses the tabs they had open, on a day when the app has no way to tell them
// why.
const FirstVersion = 1

// Where a window was, in **desktop physical pixels** — the units `WindowBox` reports and the ones
// `windowPort.create` takes for a position. The size it takes is logical, so whoever restores
// divides by the scale factor of the monitor the window lands on: getting that backwards opens a
// window twice the size it had on a scaled screen.
export interface StoredBox {
  left: number
  top: number
  width: number
  height: number
}

// One window of the session. `box` is optional because a version 1 document has none, and because
// a window whose geometry could not be read still has tabs worth restoring.
export interface StoredWindow {
  tabs: TabSeed[]
  activeIndex: number
  box?: StoredBox
}

// The whole document. `sidebarWidth` is a **named key beside `windows`**, not window state and
// not tab state: the sidebar is one width for the app, and this is what migration 3's comment
// promised when it made the session an object with a version rather than a bare array of tabs —
// *"the sidebar width and per-table sizes join as named parts of the same document, and a named
// part costs no migration"*. It costs no version bump either: an older app ignores a key it does
// not know and is wrong about nothing, which is the only thing the version is for.
//
// **`tables` is not here yet**, although §8 of the spec names it, because nothing in the app
// produces a table size: B27's resizable tables are not built, and a named place for a value that
// does not exist is one more thing to read and nothing to store.
export interface StoredSession {
  windows: StoredWindow[]
  sidebarWidth?: number
}

const routeNames: readonly string[] = Object.values(RouteName)

const isRouteName = (value: unknown): value is RouteName =>
  typeof value === 'string' && routeNames.includes(value)

// A location we can still open. The query is carried as it was written: a filter or a page that
// no longer resolves is the screen's business, and every screen already says so (B6). What is
// checked here is the one thing that decides whether the tab can exist at all.
const readLocation = (value: unknown): TabLocation | null => {
  if (typeof value !== 'object' || value === null) return null
  const { name, query } = value as { name?: unknown; query?: unknown }
  if (!isRouteName(name)) return null
  return query === undefined || query === null
    ? { name }
    : { name, query: query as TabLocation['query'] }
}

// A stored view, as far as it can be trusted here: an object, and nothing more. What it means
// is the screen's, and the screen validates it (`lib/tabs/tabView.ts`). Anything else — a
// string, a number, an array — is dropped, and **the entry survives without it**: a tab that
// opens saying what it has beats a tab that vanishes (B6).
const readView = (value: unknown): unknown =>
  typeof value === 'object' && value !== null && !Array.isArray(value)
    ? value
    : undefined

// Two shapes, one reader. Before 3.7a an entry *was* a location, and a document written by that
// version must still open: losing somebody's tabs on an update is not a thing the app can
// explain to them afterwards.
const readEntry = (value: unknown): Entry | null => {
  if (typeof value !== 'object' || value === null) return null
  const { location, view } = value as { location?: unknown; view?: unknown }
  if (location === undefined) {
    const bare = readLocation(value)
    return bare === null ? null : { location: bare }
  }
  const read = readLocation(location)
  if (read === null) return null
  const kept = readView(view)
  return kept === undefined
    ? { location: read }
    : { location: read, view: kept }
}

const readTab = (value: unknown): TabSeed | null => {
  if (typeof value !== 'object' || value === null) return null
  const { entries, index } = value as { entries?: unknown; index?: unknown }
  if (!Array.isArray(entries) || entries.length === 0) return null
  if (typeof index !== 'number' || index < 0 || index >= entries.length)
    return null
  const read = entries.map(readEntry)
  // A tab is its history: one entry we cannot open and the back button lies. All or nothing.
  if (read.some((entry) => entry === null)) return null
  return { entries: read as Entry[], index }
}

// A number we can place a window by. `NaN` and `Infinity` are numbers to `typeof` and are not
// coordinates to anybody else.
const isFinite = (value: unknown): value is number =>
  typeof value === 'number' && Number.isFinite(value)

// Half a box is not a position: a window placed at a left with no top is a window somewhere
// nobody asked for, so the four travel together or not at all.
const readBox = (value: unknown): StoredBox | undefined => {
  if (typeof value !== 'object' || value === null) return undefined
  const { left, top, width, height } = value as Record<string, unknown>
  if (![left, top, width, height].every(isFinite)) return undefined
  return {
    left: left as number,
    top: top as number,
    width: width as number,
    height: height as number,
  }
}

// One window's tabs. A single unreadable tab is dropped **alone** — eight tabs do not vanish
// because one screen was renamed — and the active index follows what is left. `null` is a window
// with nothing in it, which is a window the user never had.
const readTabs = (tabs: unknown, activeIndex: unknown): StoredWindow | null => {
  if (!Array.isArray(tabs)) return null
  const wanted = typeof activeIndex === 'number' ? activeIndex : 0
  const kept: TabSeed[] = []
  let active = 0
  tabs.forEach((value, at) => {
    const tab = readTab(value)
    if (!tab) return
    // The active tab is the last kept one at or before where it was: if the tab that was
    // active is the one that dropped, the selection lands on its neighbour rather than on the
    // first tab.
    if (at <= wanted) active = kept.length
    kept.push(tab)
  })
  return kept.length === 0 ? null : { tabs: kept, activeIndex: active }
}

const readWindow = (value: unknown): StoredWindow | null => {
  if (typeof value !== 'object' || value === null) return null
  const { tabs, activeIndex, box } = value as Record<string, unknown>
  const read = readTabs(tabs, activeIndex)
  if (read === null) return null
  const where = readBox(box)
  return where === undefined ? read : { ...read, box: where }
}

// What was stored, as far as it can be read: the windows, in the order they were written, `main`
// first. `null` means "nothing usable", which the caller turns into one window on its landing
// tab. A window whose every tab dropped is dropped **whole** rather than restored empty: an empty
// window is one the user never had, and opening it would put a landing page on their desktop
// they did not leave there.
export const readSession = (raw: string | null): StoredSession | null => {
  if (raw === null) return null
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return null
  }
  if (typeof parsed !== 'object' || parsed === null) return null
  const { version, tabs, activeIndex, windows, sidebarWidth } =
    parsed as Record<string, unknown>
  // Version 1 said `tabs` at the top level and knew nothing about windows. It is one window,
  // wherever the window manager decides to put it — and it never carried a sidebar width.
  if (version === FirstVersion) {
    const one = readTabs(tabs, activeIndex)
    return one === null ? null : { windows: [one] }
  }
  if (version !== Version || !Array.isArray(windows)) return null
  const kept = windows
    .map(readWindow)
    .filter((window): window is StoredWindow => window !== null)
  if (kept.length === 0) return null
  // A number, and nothing more: the bounds are the sidebar's own and are enforced where it is
  // drawn (`clampSidebarWidth`). A parser that knew 168 and 420 would be a parser holding the
  // design's pixels.
  return isFinite(sidebarWidth)
    ? { windows: kept, sidebarWidth }
    : { windows: kept }
}

// Only the entry each tab is showing keeps its view. `MAX_SESSION_BYTES` is 64 KiB and its
// budget was written for "fifty tabs of route names and queries"; a record on every one of a
// tab's fifty entries is a different sum. Bounding it by construction beats raising a number,
// and what it costs is that going back in a *restored* tab gets the screen's empty state — the
// same thing that happened to every restored tab before 3.7a.
const stored = (tab: TabSeed): TabSeed => ({
  ...tab,
  entries: tab.entries.map((entry, at) =>
    at === tab.index ? entry : { location: entry.location },
  ),
})

export const writeSession = (session: StoredSession): string =>
  JSON.stringify({
    version: Version,
    windows: session.windows.map((window) => ({
      tabs: window.tabs.map(stored),
      activeIndex: window.activeIndex,
      ...(window.box ? { box: window.box } : {}),
    })),
    // Absent rather than `null` when nobody ever sized the sidebar: a key that is there and means
    // nothing is a key every reader has to ask about.
    ...(session.sidebarWidth === undefined
      ? {}
      : { sidebarWidth: session.sidebarWidth }),
  })
