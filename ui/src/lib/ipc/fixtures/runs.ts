import { CellLevel, MarkColumnView, MarkLevelView } from '../types'
import type { Cell, LiveView, RunView, RunsView } from '../types'

// A cell with no mark at all, the value every one of Tainted Cain's holds below.
const NEVER: Cell = {
  kind: 'known',
  bits: 0,
  level: CellLevel.Empty,
  online: false,
}

// A handful of runs for the Run screen in the browser: the archive is a database on the
// machine, so the fixtures are the only way to draw this list without the app.
//
// Shaped to cover what the screen has to survive rather than to look full: the watched
// launch and an online session, a win, a death, an abandonment and a run still open, a run
// whose character was never named, and an item with no name — the game not installed.
// The pack's own art, so the fixtures draw the same kind of picture the app does.
const item = (id: number, name: string | null) => ({
  id,
  name,
  iconUrl: null,
})

const runs: RunView[] = [
  {
    source: { kind: 'live' },
    ordinal: 2,
    character: 'Cain',
    characterId: 23,
    seedWords: 'FYQ8 QQ8G',
    online: false,
    outcome: { kind: 'open' },
    floors: 3,
    startingItems: [item(46, 'Paper Clip')],
    collected: [item(105, 'The D6'), item(225, 'Gimpy')],
    heldActive: item(105, 'The D6'),
    achievements: [],
  },
  {
    source: { kind: 'live' },
    ordinal: 1,
    character: 'Judas',
    characterId: 3,
    seedWords: 'AB12 CD34',
    online: false,
    outcome: { kind: 'died', killer: 'Monstro' },
    floors: 5,
    startingItems: [item(34, 'The Book of Belial')],
    collected: [item(118, 'Brimstone')],
    heldActive: null,
    achievements: [19],
  },
  {
    source: { kind: 'session', name: '09_12_2026__13_34_26' },
    ordinal: 2,
    character: 'Keeper',
    characterId: 14,
    seedWords: 'ZZ99 XX11',
    online: true,
    outcome: { kind: 'won', ending: 'Mother' },
    floors: 12,
    startingItems: [],
    collected: [item(999, null)],
    heldActive: null,
    achievements: [457],
  },
  {
    source: { kind: 'session', name: '09_12_2026__13_34_26' },
    ordinal: 1,
    character: null,
    characterId: null,
    seedWords: 'QW34 ER56',
    online: true,
    outcome: { kind: 'abandoned' },
    floors: 1,
    startingItems: [],
    collected: [],
    heldActive: null,
    achievements: [],
  },
  {
    source: { kind: 'session', name: '09_10_2026__08_00_00' },
    ordinal: 1,
    character: 'Cain',
    characterId: 2,
    seedWords: 'MN78 OP90',
    online: true,
    outcome: { kind: 'won', ending: 'The Beast' },
    floors: 11,
    startingItems: [item(46, 'Paper Clip')],
    collected: [item(118, 'Brimstone'), item(105, 'The D6')],
    heldActive: item(105, 'The D6'),
    achievements: [491],
  },
]

export const runsAnswer = (): RunsView => ({
  runs,
  totals: { runs: 5, won: 2, died: 1, abandoned: 1, open: 1 },
  // One that changes how an item is named and nothing else: the list still draws, with an id
  // where a name would be.
  diagnostics: [{ kind: 'noCatalog' }],
})

// Live while a Tainted character is being played — the case that used to be ambiguous. The
// item line writes "Cain" for both forms, and since 2026-09-15 the log's `Initialized player`
// line states the id: this answer is what the screen shows once it knows, one form and no
// note about two.
export const liveAnswer = (): LiveView => ({
  run: runs[0],
  // One row: the log stated Subtype 23, so the screen knows it is the Tainted form. An empty
  // profile, so every cell is still to take — the state a player is in when this matters most.
  marks: {
    bosses: ["Mom's Heart", 'Isaac', 'Satan'],
    art: [{}, {}, {}].map(() => ({ normalUrl: null, hardUrl: null })),
    rows: [
      {
        character: 'Tainted Cain',
        headUrl: null,
        cells: [NEVER, NEVER, NEVER],
        missing: 3,
      },
    ],
  },
  opens: [
    {
      character: 23,
      characterName: 'Tainted Cain',
      column: MarkColumnView.MomsHeart,
      level: MarkLevelView.Base,
      achievements: [
        {
          achievement: {
            kind: 'known' as const,
            id: 509,
            text: 'The Fettered',
            condition: null,
            iconUrl: null,
          },
          fanOut: 0,
        },
      ],
    },
  ],
  // No ambiguity any more: the log stated Subtype 23, so only that form is offered. The
  // diagnostic stays in the table for a run folded before that line was read.
  diagnostics: [],
})
