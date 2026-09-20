import { describe, expect, it } from 'vitest'
import { UpdateFailure, UpdateReason } from '@/lib/ipc/types'
import type { UpdatePhase, UpdateView } from '@/lib/ipc/types'
import { canCheck, canInstall, phasePart, progressPercent } from './phase'

const view = (
  phase: UpdatePhase,
  unavailable: UpdateView['unavailable'] = null,
): UpdateView => ({ currentVersion: '0.1.0', phase, unavailable })

const every: UpdatePhase[] = [
  { kind: 'idle' },
  { kind: 'checking' },
  { kind: 'upToDate' },
  { kind: 'downloading', version: '0.2.0', percent: 40 },
  { kind: 'ready', version: '0.2.0', notes: null },
  { kind: 'failed', reason: UpdateFailure.Offline },
]

describe('the check button', () => {
  it('is offered from every phase that is resting', () => {
    // The app sits in the tray for days: "up to date" goes stale, and a failure is the phase
    // you most want to be able to retry from.
    for (const phase of [
      { kind: 'idle' } as const,
      { kind: 'upToDate' } as const,
      { kind: 'ready', version: '0.2.0', notes: null } as const,
      { kind: 'failed', reason: UpdateFailure.Offline } as const,
    ]) {
      expect(canCheck(view(phase)), phase.kind).toBe(true)
    }
  })

  it('is refused while something is already on its way', () => {
    // A second check on top of a running one is not a second answer, and asking during a
    // download would throw away bytes already paid for.
    expect(canCheck(view({ kind: 'checking' }))).toBe(false)
    expect(
      canCheck(view({ kind: 'downloading', version: '0.2.0', percent: 1 })),
    ).toBe(false)
  })

  it('is refused in a build that has no updater, from every phase', () => {
    // Not "off": impossible. A development build registers no plugin, and a button that did
    // nothing would be worse than one that says why.
    for (const phase of every) {
      expect(canCheck(view(phase, UpdateReason.NotSupported)), phase.kind).toBe(
        false,
      )
    }
  })
})

describe('the install button', () => {
  it('exists only once the bytes are verified and held', () => {
    for (const phase of every) {
      expect(canInstall(view(phase)), phase.kind).toBe(phase.kind === 'ready')
    }
  })

  it('is refused in a build that has no updater', () => {
    expect(
      canInstall(
        view(
          { kind: 'ready', version: '0.2.0', notes: null },
          UpdateReason.NotSupported,
        ),
      ),
    ).toBe(false)
  })
})

describe('the progress bar', () => {
  it('reads the percentage while one is announced', () => {
    expect(
      progressPercent(
        view({ kind: 'downloading', version: '0.2.0', percent: 7 }),
      ),
    ).toBe(7)
  })

  it('has no value when the server announced no length', () => {
    // An indeterminate bar, not a zero: zero is a measurement and this is the absence of one.
    expect(
      progressPercent(
        view({ kind: 'downloading', version: '0.2.0', percent: null }),
      ),
    ).toBe(null)
  })

  it('has no value outside a download', () => {
    for (const phase of every.filter((p) => p.kind !== 'downloading')) {
      expect(progressPercent(view(phase)), phase.kind).toBe(null)
    }
  })
})

describe('the sentence', () => {
  it('tells "nothing was asked" from "there is nothing newer"', () => {
    // Two different facts. A launch that never looked must not read as a launch that looked
    // and found nothing.
    expect(phasePart(view({ kind: 'idle' })).key).not.toBe(
      phasePart(view({ kind: 'upToDate' })).key,
    )
  })

  it('names the version it is working on', () => {
    expect(
      phasePart(view({ kind: 'downloading', version: '0.2.0', percent: 3 }))
        .params,
    ).toEqual({ version: '0.2.0' })
    expect(
      phasePart(view({ kind: 'ready', version: '0.2.0', notes: null })).params,
    ).toEqual({ version: '0.2.0' })
  })

  it('gives every failure its own sentence', () => {
    // One per variant, and all different: a new failure cannot land without somebody writing
    // what it means. `rejected` in particular must not read like bad luck — it means what was
    // served is not what it claims to be.
    const keys = Object.values(UpdateFailure).map(
      (reason) => phasePart(view({ kind: 'failed', reason })).key,
    )
    expect(new Set(keys).size).toBe(Object.values(UpdateFailure).length)
  })

  it('says the build has no updater before it says anything about phases', () => {
    // Whatever the phase says, in a development build it is not the fact that matters.
    expect(
      phasePart(view({ kind: 'upToDate' }, UpdateReason.NotSupported)).key,
    ).toBe(phasePart(view({ kind: 'idle' }, UpdateReason.NotSupported)).key)
  })
})
