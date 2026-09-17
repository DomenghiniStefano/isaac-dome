import { compact, first, last } from 'lodash-es'
import { assertNever } from '@/lib/assertNever'
import type { Cell, CharacterRow, MarksMatrix } from '@/lib/ipc/types'

// What a cell says, in the tooltip's words: the level it reached, and the two ways of not
// knowing. The unconfirmed bit travels beside the status, never as a level.
//
// There is no `Both` (B22). A mark taken on hard counts as taken on normal too, so "normal
// and hard" was never a third thing a cell could say — it was the same cell said twice, and
// it made a bare 2 read as "hard, normal missing". The file agrees rather than merely
// permitting it: a cell goes 1 → 2, so a bare 2 is the normal mark *overwritten* (B58,
// 2026-09-17). What bit 1 means beyond "the second level" is unmeasured outside Greed,
// which is why the name here stops at the level.
export const CellStatus = {
  Empty: 'empty',
  Normal: 'normal',
  Hard: 'hard',
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

const levelledStatuses: CellStatus[] = [CellStatus.Normal, CellStatus.Hard]
const unreadableStatuses: CellStatus[] = [
  CellStatus.Unknown,
  CellStatus.Unexpected,
]

// Normal counts every cell that reached *a* level, because hard is also normal. Hard counts
// the ones that reached the second, so `hard <= normal` holds by construction.
const isNormal = (cell: Cell): boolean =>
  levelledStatuses.includes(cellReading(cell).status)
const isHard = (cell: Cell): boolean =>
  cellReading(cell).status === CellStatus.Hard
// Readable is what the save lets us read: a column not located and a value outside the mask
// both stay outside every denominator.
const isReadable = (cell: Cell): boolean =>
  !unreadableStatuses.includes(cellReading(cell).status)
const isUnknown = (cell: Cell): boolean =>
  cellReading(cell).status === CellStatus.Unknown

// Two counts over one denominator, never a percentage (§5.3): `hard <= normal <= readable`.
//
// `complete` is hard over readable and not normal over readable (B22): a row is done when
// every boss is done on hard, which is what the game's own widget means by a full row. The
// reference profile loses one complete character to that change — Isaac, whose last cell is
// a bare 1 — and losing it is the point.
export interface Tally {
  normal: number
  hard: number
  readable: number
  complete: boolean
}

const tallyOf = (cells: Cell[]): Tally => {
  const normal = cells.filter(isNormal).length
  const hard = cells.filter(isHard).length
  const readable = cells.filter(isReadable).length
  return { normal, hard, readable, complete: readable > 0 && hard === readable }
}

export const rowTally = (row: CharacterRow): Tally => tallyOf(row.cells)

export const columnTallies = (matrix: MarksMatrix): Tally[] =>
  matrix.bosses.map((_, column) =>
    tallyOf(compact(matrix.characters.map((row) => row.cells[column]))),
  )

// What a number is worth reading as: full, on its way, or not readable at all. `0/0` is the
// case this exists for — an equality alone would call an unreadable row finished.
export const TallyTone = {
  Full: 'full',
  Partial: 'partial',
  Unreadable: 'unreadable',
} as const
export type TallyTone = (typeof TallyTone)[keyof typeof TallyTone]

export interface TallyColumn {
  value: number
  readable: number
  tone: TallyTone
}

export interface TallyColumns {
  normal: TallyColumn
  hard: TallyColumn
}

const toneOf = (value: number, readable: number): TallyTone => {
  if (readable === 0) return TallyTone.Unreadable
  return value === readable ? TallyTone.Full : TallyTone.Partial
}

const column = (value: number, readable: number): TallyColumn => ({
  value,
  readable,
  tone: toneOf(value, readable),
})

// B22 item 4: two columns, each over the readable cells. The denominator is stated twice
// because the columns are read apart — under its own heading `hard` is not "the number after
// the dot", it is how many bosses reached the second level.
export const tallyColumns = (tally: Tally): TallyColumns => ({
  normal: column(tally.normal, tally.readable),
  hard: column(tally.hard, tally.readable),
})

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
  // The group's own cells, read the way a row's are: its header draws the same two columns,
  // so it carries the same Tally rather than three numbers that have to be reassembled.
  tally: Tally
  unknown: number
}

const groupView = (matrix: MarksMatrix, group: MatrixGroup): GroupView => {
  const tainted = group === MatrixGroup.Tainted
  const rows = matrix.characters.flatMap((row, index) =>
    row.tainted === tainted ? [{ row, index, tally: rowTally(row) }] : [],
  )
  const cells = rows.flatMap((r) => r.row.cells)
  return {
    group,
    rows,
    first: first(rows)?.row.character ?? '',
    last: last(rows)?.row.character ?? '',
    tally: tallyOf(cells),
    unknown: cells.filter(isUnknown).length,
  }
}

// Base, then Tainted. A group with no row isn't drawn.
export const matrixGroups = (matrix: MarksMatrix): GroupView[] =>
  [MatrixGroup.Base, MatrixGroup.Tainted]
    .map((group) => groupView(matrix, group))
    .filter((g) => g.rows.length > 0)

// The strip splits the same way the rows do. `both` is gone with `CellStatus.Both`: under
// B22 a hard cell is also a normal one, so "cells with both levels" counted the overlap of
// a set with its own superset — a number that could only ever be `hard` minus the bare 2s,
// which is a fact about how a profile was played and not about its progress.
//
// The unreadable count left with B23, and it is not lost: it is a gap in our own tables and
// not a fact about the player, so it belongs where it happens — every group header prints
// its own, every unreadable cell says so, and `readable` still speaks for the alert above
// the grid when there is nothing at all to read.
export interface CompletionKpis {
  normal: number
  hard: number
  readable: number
  completeCharacters: number
  characters: number
}

export const completionKpis = (matrix: MarksMatrix): CompletionKpis => {
  const tally = tallyOf(matrix.characters.flatMap((r) => r.cells))
  return {
    normal: tally.normal,
    hard: tally.hard,
    readable: tally.readable,
    completeCharacters: matrix.characters.filter((r) => rowTally(r).complete)
      .length,
    characters: matrix.characters.length,
  }
}
