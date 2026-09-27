import { assertNever } from '@/lib/assertNever'
import { StateTone } from '@/lib/facets/stateTone'
import type { MessagePart } from '@/lib/ipc/errorText'
import type { ChallengeStateView, PageProgress } from '@/lib/ipc/types'

// One row of a page's profile state (design decision 6): a `Badge` tone and the sentence it
// carries. `ProgressBadge` draws these compact; a single page's hero draws the same lines
// larger, so the shape is a plain view value and not a component of its own.
export interface ProgressLine {
  key: string
  variant: StateTone
  label: MessagePart
}

const line = (
  key: string,
  variant: StateTone,
  label: MessagePart,
): ProgressLine => ({
  key,
  variant,
  label,
})

const challengeLine = (state: ChallengeStateView): ProgressLine => {
  switch (state.kind) {
    case 'done':
      return line('state', StateTone.Done, { key: 'wiki.progress.done' })
    case 'available':
      return line('state', StateTone.Now, { key: 'wiki.progress.available' })
    case 'blocked':
      return line('state', StateTone.Blocked, {
        key: 'wiki.progress.blocked',
        params: { count: state.missing.length },
      })
    case 'unknown':
      return line('state', StateTone.Unknown, {
        key: 'wiki.progress.unknownState',
      })
    default:
      return assertNever(state)
  }
}

const itemLines = (
  p: Extract<PageProgress, { kind: 'item' }>,
): ProgressLine[] => {
  const lines: ProgressLine[] = []
  if (p.collected !== null) {
    lines.push(
      line('collected', p.collected ? StateTone.Done : StateTone.Blocked, {
        key: p.collected
          ? 'wiki.progress.itemCollected'
          : 'wiki.progress.itemNotCollected',
      }),
    )
  }
  if (p.unlocked === true) {
    lines.push(
      line('unlocked', StateTone.Done, { key: 'wiki.progress.unlocked' }),
    )
  } else if (p.unlocked === false) {
    lines.push(
      p.unlockedBy === null
        ? line('unlocked', StateTone.Blocked, { key: 'wiki.progress.locked' })
        : line('unlocked', StateTone.Blocked, {
            key: 'wiki.progress.lockedBy',
            params: { id: p.unlockedBy },
          }),
    )
  }
  return lines
}

const characterLines = (
  p: Extract<PageProgress, { kind: 'character' }>,
): ProgressLine[] => {
  const lines: ProgressLine[] = []
  if (p.unlocked === true) {
    lines.push(
      line('unlocked', StateTone.Done, { key: 'wiki.progress.unlocked' }),
    )
  } else if (p.unlocked === false) {
    lines.push(
      line('unlocked', StateTone.Blocked, { key: 'wiki.progress.locked' }),
    )
  }
  // Vacuity guard: a character with no marks at all (`marksTotal` 0) has nothing to report,
  // not "0 of 0" — the same reading `categoryProgress` gives an empty category.
  if (p.marksTotal > 0) {
    const done = p.marksDone >= p.marksTotal
    lines.push(
      line('marks', done ? StateTone.Done : StateTone.Now, {
        key: 'wiki.progress.marks',
        params: { done: p.marksDone, total: p.marksTotal },
      }),
    )
  }
  return lines
}

// The three tallies `docs/save-format.md` measured (design decision 7): met is still in
// progress, killed is the good outcome, killed you the bad one — three of the four state
// tones this module otherwise uses, read for what each tally means rather than to compare
// them (decision 7: killed and met disagree on 213 keys, and nothing is derived from that).
const bestiaryLines = (
  p: Extract<PageProgress, { kind: 'bestiary' }>,
): ProgressLine[] => [
  line('met', StateTone.Now, {
    key: 'wiki.progress.bestiaryMet',
    params: { count: p.met },
  }),
  line('killed', StateTone.Done, {
    key: 'wiki.progress.bestiaryKilled',
    params: { count: p.killed },
  }),
  line('killedYou', StateTone.Blocked, {
    key: 'wiki.progress.bestiaryKilledYou',
    params: { count: p.killedYou },
  }),
]

// Every line a page's own `PageProgress` earns, one variant per kind (design decision 6's
// table). A field the save never answered (`null`) writes no line — not a guessed "not done".
export const progressLines = (progress: PageProgress): ProgressLine[] => {
  switch (progress.kind) {
    case 'achievement':
      return [
        line('done', progress.done ? StateTone.Done : StateTone.Blocked, {
          key: progress.done ? 'wiki.progress.done' : 'wiki.progress.notDone',
        }),
      ]
    case 'item':
      return itemLines(progress)
    case 'unlockable':
      return [
        progress.unlocked
          ? line('unlocked', StateTone.Done, { key: 'wiki.progress.unlocked' })
          : line('unlocked', StateTone.Blocked, {
              key: 'wiki.progress.lockedBy',
              params: { id: progress.unlockedBy },
            }),
      ]
    case 'character':
      return characterLines(progress)
    case 'challenge':
      return [challengeLine(progress.state)]
    case 'bestiary':
      return bestiaryLines(progress)
    default:
      return assertNever(progress)
  }
}
