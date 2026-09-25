import { describe, expect, it } from 'vitest'
import { MarkColumnView, MarkLevelView, SecondLevelView } from '@/lib/ipc/types'
import type { LiveOpen } from '@/lib/ipc/types'
import { cellName, opensRows } from './opensRows'

const t = (key: string): string => `«${key}»`

const open = (over: Partial<LiveOpen> = {}): LiveOpen => ({
  character: 4,
  characterName: 'Judas',
  column: MarkColumnView.Satan,
  level: MarkLevelView.Base,
  secondLevel: null,
  achievements: [],
  ...over,
})

describe('cellName', () => {
  it("is the column's boss at the base level", () => {
    expect(cellName(t, open())).toBe('Satan')
  })

  it("adds the second level in the column's own word", () => {
    expect(
      cellName(
        t,
        open({
          column: MarkColumnView.Greed,
          level: MarkLevelView.Second,
          secondLevel: SecondLevelView.UltraGreedier,
        }),
      ),
    ).toBe('Greed · «live.secondLevel.ultraGreedier»')
  })
})

describe('opensRows', () => {
  it('gives every achievement of every cell a row of its own', () => {
    const rows = opensRows(
      [
        open({
          achievements: [
            {
              achievement: {
                kind: 'known',
                id: 88,
                text: 'Judas’ Tongue',
                condition: 'Beat Satan as Judas',
                iconUrl: 'asset://88.png',
              },
              fanOut: 2,
            },
            { achievement: { kind: 'unknown', slot: 640 }, fanOut: 0 },
          ],
        }),
      ],
      t,
    )
    expect(rows).toEqual([
      {
        key: '4-satan-Judas’ Tongue',
        name: 'Judas’ Tongue',
        target: { kind: 'achievement', id: 88 },
        condition: 'Beat Satan as Judas',
        iconUrl: 'asset://88.png',
        cell: 'Satan',
        fanOut: 2,
      },
      {
        key: '4-satan-640',
        name: '640',
        target: null,
        condition: null,
        iconUrl: null,
        cell: 'Satan',
        fanOut: 0,
      },
    ])
  })

  it('a run that opens nothing has no rows', () => {
    expect(opensRows([open()], t)).toEqual([])
  })
})
