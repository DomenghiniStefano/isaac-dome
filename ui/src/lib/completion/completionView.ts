import { compact, first, last } from 'lodash-es'
import { assertNever } from '@/lib/assertNever'
import type { Cell, CharacterRow, MarksMatrix } from '@/lib/ipc/types'

// What a cell says, in the tooltip's words: the two levels apart and together, and the two
// ways of not knowing. The unconfirmed bit travels beside the status, never as a level.
export const CellStatus = {
  Empty: 'empty',
  Normal: 'normal',
  Hard: 'hard',
  Both: 'both',
  Unknown: 'unknown',
  Unexpected: 'unexpected',
} as const
export type CellStatus = (typeof CellStatus)[keyof typeof CellStatus]

export interface CellReading {
  status: CellStatus
  third: boolean
}

// The same bit rules as markVisual (DESIGN-BRIEF.md §5.3): bit 0 the normal mark, bit 1 the
// hard one, bit 2 unconfirmed, anything higher unexpected.
const Bit = { Normal: 1, Hard: 2, Third: 4 } as const
const knownBits = Bit.Normal | Bit.Hard | Bit.Third

const levelStatus = (normal: boolean, hard: boolean): CellStatus => {
  if (normal && hard) return CellStatus.Both
  if (hard) return CellStatus.Hard
  if (normal) return CellStatus.Normal
  return CellStatus.Empty
}

export const cellReading = (cell: Cell): CellReading => {
  switch (cell.kind) {
    case 'unknown':
      return { status: CellStatus.Unknown, third: false }
    case 'unexpected':
      return { status: CellStatus.Unexpected, third: false }
    case 'known': {
      const { bits } = cell
      if ((bits & ~knownBits) !== 0)
        return { status: CellStatus.Unexpected, third: false }
      return {
        status: levelStatus((bits & Bit.Normal) !== 0, (bits & Bit.Hard) !== 0),
        third: (bits & Bit.Third) !== 0,
      }
    }
    default:
      return assertNever(cell)
  }
}

const startedStatuses: CellStatus[] = [
  CellStatus.Normal,
  CellStatus.Hard,
  CellStatus.Both,
]
const unreadableStatuses: CellStatus[] = [
  CellStatus.Unknown,
  CellStatus.Unexpected,
]

const isStarted = (cell: Cell): boolean =>
  startedStatuses.includes(cellReading(cell).status)
// Readable is what the save lets us read: a column not located and a value outside the mask
// both stay outside every denominator.
const isReadable = (cell: Cell): boolean =>
  !unreadableStatuses.includes(cellReading(cell).status)
const isUnknown = (cell: Cell): boolean =>
  cellReading(cell).status === CellStatus.Unknown

// Started over readable: a count with its denominator, never a percentage (§5.3).
export interface Tally {
  started: number
  readable: number
  complete: boolean
}

const tallyOf = (cells: Cell[]): Tally => {
  const started = cells.filter(isStarted).length
  const readable = cells.filter(isReadable).length
  return { started, readable, complete: readable > 0 && started === readable }
}

export const rowTally = (row: CharacterRow): Tally => tallyOf(row.cells)

export const columnTallies = (matrix: MarksMatrix): Tally[] =>
  matrix.bosses.map((_, column) =>
    tallyOf(compact(matrix.characters.map((row) => row.cells[column]))),
  )

// The two groups a player thinks in (Schermate.dc.html), read from `tainted` rather than
// from a row index: the file's own three blocks stay in `group`.
export const MatrixGroup = { Base: 'base', Tainted: 'tainted' } as const
export type MatrixGroup = (typeof MatrixGroup)[keyof typeof MatrixGroup]

export interface GroupRow {
  row: CharacterRow
  // The row's position in the matrix, for alternating backgrounds and keys.
  index: number
  tally: Tally
}

export interface GroupView {
  group: MatrixGroup
  rows: GroupRow[]
  first: string
  last: string
  started: number
  readable: number
  unknown: number
}

const groupView = (matrix: MarksMatrix, group: MatrixGroup): GroupView => {
  const tainted = group === MatrixGroup.Tainted
  const rows = matrix.characters.flatMap((row, index) =>
    row.tainted === tainted ? [{ row, index, tally: rowTally(row) }] : [],
  )
  const cells = rows.flatMap((r) => r.row.cells)
  const tally = tallyOf(cells)
  return {
    group,
    rows,
    first: first(rows)?.row.character ?? '',
    last: last(rows)?.row.character ?? '',
    started: tally.started,
    readable: tally.readable,
    unknown: cells.filter(isUnknown).length,
  }
}

// Base, then Tainted. A group with no row isn't drawn.
export const matrixGroups = (matrix: MarksMatrix): GroupView[] =>
  [MatrixGroup.Base, MatrixGroup.Tainted]
    .map((group) => groupView(matrix, group))
    .filter((g) => g.rows.length > 0)

export interface CompletionKpis {
  started: number
  readable: number
  // Cells with both the normal and the hard level.
  both: number
  completeCharacters: number
  characters: number
  unknown: number
  cells: number
}

export const completionKpis = (matrix: MarksMatrix): CompletionKpis => {
  const cells = matrix.characters.flatMap((r) => r.cells)
  const tally = tallyOf(cells)
  return {
    started: tally.started,
    readable: tally.readable,
    both: cells.filter((c) => cellReading(c).status === CellStatus.Both).length,
    completeCharacters: matrix.characters.filter((r) => rowTally(r).complete)
      .length,
    characters: matrix.characters.length,
    unknown: cells.filter(isUnknown).length,
    cells: cells.length,
  }
}
