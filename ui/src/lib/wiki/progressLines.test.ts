import { describe, expect, it } from 'vitest'
import { StateTone } from '@/lib/facets/stateTone'
import type { PageProgress } from '@/lib/ipc/types'
import { progressLines } from './progressLines'

describe('progressLines', () => {
  it('reads an achievement as done or not done', () => {
    expect(progressLines({ kind: 'achievement', done: true })).toEqual([
      {
        key: 'done',
        variant: StateTone.Done,
        label: { key: 'wiki.progress.done' },
      },
    ])
    expect(progressLines({ kind: 'achievement', done: false })).toEqual([
      {
        key: 'done',
        variant: StateTone.Blocked,
        label: { key: 'wiki.progress.notDone' },
      },
    ])
  })

  it('gives an item one line per field the save actually answered', () => {
    const neither: PageProgress = {
      kind: 'item',
      collected: null,
      unlocked: null,
      unlockedBy: null,
    }
    expect(progressLines(neither)).toEqual([])

    const lockedByFive: PageProgress = {
      kind: 'item',
      collected: false,
      unlocked: false,
      unlockedBy: 5,
    }
    expect(progressLines(lockedByFive)).toEqual([
      {
        key: 'collected',
        variant: StateTone.Blocked,
        label: { key: 'wiki.progress.itemNotCollected' },
      },
      {
        key: 'unlocked',
        variant: StateTone.Blocked,
        label: { key: 'wiki.progress.lockedBy', params: { id: 5 } },
      },
    ])
  })

  it('reads an unlockable by its own unlocked flag', () => {
    expect(
      progressLines({ kind: 'unlockable', unlocked: true, unlockedBy: 7 }),
    ).toEqual([
      {
        key: 'unlocked',
        variant: StateTone.Done,
        label: { key: 'wiki.progress.unlocked' },
      },
    ])
    expect(
      progressLines({ kind: 'unlockable', unlocked: false, unlockedBy: 7 }),
    ).toEqual([
      {
        key: 'unlocked',
        variant: StateTone.Blocked,
        label: { key: 'wiki.progress.lockedBy', params: { id: 7 } },
      },
    ])
  })

  it('skips the marks line when the character has no marks at all (vacuity guard)', () => {
    const noMarks: PageProgress = {
      kind: 'character',
      unlocked: null,
      marksDone: 0,
      marksTotal: 0,
    }
    expect(progressLines(noMarks)).toEqual([])
  })

  it('gives a character its unlock state and its marks, done once done reaches total', () => {
    const partial: PageProgress = {
      kind: 'character',
      unlocked: true,
      marksDone: 3,
      marksTotal: 12,
    }
    expect(progressLines(partial)).toEqual([
      {
        key: 'unlocked',
        variant: StateTone.Done,
        label: { key: 'wiki.progress.unlocked' },
      },
      {
        key: 'marks',
        variant: StateTone.Now,
        label: { key: 'wiki.progress.marks', params: { done: 3, total: 12 } },
      },
    ])
    const complete: PageProgress = {
      kind: 'character',
      unlocked: true,
      marksDone: 12,
      marksTotal: 12,
    }
    expect(progressLines(complete)[1]?.variant).toBe(StateTone.Done)
  })

  it('maps every challenge state to its own tone, blocked carrying the missing count', () => {
    expect(
      progressLines({ kind: 'challenge', state: { kind: 'done' } }),
    ).toEqual([
      {
        key: 'state',
        variant: StateTone.Done,
        label: { key: 'wiki.progress.done' },
      },
    ])
    expect(
      progressLines({ kind: 'challenge', state: { kind: 'available' } }),
    ).toEqual([
      {
        key: 'state',
        variant: StateTone.Now,
        label: { key: 'wiki.progress.available' },
      },
    ])
    expect(
      progressLines({
        kind: 'challenge',
        state: { kind: 'blocked', missing: [1, 2] },
      }),
    ).toEqual([
      {
        key: 'state',
        variant: StateTone.Blocked,
        label: { key: 'wiki.progress.blocked', params: { count: 2 } },
      },
    ])
    expect(
      progressLines({ kind: 'challenge', state: { kind: 'unknown' } }),
    ).toEqual([
      {
        key: 'state',
        variant: StateTone.Unknown,
        label: { key: 'wiki.progress.unknownState' },
      },
    ])
  })

  it("names a bestiary entry's three tallies, and only those three", () => {
    const lines = progressLines({
      kind: 'bestiary',
      met: 4,
      killed: 2,
      killedYou: 1,
    })
    expect(lines.map((l) => l.key)).toEqual(['met', 'killed', 'killedYou'])
    expect(lines.map((l) => l.variant)).toEqual([
      StateTone.Now,
      StateTone.Done,
      StateTone.Blocked,
    ])
    expect(lines.map((l) => l.label)).toEqual([
      { key: 'wiki.progress.bestiaryMet', params: { count: 4 } },
      { key: 'wiki.progress.bestiaryKilled', params: { count: 2 } },
      { key: 'wiki.progress.bestiaryKilledYou', params: { count: 1 } },
    ])
  })
})
