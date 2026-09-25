import type { Message, Translate } from '@/i18n/message'
import { assertNever } from '@/lib/assertNever'
import type { RunOutcomeView, RunSource, RunsDiagnostic } from '@/lib/ipc/types'
import type { FilterBarLabels } from '@/lib/facets/labels'
import { StateTone, stateDots } from '@/lib/facets/stateTone'
import type { FacetSlot } from '@/lib/facets/facetOptions'
import type { FilterBarDescriptor } from '@/lib/facets/filterBar'
import type { RunView } from '@/lib/ipc/types'
import { RunCompany, RunFacet, outcomeOrder, runFaceting } from './runFacets'

const facetTitle: Record<RunFacet, Message> = {
  [RunFacet.Outcome]: 'runs.facet.outcome',
  [RunFacet.Character]: 'runs.facet.character',
  [RunFacet.Online]: 'runs.facet.online',
  [RunFacet.Source]: 'runs.facet.source',
}

/**
 * The outcome in one word. `open` is the run being played and is **not** a failure — the
 * model's own spec says it must never be drawn as one, so it is worded as a state and not as
 * an ending.
 *
 * A table, read by the state row and by the row's badge alike, so the two say the same thing.
 */
export const outcomeText: Record<RunOutcomeView['kind'], Message> = {
  won: 'runs.outcome.won',
  died: 'runs.outcome.died',
  abandoned: 'runs.outcome.abandoned',
  open: 'runs.outcome.open',
}

/**
 * The outcome is the Run diary's state (spec 3.10 §5), and it wears the tones `RunRow.vue`
 * already gives it: won is done, died is blocked, abandoned is the dashed partial — a fact,
 * never the unknown hatch this design keeps for what it could not read — and open is in
 * progress, never a failure.
 */
const outcomeDot = stateDots<RunOutcomeView['kind']>({
  won: StateTone.Done,
  died: StateTone.Blocked,
  abandoned: StateTone.Partial,
  open: StateTone.Now,
})

// Spec 3.10 §3: the character is the filter a reader reaches for on this list; who you played
// with and where the run was read from are behind the fold.
const runSlots: FacetSlot<RunFacet>[] = [
  { facet: RunFacet.Character, inView: true },
  { facet: RunFacet.Online, inView: false },
  { facet: RunFacet.Source, inView: false },
]

const barLabels: FilterBarLabels = {
  rows: 'runs.rows',
  search: 'runs.search',
}

/** A table, as the outcome's is: exhaustive by its type, and the one list `isSource` reads. */
export const sourceText: Record<RunSource['kind'], Message> = {
  live: 'runs.source.live',
  session: 'runs.source.session',
}

// A facet value is a string, and one the build does not know — a newer backend, a hand-edited
// fixture — is shown as it came rather than looked up into `undefined` (card #80, P10).
const isOutcome = (value: string): value is RunOutcomeView['kind'] =>
  Object.hasOwn(outcomeText, value)
const isSource = (value: string): value is RunSource['kind'] =>
  Object.hasOwn(sourceText, value)

/** A facet's value in words. A value outside its set is shown as it came, never dropped. */
export const runFacetValueLabel = (
  t: Translate,
  facet: RunFacet,
  value: string,
): string => {
  switch (facet) {
    case RunFacet.Outcome:
      return isOutcome(value) ? t(outcomeText[value]) : value
    case RunFacet.Online:
      switch (value) {
        case RunCompany.Online:
          return t('runs.online.online')
        case RunCompany.Solo:
          return t('runs.online.solo')
        default:
          return value
      }
    case RunFacet.Source:
      return isSource(value) ? t(sourceText[value]) : value
    // The characters are names the log printed: there is nothing to translate, and the same
    // name covers a Tainted form (the spec's §3).
    case RunFacet.Character:
      return value
    default:
      return assertNever(facet)
  }
}

// The Run diary's filter bar. No sort: the order is the archive's (`runOrder.ts`).
export const runsBar: FilterBarDescriptor<RunView, RunFacet, never> = {
  faceting: runFaceting,
  facets: runSlots,
  state: {
    facet: RunFacet.Outcome,
    order: outcomeOrder,
    dot: outcomeDot,
    text: outcomeText,
  },
  title: facetTitle,
  labels: barLabels,
}

/**
 * What an archive that is less than whole says about itself. Each diagnostic names what is
 * missing: none of them is allowed to read as "no runs", which would be the app answering a
 * question it could not ask.
 */
export const diagnosticText = (d: RunsDiagnostic): Message => {
  switch (d.kind) {
    case 'noLogFolder':
      return 'runs.diagnostic.noLogFolder'
    case 'storeUnavailable':
      return 'runs.diagnostic.storeUnavailable'
    case 'unreadableEvents':
      return 'runs.diagnostic.unreadableEvents'
    case 'noCatalog':
      return 'runs.diagnostic.noCatalog'
    case 'unreadableSessions':
      return 'runs.diagnostic.unreadableSessions'
    case 'liveLogUnreadable':
      return 'runs.diagnostic.liveLogUnreadable'
    default:
      return assertNever(d)
  }
}
