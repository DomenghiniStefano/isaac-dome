import { storeReasonPart } from '@/lib/ipc/errorText'
import { DiagnosticAction, entriesFrom, Severity } from './spec'
import type { DiagnosticEntry, DiagnosticRow } from './spec'
import type { QueueDiagnostic } from '@/lib/ipc/types'

// `completed` and `unresolved` are said under the rows themselves, by the table that left
// them out: `null` is the screen saying so, not a kind nobody handled.
const table: Record<QueueDiagnostic['kind'], DiagnosticRow | null> = {
  storeUnavailable: {
    severity: Severity.Warning,
    title: 'plan.alerts.storeUnavailableTitle',
    body: 'ipcErrors.storeUnavailable',
  },
  unreadable: {
    severity: Severity.Info,
    title: 'plan.alerts.unreadableTitle',
    body: 'plan.alerts.unreadable',
  },
  noCatalog: {
    severity: Severity.Info,
    title: 'plan.alerts.noCatalogTitle',
    body: 'plan.alerts.noCatalog',
  },
  goalsPending: {
    severity: Severity.Info,
    title: 'plan.alerts.goalsPendingTitle',
    body: 'plan.alerts.goalsPending',
    action: DiagnosticAction.ImportGoals,
  },
  completed: null,
  unresolved: null,
}

// The one row whose sentence needs a second key: what failed, then why. N2 made the reason a
// variant precisely so this half could be built where the words live, and the table above
// cannot express it because the key depends on the value.
export const planEntries = (
  diagnostics: QueueDiagnostic[],
): DiagnosticEntry[] => {
  const unavailable = diagnostics.find((d) => d.kind === 'storeUnavailable')
  return entriesFrom(diagnostics, table).map((e) =>
    e.key === 'storeUnavailable' && unavailable?.kind === 'storeUnavailable'
      ? { ...e, body: [...e.body, storeReasonPart(unavailable.reason)] }
      : e,
  )
}
