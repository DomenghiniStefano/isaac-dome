import { entriesFrom, Severity } from './spec'
import type { DiagnosticEntry, DiagnosticRow } from './spec'
import type { RunsDiagnostic } from '@/lib/ipc/types'

// Every way the archive can be less than whole, and how loudly each one should say so.
//
// None of them may be left out: an empty list with an unread archive behind it reads as "you
// have played nothing", which is the app answering a question it could not ask. The one that
// stands in for the whole content — no folder to read — is the alarm; the one that only
// changes how an item is *named* is a note.
const table: Record<RunsDiagnostic['kind'], DiagnosticRow> = {
  noLogFolder: {
    severity: Severity.Warning,
    body: 'runs.diagnostic.noLogFolder',
  },
  storeUnavailable: {
    severity: Severity.Warning,
    body: 'runs.diagnostic.storeUnavailable',
  },
  unreadableEvents: {
    severity: Severity.Note,
    body: 'runs.diagnostic.unreadableEvents',
  },
  noCatalog: {
    severity: Severity.Info,
    body: 'runs.diagnostic.noCatalog',
  },
}

export const runsEntries = (diagnostics: RunsDiagnostic[]): DiagnosticEntry[] =>
  entriesFrom(diagnostics, table)
