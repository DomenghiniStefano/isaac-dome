import { describe, expect, it } from 'vitest'
import { candidates, setupWith } from '@/lib/ipc/fixtures/profile'
import { MissingReason } from '@/lib/ipc/types'
import type { CandidateView, SetupState } from '@/lib/ipc/types'
import { LoadStatus } from '@/stores/loadStatus'
import { gateState } from './gateView'

const first = candidates[0] as CandidateView

const activeSetup = setupWith({
  kind: 'active',
  profile: first,
  autoSelected: false,
})

const choosingSetup = setupWith({
  kind: 'needsChoice',
  reason: { kind: 'neverChosen' },
  suggested: null,
})

// Nothing to choose from: the chain broke before the saves, so `candidates` is empty.
const blockedSetup = (reason: MissingReason): SetupState => ({
  ...setupWith({ kind: 'none', reason }),
  candidates: [],
})

describe('gateState', () => {
  it('hands the screen through when a profile is active', () => {
    expect(gateState(activeSetup, LoadStatus.Ready)).toEqual({
      kind: 'content',
    })
  })

  it('waits before the first answer, instead of claiming there is nothing', () => {
    expect(gateState(null, LoadStatus.Loading)).toEqual({ kind: 'waiting' })
  })

  it('asks for a choice only when there is one to make', () => {
    expect(gateState(choosingSetup, LoadStatus.Ready)).toEqual({
      kind: 'selection',
    })
  })

  it('says the game is missing, not that you failed to choose', () => {
    expect(
      gateState(blockedSetup(MissingReason.GameNotFound), LoadStatus.Ready),
    ).toEqual({ kind: 'blocked', reason: MissingReason.GameNotFound })
  })

  it('tells the three reasons apart, so one sentence cannot serve them all', () => {
    const reasons = [
      MissingReason.SteamNotFound,
      MissingReason.GameNotFound,
      MissingReason.NoSaves,
    ]
    expect(
      reasons.map((r) => gateState(blockedSetup(r), LoadStatus.Ready)),
    ).toEqual(reasons.map((r) => ({ kind: 'blocked', reason: r })))
  })

  it('sends a failed read to the profile screen, which carries the error', () => {
    expect(gateState(null, LoadStatus.Failed)).toEqual({ kind: 'failed' })
  })

  // The order of the branches is the property: `isActive` beating `waiting` is what keeps a
  // working screen lit today, and a reload that fails must not be the thing that blanks it.
  it('does not switch off a working screen when a reload fails', () => {
    expect(gateState(activeSetup, LoadStatus.Failed)).toEqual({
      kind: 'content',
    })
  })
})
