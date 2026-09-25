import { mapValues } from 'lodash-es'

// The five tones a state row's square can wear, whichever list it belongs to. A state is never
// colour alone — the square carries the colour, the name beside it says it — and the same tone
// is the same square on every screen: done, doable now, in the way, partly read (the blocked
// colour with a dashed edge), and not read at all (the unknown hatch).
export const StateTone = {
  Done: 'done',
  Now: 'now',
  Blocked: 'blocked',
  Partial: 'partial',
  Unknown: 'unknown',
} as const
export type StateTone = (typeof StateTone)[keyof typeof StateTone]

const toneDot: Record<StateTone, string> = {
  [StateTone.Done]: 'bg-state-done',
  [StateTone.Now]: 'bg-state-now',
  [StateTone.Blocked]: 'bg-state-blocked',
  [StateTone.Partial]: 'border border-dashed border-state-blocked',
  [StateTone.Unknown]:
    'hatch-unknown border border-dashed border-state-unknown',
}

// A list's squares, from the tone each of its states wears.
export const stateDots = <State extends string>(
  tones: Record<State, StateTone>,
): Record<State, string> => mapValues(tones, (tone) => toneDot[tone])
