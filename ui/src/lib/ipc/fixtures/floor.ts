import type { CommandArgs } from '../transport'
import { RoomKindView } from '../types'
import type { FloorCandidate, FloorView, RoomIconView } from '../types'

// `?floor=empty` answers an untouched grid; absent, the fixture solves whatever the screen
// sends, the way the backend does — with three made-up rules, so the development server can
// draw the screen without a Rust build.
export const FloorScenario = {
  Empty: 'empty',
  Solve: 'solve',
} as const
export type FloorScenario = (typeof FloorScenario)[keyof typeof FloorScenario]

const WIDTH = 13
const CELLS = 169

/**
 * **These are not the game's rules and are not meant to be.** `crates/floor` holds those, each
 * carrying the sentence it was read from; what a browser needs instead is the *shapes* the
 * screen has to draw — a cell one target wants, a cell two want, a cell all three want, and
 * each of the three ranks — because those are the shapes that cannot be judged by reading the
 * code. The quotes say so out loud, so a fixture answer is never mistaken for an answer.
 *
 * One rule each. They overlap on purpose: the whole question this screen had to answer again
 * is what a cell looks like when more than one target claims it.
 */
// A rank from the tests a place passes, best first: the first test it passes is its rank, and
// one that passes none ranks after all of them.
const rankedBy =
  (...tests: ((n: number) => boolean)[]) =>
  (n: number): number => {
    const at = tests.findIndex((test) => test(n))
    return at === -1 ? tests.length : at
  }

const RULES = [
  {
    target: 'secret',
    id: 'fixture-secret',
    // Four neighbours or more is the best place, three the next, two the least.
    allowed: (n: number) => n >= 2,
    rank: rankedBy(
      (n) => n >= 4,
      (n) => n >= 3,
    ),
  },
  {
    target: 'superSecret',
    id: 'fixture-super',
    allowed: (n: number) => n >= 1 && n <= 3,
    rank: rankedBy(
      (n) => n === 1,
      (n) => n === 2,
    ),
  },
  {
    target: 'ultraSecret',
    id: 'fixture-ultra',
    allowed: (n: number) => n >= 3,
    rank: (n: number) => (n >= 4 ? 0 : 1),
  },
] as const

const QUOTE =
  'Development fixture, not a rule of the game: it exists so the screen can be drawn in a browser.'
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
  const empty = cells
    .map((cell, index) => ({ cell: index, empty: cell === null }))
    .filter((c) => c.empty)
    .map((c) => ({
      cell: c.cell,
      neighbours: neighbours(c.cell).filter((n) => cells[n] !== null).length,
    }))
  const solutions = RULES.map((rule) => {
    const candidates: FloorCandidate[] = empty
      .filter((c) => rule.allowed(c.neighbours))
      .map((c) => ({
        cell: c.cell,
        neighbours: c.neighbours,
        rank: rule.rank(c.neighbours),
        applied: [{ id: rule.id, quote: QUOTE, url: URL }],
      }))
      .sort((a, b) => a.rank - b.rank || a.cell - b.cell)
    return { target: rule.target, candidates, unresolved: [] }
  })
  return { solutions, painted, diagnostics: [] }
}

// The fourteen kinds with no picture at all. A browser has no game to crop one from, and that
// is the case worth having in front of us by default: it is what a machine without the game
// shows, and the screen has to be complete without a single icon.
export const roomIconsAnswer = (): RoomIconView[] =>
  Object.values(RoomKindView).map((kind) => ({ kind, iconUrl: null }))
