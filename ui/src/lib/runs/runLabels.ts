import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import type { RunOutcomeView, RunSource, RunsDiagnostic } from '@/lib/ipc/types'
import type { FilterBarLabels } from '@/lib/facets/labels'
import type { FacetSlot } from '@/lib/facets/facetOptions'
import { RunCompany, RunFacet } from './runFacets'

type Key = MessageKey<MessageSchema>

export const facetTitle: Record<RunFacet, Key> = {
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
 * A table and not a `switch` since 3.10: the state row takes its words as a table, and one
 * mapping written twice is how the row and the badge end up saying different things.
 */
export const outcomeTextByKind: Record<RunOutcomeView['kind'], Key> = {
  won: 'runs.outcome.won',
  died: 'runs.outcome.died',
  abandoned: 'runs.outcome.abandoned',
  open: 'runs.outcome.open',
}

export const outcomeText = (outcome: RunOutcomeView['kind']): Key =>
  outcomeTextByKind[outcome]

/**
 * The outcome is the Run diary's state (spec 3.10 §5), and it wears the tones `RunRow.vue`
 * already gives it: won is done, died is blocked, abandoned is the dashed partial — a fact,
 * never the unknown hatch this design keeps for what it could not read — and open is in
 * progress, never a failure.
 */
export const outcomeDot: Record<RunOutcomeView['kind'], string> = {
  won: 'bg-state-done',
  died: 'bg-state-blocked',
  abandoned: 'border border-dashed border-state-blocked',
  open: 'bg-state-now',
}

// Spec 3.10 §3: the character is the filter a reader reaches for on this list; who you played
// with and where the run was read from are behind the fold.
export const runSlots: FacetSlot<RunFacet>[] = [
  { facet: RunFacet.Character, inView: true },
  { facet: RunFacet.Online, inView: false },
  { facet: RunFacet.Source, inView: false },
]

export const barLabels: FilterBarLabels = {
  rows: 'runs.rows',
  search: 'runs.search',
}

export const sourceText = (source: RunSource['kind']): Key =>
  source === 'live' ? 'runs.source.live' : 'runs.source.session'

/** A facet's value in words. A value outside its set is shown as it came, never dropped. */
export const facetValueLabel = (
  t: (key: Key, named?: Record<string, unknown>) => string,
  facet: RunFacet,
  value: string,
): string => {
  switch (facet) {
    case RunFacet.Outcome:
      return t(outcomeText(value as RunOutcomeView['kind']))
    case RunFacet.Online:
      return t(
        value === RunCompany.Online ? 'runs.online.online' : 'runs.online.solo',
      )
    case RunFacet.Source:
      return t(sourceText(value as RunSource['kind']))
    // The characters are names the log printed: there is nothing to translate, and the same
    // name covers a Tainted form (the spec's §3).
    case RunFacet.Character:
      return value
    default:
      return assertNever(facet)
  }
}

/**
 * What an archive that is less than whole says about itself. Each diagnostic names what is
 * missing: none of them is allowed to read as "no runs", which would be the app answering a
 * question it could not ask.
 */
export const diagnosticText = (d: RunsDiagnostic): Key => {
  switch (d.kind) {
    case 'noLogFolder':
      return 'runs.diagnostic.noLogFolder'
    case 'storeUnavailable':
      return 'runs.diagnostic.storeUnavailable'
    case 'unreadableEvents':
      return 'runs.diagnostic.unreadableEvents'
    case 'noCatalog':
      return 'runs.diagnostic.noCatalog'
    default:
      return assertNever(d)
  }
}
