import { describe, expect, it } from 'vitest'
import { candidates, setupWith } from '@/lib/ipc/fixtures/profile'
import { MissingReason } from '@/lib/ipc/types'
import type { CandidateView, SetupState } from '@/lib/ipc/types'
import { gateState } from './gateView'

const first = candidates[0] as CandidateView

const activeSetup = setupWith({
  kind: 'active',
  profile: first,
  autoSelected: false,
})

describe('gateState', () => {
  it('hands the screen through when a profile is active', () => {
    expect(gateState(activeSetup)).toEqual({ kind: 'content' })
  })

  it('waits before the first answer, instead of claiming there is nothing', () => {
    expect(gateState(null)).toEqual({ kind: 'waiting' })
  })

  // "Choose one of these", "the chain broke here" and "the read failed" are `welcomeView.ts`'s
  // states, asserted in `welcomeView.test.ts` on the function that owns them.
  it('waits rather than guessing, in the states the welcome now owns', () => {
    const choosing = setupWith({
      kind: 'needsChoice',
      reason: { kind: 'neverChosen' },
      suggested: null,
    })
    const nothing: SetupState = {
      ...setupWith({ kind: 'none', reason: MissingReason.GameNotFound }),
      candidates: [],
    }
    expect(gateState(choosing)).toEqual({ kind: 'waiting' })
    expect(gateState(nothing)).toEqual({ kind: 'waiting' })
  })
})
