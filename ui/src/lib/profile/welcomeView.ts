import { assertNever } from '@/lib/assertNever'
import type { MissingReason, SetupState } from '@/lib/ipc/types'
import { LoadStatus } from '@/stores/loadStatus'

/**
 * Whether the window draws the welcome instead of the shell, and what it says.
 *
 * The takeover is a **state, not a route**: here a route is a tab, and 3.7b's session
 * document stores a window's tabs by location — a tab pointing at the welcome would be saved
 * and restored, which puts a state inside a document that describes contents.
 *
 * It owns two of the states the gate used to flatten (`needsChoice` and `none`), which is why
 * `gateState` no longer has them: with no profile settled the shell is not drawn at all.
 */
export type WelcomeState =
  /** The shell draws — a profile is settled, or nothing is known yet and a guess would flash. */
  | { kind: 'hidden' }
  /** The setup could not be read, and there is no profile to fall back on. */
  | { kind: 'failed' }
  /** There are candidates. `savedGone` is the one case with a sentence of its own. */
  | { kind: 'choose'; savedGone: boolean }
  /** Nothing to choose, and we know which link of the chain broke. */
  | { kind: 'empty'; reason: MissingReason }

export const welcomeState = (
  setup: SetupState | null,
  status: LoadStatus,
  picking: boolean,
): WelcomeState => {
  const settled = setup !== null && setup.active.kind === 'active'
  // The same order `gateState` states as its own property: a settled profile wins over a
  // failed reload, so a window that was working a second ago is not blanked by one.
  if (settled && !picking) return { kind: 'hidden' }
  if (status === LoadStatus.Failed) return { kind: 'failed' }
  // Nothing read yet: the shell draws its skeletons. Showing the welcome here would flash it
  // in front of everyone who has a profile already.
  if (setup === null) return { kind: 'hidden' }
  switch (setup.active.kind) {
    case 'active':
      return { kind: 'choose', savedGone: false }
    case 'needsChoice':
      return {
        kind: 'choose',
        savedGone: setup.active.reason.kind === 'savedProfileGone',
      }
    case 'none':
      return { kind: 'empty', reason: setup.active.reason }
    default:
      return assertNever(setup.active)
  }
}
