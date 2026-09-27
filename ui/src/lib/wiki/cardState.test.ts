import { describe, expect, it } from 'vitest'
import { StateTone } from '@/lib/facets/stateTone'
import type { PageProgress } from '@/lib/ipc/types'
import { cardState } from './cardState'

describe('cardState', () => {
  it('says nothing without a save or for a kind the save does not know', () => {
    expect(cardState(null)).toBeNull()
  })

  it('is done when the page is complete, whatever its first line says', () => {
    // A boss killed once is done, though its first line is the "met" count.
    const boss: PageProgress = {
      kind: 'bestiary',
      met: 58,
      killed: 61,
      killedYou: 2,
    }
    expect(cardState(boss)?.tone).toBe(StateTone.Done)
  })

  it('takes the first line’s tone when the page is not complete', () => {
    const locked: PageProgress = {
      kind: 'item',
      collected: false,
      unlocked: false,
      unlockedBy: 7,
    }
    expect(cardState(locked)?.tone).toBe(StateTone.Blocked)
    const met: PageProgress = {
      kind: 'bestiary',
      met: 3,
      killed: 0,
      killedYou: 0,
    }
    expect(cardState(met)?.tone).toBe(StateTone.Now)
  })

  it('carries a character’s marks as a fraction, and nothing else does', () => {
    const isaac: PageProgress = {
      kind: 'character',
      unlocked: true,
      marksDone: 9,
      marksTotal: 12,
    }
    expect(cardState(isaac)?.fraction).toEqual({ done: 9, total: 12 })
    const hidden: PageProgress = {
      kind: 'character',
      unlocked: true,
      marksDone: 0,
      marksTotal: 0,
    }
    expect(cardState(hidden)?.fraction).toBeNull()
    expect(cardState({ kind: 'achievement', done: true })?.fraction).toBeNull()
  })

  it('keeps every line, for the hover', () => {
    const boss: PageProgress = {
      kind: 'bestiary',
      met: 1,
      killed: 1,
      killedYou: 0,
    }
    expect(cardState(boss)?.lines.map((l) => l.key)).toEqual([
      'met',
      'killed',
      'killedYou',
    ])
  })
})
