import { ChallengesDiagnostic } from '@/lib/ipc/types'
import { entriesFromValues, Severity } from './spec'
import type { DiagnosticEntry, DiagnosticRow } from './spec'

// Without the game there is no list at all, so that one is an alarm and not a note — the
// screen is empty and has to say why. An unread section 7 takes the states away and leaves the
// rows; a missing wiki takes the conditions away and leaves both.
const table: Record<ChallengesDiagnostic, DiagnosticRow> = {
  [ChallengesDiagnostic.NoCatalog]: {
    severity: Severity.Warning,
    title: 'challenges.diagnostics.noCatalogTitle',
    body: 'challenges.diagnostics.noCatalog',
  },
  [ChallengesDiagnostic.NoChallengesSection]: {
    severity: Severity.Warning,
    title: 'challenges.diagnostics.noChallengesSectionTitle',
    body: 'challenges.diagnostics.noChallengesSection',
  },
  [ChallengesDiagnostic.NoAchievementSection]: {
    severity: Severity.Warning,
    title: 'challenges.diagnostics.noAchievementSectionTitle',
    body: 'challenges.diagnostics.noAchievementSection',
  },
  [ChallengesDiagnostic.NoWiki]: {
    severity: Severity.Note,
    body: 'challenges.diagnostics.noWiki',
  },
}

export const challengeEntries = (
  diagnostics: ChallengesDiagnostic[],
): DiagnosticEntry[] => entriesFromValues(diagnostics, table)
