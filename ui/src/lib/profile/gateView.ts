import { assertNever } from '@/lib/assertNever'
import type { ActiveProfile, MissingReason, SetupState } from '@/lib/ipc/types'
import { LoadStatus } from '@/stores/loadStatus'

/**
 * What the Progress gate shows. Four states and not one: the gate used to collapse them
 * into a single sentence — "choose a profile" — which on a machine where the chain is
 * broken asks for a choice that cannot be made, and says nothing about which link broke.
 * The IPC kept the cases apart (`ActiveProfile`); only the view had flattened them.
 */
export type GateState =
  /** A profile is active: the screen itself. */
  | { kind: 'content' }
  /** Nothing is known yet. Saying "there is nothing" here would be a guess. */
  | { kind: 'waiting' }
  /** There are candidates: DESIGN-BRIEF.md §4.3 — Progress *is* the profile selection. */
  | { kind: 'selection' }
  /** The read failed. The profile screen carries the error and the retry. */
  | { kind: 'failed' }
  /** Nothing to select, and we know why. */
  | { kind: 'blocked'; reason: MissingReason }

const fromActive = (active: ActiveProfile): GateState => {
  switch (active.kind) {
    case 'active':
      return { kind: 'content' }
    case 'needsChoice':
      return { kind: 'selection' }
    case 'none':
      return { kind: 'blocked', reason: active.reason }
    default:
      return assertNever(active)
  }
}

/**
 * An active profile wins over a failed status, which is what the store's `isActive` did by
 * winning over `waiting`: a reload that fails must not be the thing that blanks a screen
 * that was working a second ago.
 */
export const gateState = (
  setup: SetupState | null,
  status: LoadStatus,
): GateState => {
  const state = setup === null ? null : fromActive(setup.active)
  if (state?.kind === 'content') return state
  if (status === LoadStatus.Failed) return { kind: 'failed' }
  return state ?? { kind: 'waiting' }
}
