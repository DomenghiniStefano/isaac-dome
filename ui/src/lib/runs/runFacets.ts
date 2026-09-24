import { sortBy, uniq } from 'lodash-es'
import { assertNever } from '@/lib/assertNever'
import { createFaceting } from '@/lib/facets/faceting'
import type { RunOutcomeView, RunSource, RunView } from '@/lib/ipc/types'

// The Run diary's half of a faceted list: which facets it has, how a run answers one, what the
// search reads, what each facet offers. Matching, the counts and the active count are the
// engine's — this is the third screen on it, which is what N3 unified it for.

export const RunFacet = {
  Outcome: 'outcome',
  Character: 'character',
  Online: 'online',
  Source: 'source',
} as const
export type RunFacet = (typeof RunFacet)[keyof typeof RunFacet]

const facetOrder: RunFacet[] = [
  RunFacet.Outcome,
  RunFacet.Character,
  RunFacet.Online,
  RunFacet.Source,
]

// The four outcomes in the order the screen offers them: what finished, then what did not.
// `open` is the run being played and is never a failure — sub-project 1's spec says so and
// the order says it too, by not putting it beside `died`.
//
// Exported since 3.10, when the outcome became the Run diary's state row: the control draws
// all four whatever the archive holds, so it needs the order and not just the values present.
export const outcomeOrder: RunOutcomeView['kind'][] = [
  'won',
  'died',
  'abandoned',
  'open',
]

// Ours, not the wire's: the archive says  and a facet needs two values with
// names. A const object and not a union of strings — the repo's rule, and the same reason the
// wire's own fieldless enums are objects.
export const RunCompany = {
  Online: 'online',
  Solo: 'solo',
} as const
export type RunCompany = (typeof RunCompany)[keyof typeof RunCompany]

const onlineOrder: RunCompany[] = [RunCompany.Online, RunCompany.Solo]
// Typed on the wire's tag, so a renamed source is a compile error here and not a facet value
// that stops sorting.
const sourceOrder: RunSource['kind'][] = ['live', 'session']

const facetValues = (run: RunView, facet: RunFacet): string[] => {
  switch (facet) {
    case RunFacet.Outcome:
      return [run.outcome.kind]
    // A run with no character contributes **no value**: `character: null` is "no item line
    // ever named it", which is not a character called unknown, and offering one would put a
    // row in the list that the archive does not have.
    case RunFacet.Character:
      return run.character === null ? [] : [run.character]
    case RunFacet.Online:
      return [run.online ? RunCompany.Online : RunCompany.Solo]
    case RunFacet.Source:
      return [run.source.kind]
    default:
      return assertNever(facet)
  }
}

// What the search reads: the seed is how a player names a run to themselves, the character is
// how they remember it. The ending and the killer come with the outcome, so a search for
// "Monstro" finds the runs it ended.
const searchText = (run: RunView): string =>
  [
    run.seedWords,
    run.character ?? '',
    run.outcome.kind === 'won' ? run.outcome.ending : '',
    run.outcome.kind === 'died' ? run.outcome.killer : '',
  ].join(' ')

const facetOptions = (runs: RunView[], facet: RunFacet): string[] => {
  switch (facet) {
    case RunFacet.Outcome:
      return outcomeOrder
    case RunFacet.Online:
      return onlineOrder
    case RunFacet.Source:
      return sourceOrder
    // The characters are whoever was played, by name, because a name is all the log gives —
    // and the same name covers a Tainted form (§3 of the spec).
    case RunFacet.Character:
      return sortBy(uniq(runs.flatMap((run) => facetValues(run, facet))))
    default:
      return assertNever(facet)
  }
}

export const runFaceting = createFaceting<RunView, RunFacet>({
  order: facetOrder,
  values: facetValues,
  text: searchText,
  options: facetOptions,
})
