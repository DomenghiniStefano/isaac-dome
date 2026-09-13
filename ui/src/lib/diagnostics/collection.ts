import { entriesFrom, Severity } from './spec'
import type { DiagnosticEntry, DiagnosticRow } from './spec'
import type { CollectionDiagnostic } from '@/lib/ipc/types'

// A catalog newer than the save coexists with real rows: a note. The two unread sections
// stand in for the content, so they are alarms.
const table: Record<CollectionDiagnostic['kind'], DiagnosticRow> = {
  noCatalog: {
    severity: Severity.Info,
    title: 'collection.diagnostics.noCatalogTitle',
    body: 'collection.diagnostics.noCatalog',
  },
  noCollectionSection: {
    severity: Severity.Warning,
    title: 'collection.diagnostics.noCollectionSectionTitle',
    body: 'collection.diagnostics.noCollectionSection',
  },
  noAchievementSection: {
    severity: Severity.Warning,
    title: 'collection.diagnostics.noAchievementSectionTitle',
    body: 'collection.diagnostics.noAchievementSection',
  },
  itemsBeyondSlots: {
    severity: Severity.Note,
    body: 'collection.diagnostics.itemsBeyondSlots',
  },
}

export const collectionEntries = (
  diagnostics: CollectionDiagnostic[],
): DiagnosticEntry[] => entriesFrom(diagnostics, table)
