import { describe, expect, it } from 'vitest'
import { candidates, noneSetup, setupWith } from '@/lib/ipc/fixtures/profile'
import type { CandidateView } from '@/lib/ipc/types'
import { settledProfile } from './settledProfile'

const first = candidates[0] as CandidateView

describe('settledProfile', () => {
  it('answers the active profile, flag and all', () => {
    const active = {
      kind: 'active',
      profile: first,
      autoSelected: true,
    } as const
    expect(settledProfile(setupWith(active))).toStrictEqual(active)
  })

  it('is null before anything is read', () => {
    expect(settledProfile(null)).toBeNull()
  })

  it('is null while a choice is still to make', () => {
    const setup = setupWith({
      kind: 'needsChoice',
      reason: { kind: 'neverChosen' },
      suggested: first.id,
    })
    expect(settledProfile(setup)).toBeNull()
  })

  it('is null where no save was found', () => {
    expect(settledProfile(noneSetup)).toBeNull()
  })
})
