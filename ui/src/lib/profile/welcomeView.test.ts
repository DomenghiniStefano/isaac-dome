import { describe, expect, it } from 'vitest'
import { candidates, setupWith } from '@/lib/ipc/fixtures/profile'
import { MissingReason } from '@/lib/ipc/types'
import type { CandidateView, SetupState } from '@/lib/ipc/types'
import { LoadStatus } from '@/stores/loadStatus'
import { welcomeState } from './welcomeView'

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

const savedGoneSetup = setupWith({
  kind: 'needsChoice',
  reason: { kind: 'savedProfileGone', was: 'abc' },
  suggested: null,
})

// Nothing to choose from: the chain broke before the saves, so `candidates` is empty.
const emptySetup = (reason: MissingReason): SetupState => ({
  ...setupWith({ kind: 'none', reason }),
  candidates: [],
})

describe('welcomeState', () => {
  it('does not draw before the setup is known, so it never flashes', () => {
    expect(welcomeState(null, LoadStatus.Loading, false)).toEqual({
      kind: 'hidden',
    })
  })

  it('stands aside for a settled profile', () => {
    expect(welcomeState(activeSetup, LoadStatus.Ready, false)).toEqual({
      kind: 'hidden',
    })
  })

  it('opens over a settled profile when the picker was asked for', () => {
    expect(welcomeState(activeSetup, LoadStatus.Ready, true)).toEqual({
      kind: 'choose',
      savedGone: false,
    })
  })

  it('asks for a choice when there is one to make', () => {
    expect(welcomeState(choosingSetup, LoadStatus.Ready, false)).toEqual({
      kind: 'choose',
      savedGone: false,
    })
  })

  it('says when the save you had chosen is the one that went', () => {
    expect(welcomeState(savedGoneSetup, LoadStatus.Ready, false)).toEqual({
      kind: 'choose',
      savedGone: true,
    })
  })

  it('takes the broken chain too, rather than an app drawn around nothing', () => {
    expect(
      welcomeState(
        emptySetup(MissingReason.GameNotFound),
        LoadStatus.Ready,
        false,
      ),
    ).toEqual({ kind: 'empty', reason: MissingReason.GameNotFound })
  })

  // Moved here from `gateState` with the states themselves: the welcome owns the broken
  // chain now, and one sentence cannot serve the three reasons.
  it('tells the reasons apart, so one sentence cannot serve them all', () => {
    const reasons = [
      MissingReason.SteamNotFound,
      MissingReason.GameNotFound,
      MissingReason.NoSaves,
    ]
    expect(
      reasons.map((r) => welcomeState(emptySetup(r), LoadStatus.Ready, false)),
    ).toEqual(reasons.map((r) => ({ kind: 'empty', reason: r })))
  })

  // The order of the branches is the property, and it is `gateState`'s: a reload that fails
  // must not be the thing that blanks a window that was working a second ago.
  it('lets a settled profile win over a failed reload', () => {
    expect(welcomeState(activeSetup, LoadStatus.Failed, false)).toEqual({
      kind: 'hidden',
    })
  })

  it('speaks when the read failed and there is no profile to fall back on', () => {
    expect(welcomeState(null, LoadStatus.Failed, false)).toEqual({
      kind: 'failed',
    })
  })
})
