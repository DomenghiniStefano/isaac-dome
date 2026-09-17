import { entriesFrom, Severity } from './spec'
import type { DiagnosticEntry, DiagnosticRow } from './spec'
import type { ChallengesDiagnostic } from '@/lib/ipc/types'

// Without the game there is no list at all, so that one is an alarm and not a note — the
// screen is empty and has to say why. An unread section 7 takes the states away and leaves the
// rows; a missing wiki takes the conditions away and leaves both.
const table: Record<ChallengesDiagnostic['kind'], DiagnosticRow> = {
  noCatalog: {
    severity: Severity.Warning,
    title: 'challenges.diagnostics.noCatalogTitle',
    body: 'challenges.diagnostics.noCatalog',
  },
  noChallengesSection: {
    severity: Severity.Warning,
    title: 'challenges.diagnostics.noChallengesSectionTitle',
    body: 'challenges.diagnostics.noChallengesSection',
  },
  noAchievementSection: {
    severity: Severity.Warning,
    title: 'challenges.diagnostics.noAchievementSectionTitle',
    body: 'challenges.diagnostics.noAchievementSection',
  },
  noWiki: {
    severity: Severity.Note,
    body: 'challenges.diagnostics.noWiki',
  },
}

export const challengeEntries = (
  diagnostics: ChallengesDiagnostic[],
): DiagnosticEntry[] => entriesFrom(diagnostics, table)
