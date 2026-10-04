import { describe, expect, it } from 'vitest'
import spacing from '@/assets/theme/spacing.css?raw'
import { ListTable, TableColumns } from './columns'
import { ACTIONS_KEY, ColumnFold, ColumnWidthKind } from './gridColumn'

// The tables that keep every column at compact, each with its reason beside it.
const KEEPS_EVERY_COLUMN: ListTable[] = [
  // Live sits beside the run being played; its four columns are what the row is — the
  // achievement, the cell that opens it, how to get it, how much it opens.
  ListTable.Live,
]

describe('every list table', () => {
  for (const [name, columns] of Object.entries(TableColumns)) {
    describe(name, () => {
      it('names each column once, and never Actions — the table adds it', () => {
        const keys = columns.map((c) => c.key)
        expect(new Set(keys).size).toBe(keys.length)
        expect(keys).not.toContain(ACTIONS_KEY)
      })

      it('sizes every fixed column with a token that exists', () => {
        for (const column of columns) {
          if (column.width.kind !== ColumnWidthKind.Fixed) continue
          const token = column.width.class.replace(/^w-/, '--spacing-')
          expect(spacing).toContain(`${token}:`)
        }
      })

      it('folds something at compact, unless it is declared to keep all', () => {
        if (KEEPS_EVERY_COLUMN.includes(name as ListTable)) return
        expect(columns.some((c) => c.fold === ColumnFold.Compact)).toBe(true)
      })
    })
  }

  it('is at least one table, so the loop above is not empty', () => {
    expect(Object.keys(TableColumns).length).toBeGreaterThan(0)
  })
})
