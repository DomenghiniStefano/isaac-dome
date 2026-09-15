import { emptyFilter } from '@/lib/facets/faceting'
import type { FacetFilter } from '@/lib/facets/faceting'

// What a screen declares so the shell can hand it back a reading it wrote before — before a
// tear-off, before the app was last closed, or before it was updated. `read` answering `null` is
// not an error: it is this app meeting a record written by a version of itself that no longer
// exists, and the answer to that is the empty reading, never a crash and never a filter nobody
// set.
//
// There is no key. A record lives on a history entry and an entry has exactly one screen, so a
// key would name what the entry already names.
export interface TabViewSpec<T> {
  empty: () => T
  read: (value: unknown) => T | null
}

export const readString = (
  value: unknown,
  allowed: readonly string[],
): string | null =>
  typeof value === 'string' && allowed.includes(value) ? value : null

// `allowed` left out means "any string": a page key or a search query is not drawn from a set
// this side knows. Given one, a value outside it is dropped rather than kept — a pick nobody can
// see filters everything away and reads as an empty screen.
export const readStringArray = (
  value: unknown,
  allowed?: readonly string[],
): string[] | null => {
  if (!Array.isArray(value)) return null
  if (!value.every((each) => typeof each === 'string')) return null
  const strings = value as string[]
  return allowed === undefined
    ? strings
    : strings.filter((each) => allowed.includes(each))
}

// The facets are what `order` says they are. A stored `picks` is read facet by facet through it,
// so a facet since added arrives empty and one that has gone is not carried into a `Record` the
// engine would index by a key that no longer exists.
export const readFacetFilter = <F extends string>(
  value: unknown,
  order: readonly F[],
): FacetFilter<F> | null => {
  if (typeof value !== 'object' || value === null) return null
  const { query, picks } = value as { query?: unknown; picks?: unknown }
  if (typeof query !== 'string') return null
  if (typeof picks !== 'object' || picks === null)
    return { ...emptyFilter<F>([...order]), query }
  const source = picks as Record<string, unknown>
  return {
    query,
    picks: Object.fromEntries(
      order.map((facet) => [facet, readStringArray(source[facet]) ?? []]),
    ) as Record<F, string[]>,
  }
}
