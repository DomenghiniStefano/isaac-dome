import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import type { Entry, Session, TabSeed } from '@/stores/tabModel'

// The document's version. It is bumped when an older app could read the new shape and be wrong
// about it — never for a part it can simply ignore. An entry gaining a `view` is such a part, so
// 3.7a does not move it, and neither will 3.7c's sidebar width and table sizes. **3.7b does**:
// the top level stops saying `tabs` and starts saying `windows`, which an older app would read
// as a session it cannot use.
const Version = 1

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

// What was stored, as far as it can be read. `null` means "nothing usable", which the caller
// turns into the landing tab. A single unreadable tab is dropped **alone** — eight tabs do not
// vanish because one screen was renamed — and the active index follows what is left.
export const readSession = (raw: string | null): Session | null => {
  if (raw === null) return null
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return null
  }
  if (typeof parsed !== 'object' || parsed === null) return null
  const { version, tabs, activeIndex } = parsed as {
    version?: unknown
    tabs?: unknown
    activeIndex?: unknown
  }
  if (version !== Version || !Array.isArray(tabs)) return null
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
  return { tabs: kept, activeIndex: kept.length === 0 ? 0 : active }
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

export const writeSession = (session: Session): string =>
  JSON.stringify({
    version: Version,
    tabs: session.tabs.map(stored),
    activeIndex: session.activeIndex,
  })
