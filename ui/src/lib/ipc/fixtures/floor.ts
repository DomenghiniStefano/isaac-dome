import type { CommandArgs } from '../transport'
import type { FloorView, RoomKindView } from '../types'

// `?floor=empty` answers an untouched grid; absent, the fixture solves whatever the screen
// sends, the way the backend does — with one rule, so the development server can draw the
// screen without a Rust build.
export const FloorScenario = {
  Empty: 'empty',
  Solve: 'solve',
} as const
export type FloorScenario = (typeof FloorScenario)[keyof typeof FloorScenario]

const WIDTH = 13
const CELLS = 169

const QUOTE =
  'Secret Rooms are equally as likely to be in a valid location with 3 neighbors, as it is with 4 neighbors.'
const URL = 'https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room'

const neighbours = (cell: number): number[] => {
  const x = cell % WIDTH
  const out: number[] = []
  if (x > 0) out.push(cell - 1)
  if (x + 1 < WIDTH) out.push(cell + 1)
  if (cell - WIDTH >= 0) out.push(cell - WIDTH)
  if (cell + WIDTH < CELLS) out.push(cell + WIDTH)
  return out
}

export const floorAnswer = (
  scenario: FloorScenario,
  args?: CommandArgs,
): FloorView => {
  const cells = (args?.cells ?? []) as (RoomKindView | null)[]
  const painted = cells.filter((c) => c !== null).length
  if (scenario === FloorScenario.Empty || painted === 0) {
    return { solutions: [], painted: 0, diagnostics: [{ kind: 'gridEmpty' }] }
  }
  const candidates = cells
    .map((cell, index) => ({ cell: index, empty: cell === null }))
    .filter((c) => c.empty)
    .map((c) => ({
      cell: c.cell,
      neighbours: neighbours(c.cell).filter((n) => cells[n] !== null).length,
    }))
    .filter((c) => c.neighbours >= 2)
    .map((c) => ({
      cell: c.cell,
      neighbours: c.neighbours,
      rank: c.neighbours >= 3 ? 0 : 1,
      applied: [{ id: 'secret-neighbours', quote: QUOTE, url: URL }],
    }))
    .sort((a, b) => a.rank - b.rank || a.cell - b.cell)
  return {
    solutions: [{ target: 'secret', candidates, unresolved: [] }],
    painted,
    diagnostics: [],
  }
}
