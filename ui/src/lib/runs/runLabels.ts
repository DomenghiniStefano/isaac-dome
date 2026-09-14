import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import type { RunOutcomeView, RunSource, RunsDiagnostic } from '@/lib/ipc/types'
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
 */
export const outcomeText = (outcome: RunOutcomeView['kind']): Key => {
  switch (outcome) {
    case 'won':
      return 'runs.outcome.won'
    case 'died':
      return 'runs.outcome.died'
    case 'abandoned':
      return 'runs.outcome.abandoned'
    case 'open':
      return 'runs.outcome.open'
    default:
      return assertNever(outcome)
  }
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
