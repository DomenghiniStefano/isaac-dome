import { describe, expect, it } from 'vitest'
import { completionMatrix } from '@/lib/ipc/fixtures/completion'
import { CellLevel, type Cell, type MarksMatrix } from '@/lib/ipc/types'
import {
  CellStatus,
  MatrixGroup,
  TallyTone,
  cellReading,
  columnTallies,
  completionKpis,
  matrixGroups,
  rowTally,
  tallyColumns,
} from './completionView'

// The reference profile of DESIGN-BRIEF.md §5.4. The expected values below are counted on
// the digit strings of the export's completion.json — the fixture's own data, read with a
// different tool — and never taken from this module's output. The four that the brief and
// the Kit page already printed (368 readable, 166 with a level, 40 unreadable, 408 cells)
// come out of that count unchanged, which is what says the count is right.
// A cell as the IPC sends it. `bits` is the value a real cell in that state carries and
// nothing in this module reads it: since B21 the mask is decoded in `ipc::marks::cell_at`,
// and what it decided is `level` and `online`.
const wire = (bits: number, level: CellLevel, online = false): Cell => ({
  kind: 'known',
  bits,
  level,
  online,
})
const NEVER = wire(0, CellLevel.Empty)

const reference = completionMatrix()

const row = (name: string) => {
  const found = reference.characters.find((r) => r.character === name)
  if (!found) throw new Error(`no row ${name}`)
  return found
}

describe('completionKpis on the reference profile', () => {
  it('states the totals §5.4 printed, split in two, and no percentage', () => {
    expect(completionKpis(reference)).toEqual({
      normal: 166,
      hard: 152,
      readable: 368,
      completeCharacters: 2,
      characters: 34,
    })
  })
})

describe('rowTally', () => {
  // B22: a mark taken on hard counts as taken on normal too, so `normal` is the cells with
  // either level and `hard` the ones with the second. The file corroborates it rather than
  // only the product rule: a cell goes 1 → 2, so a bare 2 is the normal mark overwritten
  // and not a hard mark taken by someone who never took the normal one (B58, 2026-09-17).
  it('calls a row complete only when every readable cell is hard', () => {
    // Magdalene is hard in all twelve: 12/12 · 12/12.
    expect(rowTally(row('Magdalene'))).toEqual({
      normal: 12,
      hard: 12,
      readable: 12,
      complete: true,
    })
  })

  it('does not call a row complete that is short of one hard mark', () => {
    // Isaac's last cell is a bare 1: every boss has a level, one of them only the first.
    // Under the old single count this row read 12/12 and complete; it is not.
    expect(rowTally(row('Isaac'))).toEqual({
      normal: 12,
      hard: 11,
      readable: 12,
      complete: false,
    })
  })

  it('keeps unreadable cells out of both numerators and the denominator', () => {
    expect(rowTally(row('The Forgotten'))).toEqual({
      normal: 9,
      hard: 8,
      readable: 10,
      complete: false,
    })
  })

  it('counts an untouched row as zero of its readable cells', () => {
    expect(rowTally(row('T. Magdalene'))).toEqual({
      normal: 0,
      hard: 0,
      readable: 10,
      complete: false,
    })
  })

  it('reads a single hard cell as one on both counts, never as zero and one', () => {
    // B22's own acceptance sentence. A bare 2 is hard, and hard is also normal.
    const cells: Cell[] = [
      wire(2, CellLevel.Hard),
      ...Array.from({ length: 11 }, (): Cell => NEVER),
    ]
    expect(rowTally({ ...row('Isaac'), cells })).toEqual({
      normal: 1,
      hard: 1,
      readable: 12,
      complete: false,
    })
  })
})

describe('the two counts never cross', () => {
  // The property that makes the pair readable at a glance: hard ≤ normal ≤ readable, on
  // every row and every column of the reference. It holds by construction — bit 1 implies
  // "has a level" — and it is the assertion that catches the two being swapped.
  it('holds hard ≤ normal ≤ readable on every row and column', () => {
    const tallies = [
      ...reference.characters.map(rowTally),
      ...columnTallies(reference),
    ]
    expect(tallies).toHaveLength(34 + 12)
    tallies.forEach((t) => {
      expect(t.hard).toBeLessThanOrEqual(t.normal)
      expect(t.normal).toBeLessThanOrEqual(t.readable)
    })
    // Not a vacuous pass: the reference has to contain a row where the two differ, or the
    // property above would hold on any profile that never took a hard mark at all.
    expect(tallies.some((t) => t.hard < t.normal)).toBe(true)
  })
})

describe('tallyColumns', () => {
  // B22's "Done when": a row with twelve hard marks reads 12/12 · 12/12, and a row with a
  // bare 2 in one cell reads 1/12 · 1/12 — not 0 and 1. Each number carries the denominator
  // because the two columns are read apart: under its own heading, `hard` is not "the number
  // after the dot", it is how many bosses reached the second level.
  it('gives every readable cell to both columns when the row is hard everywhere', () => {
    expect(tallyColumns(rowTally(row('Magdalene')))).toEqual({
      normal: { value: 12, readable: 12, tone: TallyTone.Full },
      hard: { value: 12, readable: 12, tone: TallyTone.Full },
    })
  })

  it('is full at normal and short at hard one mark from the end', () => {
    expect(tallyColumns(rowTally(row('Isaac')))).toEqual({
      normal: { value: 12, readable: 12, tone: TallyTone.Full },
      hard: { value: 11, readable: 12, tone: TallyTone.Partial },
    })
  })

  it('counts a bare 2 in both columns', () => {
    const cells: Cell[] = row('Isaac').cells.map((_, i) =>
      i === 0 ? wire(2, CellLevel.Hard) : NEVER,
    )
    expect(tallyColumns(rowTally({ ...row('Isaac'), cells }))).toEqual({
      normal: { value: 1, readable: 12, tone: TallyTone.Partial },
      hard: { value: 1, readable: 12, tone: TallyTone.Partial },
    })
  })

  // 0 === 0 is not "full": a row we cannot read has to look unreadable, not finished.
  it('calls a row with nothing readable unreadable, never full', () => {
    const cells: Cell[] = row('Isaac').cells.map(() => ({
      kind: 'unknown' as const,
    }))
    expect(tallyColumns(rowTally({ ...row('Isaac'), cells }))).toEqual({
      normal: { value: 0, readable: 0, tone: TallyTone.Unreadable },
      hard: { value: 0, readable: 0, tone: TallyTone.Unreadable },
    })
  })
})

describe('columnTallies', () => {
  it('counts each boss over the characters whose cell is readable', () => {
    const tallies = columnTallies(reference)
    expect(tallies).toHaveLength(12)
    // Mom's Heart: all 17 base characters, and 4 of the 17 Tainted.
    expect(tallies[0]).toEqual({
      normal: 21,
      hard: 21,
      readable: 34,
      complete: false,
    })
    // Greed, where the two numbers are furthest apart and the reason for splitting them is
    // visible: bit 1 there is Ultra Greedier, not a harder run of the same mode.
    expect(tallies[7]).toEqual({
      normal: 16,
      hard: 5,
      readable: 34,
      complete: false,
    })
    // The Beast: readable for the 14 originals only.
    expect(tallies[11]).toEqual({
      normal: 5,
      hard: 4,
      readable: 14,
      complete: false,
    })
  })

  it('never calls a column with nothing readable complete', () => {
    const unread: MarksMatrix = {
      ...reference,
      characters: reference.characters.map((r) => ({
        ...r,
        cells: r.cells.map(() => ({ kind: 'unknown' as const })),
      })),
    }
    expect(columnTallies(unread)[0]).toEqual({
      normal: 0,
      hard: 0,
      readable: 0,
      complete: false,
    })
  })
})

describe('matrixGroups', () => {
  it('splits base and Tainted, each saying what it cannot read', () => {
    const [base, tainted] = matrixGroups(reference)
    expect(base).toMatchObject({
      group: MatrixGroup.Base,
      first: 'Isaac',
      last: 'Jacob & Esau',
      unknown: 6,
    })
    expect(base?.rows).toHaveLength(17)
    expect(tainted).toMatchObject({
      group: MatrixGroup.Tainted,
      first: 'T. Isaac',
      last: 'T. Jacob & Esau',
      unknown: 34,
    })
    expect(tainted?.rows[0]?.index).toBe(17)
    expect(tainted?.rows[0]?.tally).toEqual({
      normal: 1,
      hard: 1,
      readable: 10,
      complete: false,
    })
  })

  it('carries both counts on the group header too', () => {
    const [base] = matrixGroups(reference)
    // 151 + 15 = 166 and 138 + 14 = 152 across the two groups, which is the totals above.
    expect(base?.tally).toMatchObject({
      normal: 151,
      hard: 138,
      readable: 198,
    })
  })
})

describe('cellReading', () => {
  // `Both` is gone (B22): a cell with bit 1 is hard, and hard is also normal, so "normal
  // and hard" was never a third thing to say — it was the same cell said twice. Which of
  // the two a mask means is settled in Rust now (`crates/ipc/tests/marks.rs`); what is
  // pinned here is that the level arrives intact as a status.
  it('reads every level the IPC can send as its own status', () => {
    expect(cellReading(NEVER)).toEqual({
      status: CellStatus.Empty,
      online: false,
    })
    expect(cellReading(wire(1, CellLevel.Normal))).toEqual({
      status: CellStatus.Normal,
      online: false,
    })
    expect(cellReading(wire(2, CellLevel.Hard))).toEqual({
      status: CellStatus.Hard,
      online: false,
    })
    expect(cellReading(wire(3, CellLevel.Hard))).toEqual({
      status: CellStatus.Hard,
      online: false,
    })
  })

  // The online win is not a level: it says where a mark was taken, not how high it is. It
  // travelled here as `third` while its meaning was unsettled; the meaning was measured on
  // 2026-09-12 and the name followed only on 2026-09-20.
  it('carries the online win beside the level, never as one', () => {
    expect(cellReading(wire(4, CellLevel.Empty, true))).toEqual({
      status: CellStatus.Empty,
      online: true,
    })
    expect(cellReading(wire(5, CellLevel.Normal, true))).toEqual({
      status: CellStatus.Normal,
      online: true,
    })
    expect(cellReading(wire(7, CellLevel.Hard, true))).toEqual({
      status: CellStatus.Hard,
      online: true,
    })
  })

  it('keeps what it cannot read, and what it should not see, apart', () => {
    expect(cellReading({ kind: 'unknown' })).toEqual({
      status: CellStatus.Unknown,
      online: false,
    })
    expect(cellReading({ kind: 'unexpected', value: 49 })).toEqual({
      status: CellStatus.Unexpected,
      online: false,
    })
  })
})

// B21, the same probe as `markVisual`'s: the status comes from the level the IPC read.
// The two files each carried their own copy of the bit rules, which is the duplication
// this entry set out to end.
describe('cellReading reads the reading and not the mask', () => {
  it('takes the status from the level', () => {
    expect(
      cellReading({
        kind: 'known',
        bits: 0,
        level: CellLevel.Hard,
        online: false,
      }),
    ).toEqual({ status: CellStatus.Hard, online: false })
  })

  it('takes the online win from its own field', () => {
    expect(
      cellReading({
        kind: 'known',
        bits: 0,
        level: CellLevel.Empty,
        online: true,
      }),
    ).toEqual({ status: CellStatus.Empty, online: true })
  })
})
