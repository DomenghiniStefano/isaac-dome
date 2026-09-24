import { SearchDiagnostic } from '@/lib/ipc/types'
import { entriesFromValues, Severity } from './spec'
import type { DiagnosticEntry, DiagnosticRow } from './spec'

// `noProfile` is the expected state before a save is chosen (spec 3.5, Decision 8), so it is
// information and not an alarm. A section that didn't read is the one thing worth alarming.
const table: Record<SearchDiagnostic, DiagnosticRow> = {
  [SearchDiagnostic.NoProfile]: {
    severity: Severity.Info,
    title: 'search.diagnostics.noProfileTitle',
    body: 'search.diagnostics.noProfile',
  },
  [SearchDiagnostic.NoCatalog]: {
    severity: Severity.Info,
    title: 'search.diagnostics.noCatalogTitle',
    body: 'search.diagnostics.noCatalog',
  },
  [SearchDiagnostic.NoWiki]: {
    severity: Severity.Info,
    title: 'search.diagnostics.noWikiTitle',
    body: 'search.diagnostics.noWiki',
  },
  [SearchDiagnostic.NoAchievementSection]: {
    severity: Severity.Warning,
    title: 'search.diagnostics.noAchievementSectionTitle',
    body: 'search.diagnostics.noAchievementSection',
  },
  [SearchDiagnostic.NoCollectionSection]: {
    severity: Severity.Warning,
    title: 'search.diagnostics.noCollectionSectionTitle',
    body: 'search.diagnostics.noCollectionSection',
  },
}

export const searchEntries = (
  diagnostics: SearchDiagnostic[],
): DiagnosticEntry[] => entriesFromValues(diagnostics, table)
