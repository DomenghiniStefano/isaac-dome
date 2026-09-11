import { describe, expect, it } from 'vitest'
import { completionMatrix } from '@/lib/ipc/fixtures/completion'
import type { MarksMatrix } from '@/lib/ipc/types'
import {
  CellStatus,
  MatrixGroup,
  cellReading,
  columnTallies,
  completionKpis,
  matrixGroups,
  rowTally,
} from './completionView'

// The reference profile of DESIGN-BRIEF.md §5.4. The expected values below come from the
// brief and the Kit page's tiles (166 of 368 started, 120 cells with both levels, 3 of 34
// complete characters, 40 of 408 unreadable), and the per-row and per-column ones are
// counted by hand on the digit strings of the export's completion.json.
const reference = completionMatrix(false)

const row = (name: string) => {
  const found = reference.characters.find((r) => r.character === name)
  if (!found) throw new Error(`no row ${name}`)
  return found
}

describe('completionKpis on the reference profile', () => {
  it('states the totals §5.4 printed, and no percentage', () => {
    expect(completionKpis(reference)).toEqual({
      started: 166,
      readable: 368,
      both: 120,
      completeCharacters: 3,
      characters: 34,
      unknown: 40,
      cells: 408,
    })
  })
})

describe('rowTally', () => {
  it('calls a row complete when every readable cell is started', () => {
    expect(rowTally(row('Isaac'))).toEqual({
      started: 12,
      readable: 12,
      complete: true,
    })
  })

  it('keeps unreadable cells out of the denominator', () => {
    expect(rowTally(row('The Forgotten'))).toEqual({
      started: 9,
      readable: 10,
      complete: false,
    })
  })

  it('counts an untouched row as zero of its readable cells', () => {
    expect(rowTally(row('T. Magdalene'))).toEqual({
      started: 0,
      readable: 10,
      complete: false,
    })
  })
})

describe('columnTallies', () => {
  it('counts each boss over the characters whose cell is readable', () => {
    const tallies = columnTallies(reference)
    expect(tallies).toHaveLength(12)
    // Mom's Heart: all 17 base characters, and 4 of the 17 Tainted.
    expect(tallies[0]).toEqual({ started: 21, readable: 34, complete: false })
    // The Beast: readable for the 14 originals only.
    expect(tallies[11]).toEqual({ started: 5, readable: 14, complete: false })
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
      started: 0,
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
      started: 1,
      readable: 10,
      complete: false,
    })
  })
})

describe('cellReading', () => {
  const known = (bits: number) => cellReading({ kind: 'known', bits })

  it('reads the two levels apart and together', () => {
    expect(known(0)).toEqual({ status: CellStatus.Empty, third: false })
    expect(known(1)).toEqual({ status: CellStatus.Normal, third: false })
    expect(known(2)).toEqual({ status: CellStatus.Hard, third: false })
    expect(known(3)).toEqual({ status: CellStatus.Both, third: false })
  })

  it('carries the unconfirmed bit beside the level, never as one', () => {
    expect(known(4)).toEqual({ status: CellStatus.Empty, third: true })
    expect(known(5)).toEqual({ status: CellStatus.Normal, third: true })
    expect(known(7)).toEqual({ status: CellStatus.Both, third: true })
  })

  it('keeps what it cannot read, and what it should not see, apart', () => {
    expect(known(8)).toEqual({ status: CellStatus.Unexpected, third: false })
    expect(cellReading({ kind: 'unknown' })).toEqual({
      status: CellStatus.Unknown,
      third: false,
    })
    expect(cellReading({ kind: 'unexpected', value: 49 })).toEqual({
      status: CellStatus.Unexpected,
      third: false,
    })
  })
})
