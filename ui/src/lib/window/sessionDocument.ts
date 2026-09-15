import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import type { Entry, Session, TabSeed } from '@/stores/tabModel'

// The document's version. It is bumped when an older app could read the new shape and be wrong
// about it — never for a part it can simply ignore, which is why 3.7's sidebar width and table
// sizes will join as named keys without touching this number.
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

// A stored history entry. The reading it carries is `unknown` here on purpose — what it means
// is the screen's, and the screen validates it (`lib/tabs/tabView.ts`).
const readEntry = (value: unknown): Entry | null => {
  if (typeof value !== 'object' || value === null) return null
  const { location, view } = value as { location?: unknown; view?: unknown }
  const read = readLocation(location)
  return read === null
    ? null
    : view === undefined
      ? { location: read }
      : { location: read, view }
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

export const writeSession = (session: Session): string =>
  JSON.stringify({
    version: Version,
    tabs: session.tabs,
    activeIndex: session.activeIndex,
  })
