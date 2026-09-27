import type { StateTone } from '@/lib/facets/stateTone'
import { StateTone as Tone } from '@/lib/facets/stateTone'
import type { PageProgress } from '@/lib/ipc/types'
import type { Progress } from './progress'
import { isComplete } from './progress'
import { progressLines } from './progressLines'
import type { ProgressLine } from './progressLines'

// What a list card says about the save, in a form that does not look like the page's facts:
// one tone for the corner mark and the card's edge, the lines behind it for the hover, and a
// fraction when the state is one (a character's marks), drawn as a bar. The numbers
// themselves stay on the page.
export interface CardState {
  tone: StateTone
  fraction: Progress | null
  lines: ProgressLine[]
}

// A complete page is done whatever its first line is — a boss killed at least once opens on
// its "met" count; anything else takes the tone its first line already has.
export const cardState = (progress: PageProgress | null): CardState | null => {
  if (progress === null) return null
  const lines = progressLines(progress)
  const first = lines[0]
  if (first === undefined) return null
  const tone = isComplete(progress) === true ? Tone.Done : first.variant
  return { tone, fraction: fractionOf(progress), lines }
}

const fractionOf = (progress: PageProgress): Progress | null =>
  progress.kind === 'character' && progress.marksTotal > 0
    ? { done: progress.marksDone, total: progress.marksTotal }
    : null
