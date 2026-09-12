import { describe, expect, it } from 'vitest'
import type { SearchHit } from '@/lib/ipc/types'
import { RouteName } from '@/router/routeTable'
import { RowGroup, filterGroups, groupCounts, searchRows } from './rows'
import type { ScreenEntry } from './rows'

const hit = (over: Partial<SearchHit>): SearchHit => ({
  target: { kind: 'item', id: 105 },
  title: 'The D6',
  iconUrl: null,
  hasPage: true,
  match: { kind: 'title' },
  progress: 'unknown',
  ...over,
})

const screens: ScreenEntry[] = [
  {
    key: 'route-collection',
    label: 'routes.collection',
    text: 'Collezione',
    location: { name: RouteName.Collection },
  },
]

describe('searchRows', () => {
  it('turns one hit into every destination it can open', () => {
    const rows = searchRows([hit({})], [], { catalog: true, cap: null })
    expect(rows.map((r) => r.group)).toEqual([
      RowGroup.Wiki,
      RowGroup.Collection,
    ])
    // The Wiki row is the page itself, with its category so the sidebar stays lit.
    expect(rows[0]?.location).toEqual({
      name: RouteName.Wiki,
      query: { category: 'items', page: 'item:105' },
    })
    // The Collection row opens the list already filtered on the item's name (B3).
    const collection = rows.find((r) => r.group === RowGroup.Collection)
    expect(collection?.location).toEqual({
      name: RouteName.Collection,
      query: { q: 'The D6' },
    })
  })

  it('gives an achievement an Unlock row, and a page-less hit no Wiki row', () => {
    const rows = searchRows(
      [
        hit({
          target: { kind: 'achievement', id: 3 },
          title: 'You unlocked X',
          hasPage: false,
        }),
      ],
      [],
      { catalog: true, cap: null },
    )
    expect(rows.map((r) => r.group)).toEqual([RowGroup.Unlock])
  })

  it('drops the Unlock and Collection rows without a catalog', () => {
    // Those two lists have no names to filter by when the game isn't installed.
    const rows = searchRows([hit({})], [], { catalog: false, cap: null })
    expect(rows.map((r) => r.group)).toEqual([RowGroup.Wiki])
  })

  it('lists the screens first and caps each group in the palette', () => {
    const many = [1, 2, 3, 4, 5, 6].map((id) =>
      hit({ target: { kind: 'item', id }, title: `Bomb ${id}` }),
    )
    const rows = searchRows(many, screens, { catalog: true, cap: 5 })
    expect(rows[0]?.group).toBe(RowGroup.Screens)
    const counts = groupCounts(rows)
    expect(counts[RowGroup.Wiki]).toBe(5)
    expect(counts[RowGroup.Collection]).toBe(5)
    // Uncapped, the screen shows all six of each.
    const all = groupCounts(
      searchRows(many, screens, { catalog: true, cap: null }),
    )
    expect(all[RowGroup.Wiki]).toBe(6)
  })

  it('shows every group until one is picked', () => {
    const rows = searchRows([hit({})], screens, { catalog: true, cap: null })
    // No pick is "all of them": an empty toggle group never means an empty screen.
    expect(filterGroups(rows, [])).toHaveLength(rows.length)
    expect(filterGroups(rows, [RowGroup.Wiki]).map((r) => r.group)).toEqual([
      RowGroup.Wiki,
    ])
  })
})
