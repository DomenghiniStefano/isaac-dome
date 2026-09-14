import type { RunView, RunsView } from '../types'

// A handful of runs for the Run screen in the browser: the archive is a database on the
// machine, so the fixtures are the only way to draw this list without the app.
//
// Shaped to cover what the screen has to survive rather than to look full: the watched
// launch and an online session, a win, a death, an abandonment and a run still open, a run
// whose character was never named, and an item with no name — the game not installed.
const item = (id: number, name: string | null) => ({ id, name })

const runs: RunView[] = [
  {
    source: { kind: 'live' },
    ordinal: 2,
    character: 'Cain',
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

// Live, with the case a real machine shows only while a Tainted character is being played:
// the log writes "Cain" and the game calls two characters that, so the answer holds both and
// says so. Inventing a single form here would make the screen look decided.
export const liveAnswer = () => ({
  run: runs[0],
  opens: [
    {
      character: 2,
      characterName: 'Cain',
      column: 'momsHeart' as const,
      level: 'base' as const,
      achievements: [
        {
          kind: 'known' as const,
          id: 19,
          text: 'The Family Man',
          condition: null,
          iconUrl: null,
        },
      ],
    },
    {
      character: 23,
      characterName: 'Tainted Cain',
      column: 'momsHeart' as const,
      level: 'base' as const,
      achievements: [
        {
          kind: 'known' as const,
          id: 509,
          text: 'The Fettered',
          condition: null,
          iconUrl: null,
        },
      ],
    },
  ],
  diagnostics: [
    { kind: 'ambiguousCharacter' as const, name: 'Cain', forms: 2 },
  ],
})
