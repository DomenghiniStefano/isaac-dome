import { ref } from 'vue'
import { isIpcError } from '@/lib/ipc/errors'

// The one write error that is not swallowed. Every other failure of `set_window_session` leaves
// the tabs on screen and costs nothing to ignore — a dialog because a session did not save would
// be worse than the session not saving. `SessionTooLarge` is different in kind: it means the
// session has **stopped being saved**, and the user finds out at the next start, with nothing
// having gone wrong on screen in the meantime.
//
// A module-level ref, like `focusOrder`: it is a fact about this window, and this window is the
// one doing the writing. A window that is not the elected writer never sets it, which is right —
// it is not the one whose writes are failing.
export const sessionStopped = ref(false)

export const noteSessionError = (e: unknown): void => {
  if (isIpcError(e) && e.kind === 'sessionTooLarge') sessionStopped.value = true
}
