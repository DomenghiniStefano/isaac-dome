import { describe, expect, it } from 'vitest'
import type { RunItemRef, RunView } from '@/lib/ipc/types'
import { GroupTitleKind, RunItemView, groupItems } from './itemViews'

const item = (id: number): RunItemRef => ({ id, name: null, iconUrl: null })
const picked = (id: number, pool: string, floor: number | null) => ({
  item: item(id),
  pool,
  floor,
})

const run = {
  startingItems: [item(34)],
  collected: [
    picked(1, 'treasure', null),
    picked(2, 'shop', 0),
    picked(3, 'treasure', 1),
    picked(105, 'devil', 1),
  ],
  passives: [item(1), item(3)],
  familiars: [item(2)],
  heldActive: item(105),
  floorDetails: [
    { stage: 1, stageType: 0, name: 'Basement I', rooms: 12 },
    { stage: 2, stageType: 0, name: 'Basement II', rooms: 14 },
  ],
} as unknown as RunView

const ids = (items: RunItemRef[]) => items.map((i) => i.id)

describe('groupItems', () => {
  // Today's reading stays the default: starting, collected, the active held.
  it('reads as logged by default', () => {
    const groups = groupItems(run, RunItemView.AsLogged)
    expect(groups.map((g) => g.title)).toEqual([
      { kind: GroupTitleKind.Message, message: 'runs.startingItems' },
      { kind: GroupTitleKind.Message, message: 'runs.collected' },
      { kind: GroupTitleKind.Message, message: 'runs.heldActive' },
    ])
    expect(groups.map((g) => ids(g.items))).toEqual([
      [34],
      [1, 2, 3, 105],
      [105],
    ])
  })

  it('keeps an empty logged group, which the page says is empty', () => {
    const none = { ...run, collected: [] } as RunView
    expect(groupItems(none, RunItemView.AsLogged)[1].items).toEqual([])
  })

  it('reads by type: starting, passives, familiars, the active held', () => {
    expect(
      groupItems(run, RunItemView.ByType).map((g) => ids(g.items)),
    ).toEqual([[34], [1, 3], [2], [105]])
  })

  it('reads by origin, one group per pool in the order first met', () => {
    const groups = groupItems(run, RunItemView.ByOrigin)
    expect(groups.map((g) => g.title)).toEqual([
      { kind: GroupTitleKind.Pool, pool: 'treasure' },
      { kind: GroupTitleKind.Pool, pool: 'shop' },
      { kind: GroupTitleKind.Pool, pool: 'devil' },
    ])
    expect(groups.map((g) => ids(g.items))).toEqual([[1, 3], [2], [105]])
  })

  // An item taken before the first floor this read saw has no floor, and comes first.
  it('reads by floor, the items with no floor first', () => {
    const groups = groupItems(run, RunItemView.ByFloor)
    expect(groups.map((g) => g.title)).toEqual([
      { kind: GroupTitleKind.Floor, floor: null },
      { kind: GroupTitleKind.Floor, floor: run.floorDetails[0] },
      { kind: GroupTitleKind.Floor, floor: run.floorDetails[1] },
    ])
    expect(groups.map((g) => ids(g.items))).toEqual([[1], [2], [3, 105]])
  })

  it('drops an empty group outside the logged reading', () => {
    const lonely = { ...run, familiars: [] } as RunView
    expect(groupItems(lonely, RunItemView.ByType)).toHaveLength(3)
  })
})
