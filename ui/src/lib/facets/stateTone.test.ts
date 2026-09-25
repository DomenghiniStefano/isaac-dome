import { describe, expect, it } from 'vitest'
import { StateTone, stateDots } from './stateTone'

describe('stateDots', () => {
  it('gives each state the square of its tone', () => {
    expect(
      stateDots({
        won: StateTone.Done,
        open: StateTone.Now,
        died: StateTone.Blocked,
        abandoned: StateTone.Partial,
        unread: StateTone.Unknown,
      }),
    ).toEqual({
      won: 'bg-state-done',
      open: 'bg-state-now',
      died: 'bg-state-blocked',
      abandoned: 'border border-dashed border-state-blocked',
      unread: 'hatch-unknown border border-dashed border-state-unknown',
    })
  })

  it('two lists wearing the same tone draw the same square', () => {
    const a = stateDots({ x: StateTone.Unknown })
    const b = stateDots({ y: StateTone.Unknown })
    expect(a.x).toBe(b.y)
  })
})
