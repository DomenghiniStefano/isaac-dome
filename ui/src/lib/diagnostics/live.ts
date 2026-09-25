import { entriesFrom, Severity } from './spec'
import type { DiagnosticEntry, DiagnosticRow } from './spec'
import type { LiveDiagnostic } from '@/lib/ipc/types'

// Why the screen cannot answer, and how loudly. None of them is an alarm: not playing is the
// ordinary state of a machine, and a name we cannot place is the game's own doing. The
// ambiguous one is an **info** rather than a note because it changes how the list below it
// must be read — two forms, both shown, neither chosen.
const table: Record<LiveDiagnostic['kind'], DiagnosticRow> = {
  noRun: { severity: Severity.Note, body: 'live.diagnostic.noRun' },
  characterNotNamed: {
    severity: Severity.Note,
    body: 'live.diagnostic.characterNotNamed',
  },
  unknownCharacter: {
    severity: Severity.Info,
    body: 'live.diagnostic.unknownCharacter',
  },
  ambiguousCharacter: {
    severity: Severity.Info,
    body: 'live.diagnostic.ambiguousCharacter',
  },
  noGraph: { severity: Severity.Info, body: 'live.diagnostic.noGraph' },
  noProfile: { severity: Severity.Info, body: 'live.diagnostic.noProfile' },
  // A save that would not read is the one of these that is not ordinary: it is a warning.
  saveUnreadable: {
    severity: Severity.Warning,
    body: 'live.diagnostic.saveUnreadable',
  },
}

export const liveEntries = (diagnostics: LiveDiagnostic[]): DiagnosticEntry[] =>
  entriesFrom(diagnostics, table)
