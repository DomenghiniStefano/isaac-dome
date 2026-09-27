import { ref } from 'vue'
import type { WikiCategory } from '@/router/routeTable'

// Card grid or table, per category. Remembered across the window's own
// tabs and — through the session document (`lib/window/sessionDocument.ts`'s `wikiListView`,
// a named key beside `windows` the same way `sidebarWidth` is one) — across a restart. A
// per-viewer convenience: it never crosses to another machine and nothing here talks to Rust.
export const ListViewMode = { Grid: 'grid', Table: 'table' } as const
export type ListViewMode = (typeof ListViewMode)[keyof typeof ListViewMode]

const DefaultMode: ListViewMode = ListViewMode.Grid

// Keyed by `WikiCategory`, not by tab: opening the same category in a second tab finds it as
// the reader left it, which a tab-scoped reading (`screens/wiki/tabView.ts`) cannot do — two
// tabs on Items are the same list read twice, not two lists.
export const wikiListView = ref<Record<string, ListViewMode>>({})

const isListViewMode = (value: unknown): value is ListViewMode =>
  value === ListViewMode.Grid || value === ListViewMode.Table

export const listViewFor = (category: WikiCategory): ListViewMode =>
  wikiListView.value[category] ?? DefaultMode

export const setListViewFor = (
  category: WikiCategory,
  mode: ListViewMode,
): void => {
  wikiListView.value = { ...wikiListView.value, [category]: mode }
}

// What the stored document can still make sense of: a category this build still has, holding
// one of the two values that exist today — dropped one entry at a time, the same degrading
// `readSession` already does for the rest of the document, never the whole map for one bad
// entry.
export const readWikiListView = (
  value: unknown,
  categories: readonly WikiCategory[],
): Record<string, ListViewMode> => {
  if (typeof value !== 'object' || value === null || Array.isArray(value))
    return {}
  const source = value as Record<string, unknown>
  return Object.fromEntries(
    categories.flatMap((category) => {
      const stored = source[category]
      return isListViewMode(stored) ? [[category, stored]] : []
    }),
  ) as Record<string, ListViewMode>
}

// Read once, at restore: replaces whatever this window held rather than merging into it, the
// same way `layoutEcho.take` replaces the sidebar's layout.
export const takeWikiListView = (
  value: unknown,
  categories: readonly WikiCategory[],
): void => {
  wikiListView.value = readWikiListView(value, categories)
}

export const currentWikiListView = (): Record<string, ListViewMode> =>
  wikiListView.value
