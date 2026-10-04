import { describe, expect, it } from 'vitest'
import spacing from '@/assets/theme/spacing.css?raw'
import { factColumns } from '@/lib/wiki/factChips'
import { WikiCategory } from '@/router/routeTable'
import { ListTable, TableColumns, wikiColumns } from './columns'
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

// A category can have eight facts: at a fixed width each, the row is wider than any window and
// the name is left nothing. So the facts share what is left, and a long value truncates.
describe("the wiki's table", () => {
  const categories = Object.values(WikiCategory)
  const withFacts = categories.filter((c) => factColumns(c).length > 0)

  it('is checked on categories that have facts', () => {
    expect(withFacts.length).toBeGreaterThan(0)
  })

  it('never gives a fact a fixed width, in any category', () => {
    for (const category of withFacts) {
      const facts = wikiColumns(factColumns(category), true).filter((c) =>
        c.key.startsWith('fact-'),
      )
      expect(facts.length).toBe(factColumns(category).length)
      for (const fact of facts)
        expect(fact.width.kind, `${category} ${fact.key}`).toBe(
          ColumnWidthKind.Grow,
        )
    }
  })
})
