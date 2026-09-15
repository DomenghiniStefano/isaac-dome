import { entriesFrom, Severity } from './spec'
import type { DiagnosticEntry, DiagnosticRow } from './spec'
import type { FloorDiagnostic } from '@/lib/ipc/types'

// Why the grid cannot answer, and how loudly. An empty grid is the ordinary first state of
// this screen, so it is a note; a missing start room is an **info** because it changes how
// the list below must be read — one of the Super Secret rules is simply not being applied.
// The last two are the screen being handed something that is not a floor, or rules that did
// not load, and neither is the user's doing.
const table: Record<FloorDiagnostic['kind'], DiagnosticRow> = {
  gridEmpty: { severity: Severity.Note, body: 'floor.diagnostic.gridEmpty' },
  noStartRoom: {
    severity: Severity.Info,
    body: 'floor.diagnostic.noStartRoom',
  },
  rulesUnreadable: {
    severity: Severity.Warning,
    body: 'floor.diagnostic.rulesUnreadable',
  },
  gridMalformed: {
    severity: Severity.Warning,
    body: 'floor.diagnostic.gridMalformed',
  },
}

export const floorEntries = (
  diagnostics: FloorDiagnostic[],
): DiagnosticEntry[] => entriesFrom(diagnostics, table)
