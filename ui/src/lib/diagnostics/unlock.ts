import { entriesFrom, Severity } from './spec'
import type { DiagnosticEntry, DiagnosticRow } from './spec'
import type { UnlockDiagnostic } from '@/lib/ipc/types'

// The screen's own table, which is the part that is actually its own. The two that compare
// the save with the catalog coexist with real rows: a note, not an alert.
const table: Record<UnlockDiagnostic['kind'], DiagnosticRow> = {
  noCatalog: {
    severity: Severity.Info,
    title: 'unlock.diagnostics.noCatalogTitle',
    body: 'unlock.diagnostics.noCatalog',
  },
  noAchievementSection: {
    severity: Severity.Warning,
    title: 'unlock.diagnostics.noAchievementSectionTitle',
    body: 'unlock.diagnostics.noAchievementSection',
  },
  slotsBeyondCatalog: {
    severity: Severity.Note,
    body: 'unlock.diagnostics.slotsBeyondCatalog',
  },
  catalogBeyondSlots: {
    severity: Severity.Note,
    body: 'unlock.diagnostics.catalogBeyondSlots',
  },
}

export const unlockEntries = (
  diagnostics: UnlockDiagnostic[],
): DiagnosticEntry[] => entriesFrom(diagnostics, table)
