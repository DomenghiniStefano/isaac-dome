import { entriesFrom, Severity } from './spec'
import type { DiagnosticEntry, DiagnosticRow } from './spec'
import type { FloorDiagnostic } from '@/lib/ipc/types'

// Why the grid cannot answer, and how loudly. An empty grid is the ordinary first state of
// this screen, so it is a note. The last two are the screen being handed something that is not
// a floor, or rules that did not load, and neither is the user's doing.
//
// **The missing start room is `null`, which means the screen says it elsewhere.** It was an
// alert — a full-width box, above everything — for a sentence that qualifies one of three
// answers and asks for one cell. It is a mark beside the target switch now, with the sentence
// a hover away, the same shape every other explanation in this app takes. It stays a
// diagnostic and is still Rust's to raise; what changed is only how loudly it is drawn.
const table: Record<FloorDiagnostic['kind'], DiagnosticRow | null> = {
  gridEmpty: { severity: Severity.Note, body: 'floor.diagnostic.gridEmpty' },
  noStartRoom: null,
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
