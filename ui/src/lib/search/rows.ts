import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import type { SearchHit } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
import {
  RouteName,
  WikiCategory,
  routeTitle,
  wikiCategoryTitle,
} from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'

type Message = MessageKey<MessageSchema>

// A result is not a row: it is the **destinations** it can open. "Brimstone" is a wiki page
// and a Collection row, and the two are different places (DESIGN-BRIEF.md §4.2).
export const RowGroup = {
  Screens: 'screens',
  Wiki: 'wiki',
  Unlock: 'unlock',
  Collection: 'collection',
} as const
export type RowGroup = (typeof RowGroup)[keyof typeof RowGroup]

export const rowGroupOrder: RowGroup[] = [
  RowGroup.Screens,
  RowGroup.Wiki,
  RowGroup.Unlock,
  RowGroup.Collection,
]

export interface ScreenEntry {
  key: string
  label: Message
  /** The label as the user reads it: what the query is matched against. */
  text: string
  location: TabLocation
}

// Every row carries where it goes, whichever kind it is: opening one is the same gesture
// everywhere, and only what it *draws* differs.
export type SearchRow =
  | {
      kind: 'screen'
      key: string
      group: RowGroup
      entry: ScreenEntry
      location: TabLocation
    }
  | {
      kind: 'hit'
      key: string
      group: RowGroup
      hit: SearchHit
      location: TabLocation
    }

// Every screen and wiki category, named as the user sees them: the frontend answers these
// itself, because the backend knows nothing about the app's own pages.
export const screenEntries = (t: (m: Message) => string): ScreenEntry[] => [
  ...Object.values(RouteName)
    .filter((name) => name !== RouteName.Search)
    .map((name) => ({
      key: `route-${name}`,
      label: routeTitle[name],
      text: t(routeTitle[name]),
      location: { name },
    })),
  ...Object.values(WikiCategory).map((category) => ({
    key: `wiki-${category}`,
    label: wikiCategoryTitle[category],
    text: t(wikiCategoryTitle[category]),
    location: { name: RouteName.Wiki, query: { category } },
  })),
]

export const matchingScreens = (
  entries: ScreenEntry[],
  query: string,
): ScreenEntry[] => {
  const wanted = query.trim().toLowerCase()
  return wanted === ''
    ? []
    : entries.filter((e) => e.text.toLowerCase().includes(wanted))
}

export interface RowOptions {
  /** Without the game there are no names for Unlock and the Collection to filter by. */
  catalog: boolean
  /** How many rows a group shows: five in the palette, all of them on the screen. */
  cap: number | null
}

const destinations = (hit: SearchHit, catalog: boolean): SearchRow[] => {
  const rows: SearchRow[] = []
  const page = pageLocation(hit.target)
  if (hit.hasPage && page)
    rows.push({
      kind: 'hit',
      key: `wiki-${hit.title}`,
      group: RowGroup.Wiki,
      hit,
      location: page,
    })
  if (!catalog) return rows
  if (hit.target.kind === 'achievement')
    rows.push({
      kind: 'hit',
      key: `unlock-${hit.title}`,
      group: RowGroup.Unlock,
      hit,
      location: { name: RouteName.Unlock, query: { q: hit.title } },
    })
  if (hit.target.kind === 'item')
    rows.push({
      kind: 'hit',
      key: `collection-${hit.title}`,
      group: RowGroup.Collection,
      hit,
      location: { name: RouteName.Collection, query: { q: hit.title } },
    })
  return rows
}

export const searchRows = (
  hits: SearchHit[],
  screens: ScreenEntry[],
  { catalog, cap }: RowOptions,
): SearchRow[] => {
  const rows: SearchRow[] = [
    ...screens.map((entry) => ({
      kind: 'screen' as const,
      key: entry.key,
      group: RowGroup.Screens,
      entry,
      location: entry.location,
    })),
    ...hits.flatMap((hit) => destinations(hit, catalog)),
  ]
  // The backend's order is kept inside each group: the frontend never re-ranks.
  return rowGroupOrder.flatMap((group) => {
    const inGroup = rows.filter((r) => r.group === group)
    return cap === null ? inGroup : inGroup.slice(0, cap)
  })
}

// No pick is every group: an empty toggle group never means an empty screen.
export const filterGroups = (
  rows: SearchRow[],
  picked: RowGroup[],
): SearchRow[] =>
  picked.length === 0 ? rows : rows.filter((r) => picked.includes(r.group))

export const groupCounts = (rows: SearchRow[]): Record<RowGroup, number> =>
  Object.fromEntries(
    rowGroupOrder.map((group) => [
      group,
      rows.filter((r) => r.group === group).length,
    ]),
  ) as Record<RowGroup, number>
