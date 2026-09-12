import type { CommandFilterState } from './context'

type Filtered = CommandFilterState['filtered']

// The scoring, out of the component so it can be stated in a test. `filter: false` is the one
// change B12 item 4 asks for that the palette can't do without: its rows are already the
// backend's answer, so every mounted item shows and every group stays visible — scoring them
// again would hide a row whose text doesn't repeat the query, like a section's fragment.
export const scoreItems = (
  items: Map<string, string>,
  groups: Map<string, Set<string>>,
  search: string,
  filter: boolean,
  contains: (text: string, query: string) => boolean,
): Filtered => {
  if (!filter || !search)
    return {
      count: items.size,
      items: new Map(),
      groups: new Set(groups.keys()),
    }
  const scores = new Map(
    [...items].map(([id, text]): [string, number] => [
      id,
      contains(text, search) ? 1 : 0,
    ]),
  )
  return {
    count: [...scores.values()].filter((score) => score > 0).length,
    items: scores,
    groups: new Set(
      [...groups]
        .filter(([, ids]) => [...ids].some((id) => (scores.get(id) ?? 0) > 0))
        .map(([id]) => id),
    ),
  }
}
