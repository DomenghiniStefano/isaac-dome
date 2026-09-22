import { describe, expect, it } from 'vitest'
import utilities from '@/assets/utilities.css?raw'
import { FoldingTable, TableTracks } from './tables'

// The body of one `@utility grid-cols-<name>` declaration. `grid-cols-collection` and
// `grid-cols-collection-narrow` are told apart by the space before the brace: the narrow one
// has `-narrow` where this pattern wants that space.
const template = (name: string): string => {
  const found = utilities.match(
    new RegExp(String.raw`@utility grid-cols-${name} \{([^}]*)\}`),
  )
  if (found === null) throw new Error(`no @utility grid-cols-${name}`)
  return found[1]
}

// Every track in these templates is either a token or a `minmax()`. Counting them is the whole
// point: a narrow set that is not shorter than its full set is a pair that was never made.
const tracks = (body: string): number =>
  (body.match(/var\(--[^)]*\)|minmax\([^)]*\)/g) ?? []).length

// The pattern of `thresholds.test.ts`: the decision lives in the CSS, where the browser reads
// it, and in TypeScript, where a reader and a test can name it. If the two drift, a column
// leaves a template while its cell stays — which draws as a styling bug and is a counting one
// (spec 3.13a §7).
describe('a folding table', () => {
  it('declares both templates', () => {
    for (const name of Object.values(FoldingTable)) {
      expect(() => template(name)).not.toThrow()
      expect(() => template(`${name}-narrow`)).not.toThrow()
    }
  })

  it('declares the number of tracks the record names', () => {
    for (const name of Object.values(FoldingTable)) {
      expect(tracks(template(name))).toBe(TableTracks[name].full)
      expect(tracks(template(`${name}-narrow`))).toBe(TableTracks[name].narrow)
    }
  })

  it('is narrower when narrow, which is the whole claim', () => {
    for (const name of Object.values(FoldingTable))
      expect(TableTracks[name].narrow).toBeLessThan(TableTracks[name].full)
  })
})
