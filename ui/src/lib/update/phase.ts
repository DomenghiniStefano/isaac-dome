import type { MessagePart } from '@/lib/ipc/errorText'
import { UpdateFailure } from '@/lib/ipc/types'
import type { Block, UpdateView } from '@/lib/ipc/types'
import { assertNever } from '@/lib/assertNever'

// What the Updates screen draws, read off the view and nothing else. The phase comes from the
// backend, which holds it; everything here is a reading of it, which is why it is a module of
// functions and not state.

// A build with no updater is not a build whose update is "off": the two are different answers
// and the screen says the second one everywhere.
const offered = (view: UpdateView): boolean => view.unavailable === null

// Something is already on its way, so a second ask is not a second answer.
const busy = (view: UpdateView): boolean =>
  view.phase.kind === 'checking' || view.phase.kind === 'downloading'

export const canCheck = (view: UpdateView): boolean =>
  offered(view) && !busy(view)

export const canInstall = (view: UpdateView): boolean =>
  offered(view) && view.phase.kind === 'ready'

// `null` is an indeterminate bar and not a zero: zero is a measurement, and this is the
// absence of one — the server announced no length.
export const progressPercent = (view: UpdateView): number | null =>
  view.phase.kind === 'downloading' ? view.phase.percent : null

const failureKey = (reason: UpdateFailure): MessagePart['key'] => {
  switch (reason) {
    case UpdateFailure.Offline:
      return 'updates.failedOffline'
    case UpdateFailure.NotPublished:
      return 'updates.failedNotPublished'
    // Not bad luck, and the sentence must not read like it: what was served is not what it
    // claims to be.
    case UpdateFailure.Rejected:
      return 'updates.failedRejected'
    case UpdateFailure.InstallFailed:
      return 'updates.failedInstall'
    case UpdateFailure.Unknown:
      return 'updates.failedUnknown'
    default:
      return assertNever(reason)
  }
}

// The one line under the version. In a build with no updater that is the only fact worth
// saying, whatever phase the machine happens to be resting in.
export const phasePart = (view: UpdateView): MessagePart => {
  if (!offered(view)) return { key: 'updates.unsupported' }
  const phase = view.phase
  switch (phase.kind) {
    // "Nobody looked" and "there is nothing newer" are two different facts, and a launch that
    // never asked must not read as one that asked and found nothing.
    case 'idle':
      return { key: 'updates.idle' }
    case 'checking':
      return { key: 'updates.checking' }
    case 'upToDate':
      return { key: 'updates.upToDate' }
    case 'downloading':
      return { key: 'updates.downloading', params: { version: phase.version } }
    case 'ready':
      return { key: 'updates.ready', params: { version: phase.version } }
    case 'failed':
      return { key: failureKey(phase.reason) }
    default:
      return assertNever(phase)
  }
}

// The release notes, when the manifest carried any. Only ever shown beside the version they
// belong to, which is why they come off the phase and not off a field of their own. Blocks,
// never markdown: Rust read them, because the manifest they came in is not signed.
export const releaseNotes = (view: UpdateView): Block[] | null =>
  view.phase.kind === 'ready' && view.phase.notes.length > 0
    ? view.phase.notes
    : null
