import { storeReasonPart } from '@/lib/ipc/errorText'
import { entriesFrom, Severity } from './spec'
import type { DiagnosticEntry, DiagnosticRow } from './spec'
import type { RollDiagnostic } from '@/lib/ipc/types'

// `emptyDeck` says nothing here: the card's own empty state already names the exclusion that
// emptied the deck and the count behind it, and a second alert with none of that specificity
// would say less about the same fact.
const table: Record<RollDiagnostic['kind'], DiagnosticRow | null> = {
  noCounterSection: {
    severity: Severity.Warning,
    title: 'roll.diagnostics.noCounterSectionTitle',
    body: 'roll.diagnostics.noCounterSection',
  },
  documentUnreadable: {
    severity: Severity.Info,
    title: 'roll.diagnostics.documentUnreadableTitle',
    body: 'roll.diagnostics.documentUnreadable',
  },
  documentFromTheFuture: {
    severity: Severity.Info,
    title: 'roll.diagnostics.documentFromTheFutureTitle',
    body: 'roll.diagnostics.documentFromTheFuture',
  },
  noCatalog: {
    severity: Severity.Warning,
    title: 'roll.diagnostics.noCatalogTitle',
    body: 'roll.diagnostics.noCatalog',
  },
  playabilityUnknown: {
    severity: Severity.Info,
    title: 'roll.diagnostics.playabilityUnknownTitle',
    body: 'roll.diagnostics.playabilityUnknown',
  },
  storeUnavailable: {
    severity: Severity.Warning,
    title: 'roll.diagnostics.storeUnavailableTitle',
    body: 'ipcErrors.storeUnavailable',
  },
  emptyDeck: null,
}

// The one row whose sentence needs a second key: what failed, then why — the same shape
// `planEntries` uses, and for the same reason: the table above cannot express a key that
// depends on the value.
export const rollEntries = (
  diagnostics: RollDiagnostic[],
): DiagnosticEntry[] => {
  const unavailable = diagnostics.find((d) => d.kind === 'storeUnavailable')
  return entriesFrom(diagnostics, table).map((e) =>
    e.key === 'storeUnavailable' && unavailable?.kind === 'storeUnavailable'
      ? { ...e, body: [...e.body, storeReasonPart(unavailable.reason)] }
      : e,
  )
}
