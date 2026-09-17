import { assertNever } from '@/lib/assertNever'
import type { ActiveProfile, SetupState } from '@/lib/ipc/types'

/**
 * What a screen that needs a profile shows, now that it is only ever mounted with one.
 *
 * It used to carry three more states — "there are candidates, choose one", "the chain broke
 * here", "the read failed" — because the gate *was* the profile selection (DESIGN-BRIEF.md
 * §4.3). Since 3.8 the welcome takes all three, above the router, and while any of them is
 * true the shell is not drawn at all. They live in `welcomeView.ts`, with their tests.
 *
 * What is left is the one moment the shell is up without a profile: the instant before the
 * first answer. Skeletons, never "there is nothing" — that would be a guess.
 */
export type GateState =
  /** A profile is active: the screen itself. */
  | { kind: 'content' }
  /** Nothing is known yet. */
  | { kind: 'waiting' }

const fromActive = (active: ActiveProfile): GateState => {
  switch (active.kind) {
    case 'active':
      return { kind: 'content' }
    // Unreachable through the shell: the welcome is drawn instead of everything below it.
    // Reached anyway, a window between two states shows skeletons, not a screen it cannot
    // fill.
    case 'needsChoice':
    case 'none':
      return { kind: 'waiting' }
    default:
      return assertNever(active)
  }
}

export const gateState = (setup: SetupState | null): GateState =>
  setup === null ? { kind: 'waiting' } : fromActive(setup.active)
