import { CellLevel, MarkColumnView, MarkLevelView } from '../types'
import type {
  Cell,
  EntityRef,
  LiveView,
  PickupView,
  RunAchievementView,
  RunFloorView,
  RunView,
  RunsView,
} from '../types'

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

// What was picked up, where from and on which floor: `floor` indexes the run's `floorDetails`.
const picked = (
  id: number,
  name: string | null,
  pool: string,
  floor: number | null,
): PickupView => ({ item: item(id, name), pool, floor })

const floor = (
  stage: number,
  stageType: number,
  name: string | null,
  rooms: number | null,
): RunFloorView => ({ stage, stageType, name, rooms })

// An achievement the run unlocked; `null` text is a catalog that does not know it.
const achievement = (id: number, text: string | null): RunAchievementView => ({
  id,
  text,
  iconUrl: null,
})

// An entity as the death line names it: `9.0` is a shot, and the spawner is who fired it.
const entity = (raw: string, name: string | null): EntityRef => ({
  raw,
  name,
  iconUrl: null,
  page: null,
})

const runs: RunView[] = [
  {
    source: { kind: 'live', id: 7, writtenUnix: 1_790_000_000 },
    ordinal: 2,
    character: 'Cain',
    characterId: 23,
    characterHeadUrl: null,
    seedWords: 'FYQ8 QQ8G',
    online: false,
    outcome: { kind: 'open' },
    floors: 3,
    floorDetails: [
      floor(1, 0, 'Basement I', 12),
      floor(2, 4, 'Downpour II', 14),
      floor(3, 0, 'Caves I', null),
    ],
    startingItems: [item(46, 'Paper Clip')],
    collected: [
      picked(105, 'The D6', 'treasure', 0),
      picked(225, 'Gimpy', 'devil', 1),
    ],
    passives: [item(225, 'Gimpy')],
    familiars: [],
    heldActive: item(105, 'The D6'),
    achievements: [],
  },
  {
    source: { kind: 'live', id: 7, writtenUnix: 1_790_000_000 },
    ordinal: 1,
    character: 'Judas',
    characterId: 3,
    characterHeadUrl: null,
    seedWords: 'AB12 CD34',
    online: false,
    outcome: {
      kind: 'died',
      killer: entity('9.0', 'Projectile'),
      spawner: entity('20.0', 'Monstro'),
    },
    floors: 2,
    floorDetails: [
      floor(1, 0, 'Basement I', 11),
      floor(2, 0, 'Basement II', 13),
    ],
    startingItems: [item(34, 'The Book of Belial')],
    collected: [picked(118, 'Brimstone', 'treasure', 1)],
    passives: [item(118, 'Brimstone')],
    familiars: [],
    heldActive: null,
    achievements: [achievement(19, 'Judas')],
  },
  {
    source: { kind: 'session', name: '09_12_2026__13_34_26' },
    ordinal: 2,
    character: 'Keeper',
    characterId: 14,
    characterHeadUrl: null,
    seedWords: 'ZZ99 XX11',
    online: true,
    outcome: { kind: 'won', ending: 'Mother' },
    floors: 1,
    floorDetails: [floor(1, 3, null, null)],
    startingItems: [],
    collected: [picked(999, null, 'craneGame', 0)],
    passives: [item(999, null)],
    familiars: [],
    heldActive: null,
    achievements: [achievement(457, null)],
  },
  {
    source: { kind: 'session', name: '09_12_2026__13_34_26' },
    ordinal: 1,
    character: null,
    characterId: null,
    characterHeadUrl: null,
    seedWords: 'QW34 ER56',
    online: true,
    outcome: { kind: 'abandoned' },
    floors: 0,
    floorDetails: [],
    startingItems: [],
    collected: [],
    passives: [],
    familiars: [],
    heldActive: null,
    achievements: [],
  },
  {
    source: { kind: 'session', name: '09_10_2026__08_00_00' },
    ordinal: 1,
    character: 'Cain',
    characterId: 2,
    characterHeadUrl: null,
    seedWords: 'MN78 OP90',
    online: true,
    outcome: { kind: 'won', ending: 'The Beast' },
    floors: 1,
    floorDetails: [floor(4, 4, 'Mines II', null)],
    startingItems: [item(46, 'Paper Clip')],
    collected: [
      picked(118, 'Brimstone', 'boss', null),
      picked(105, 'The D6', 'shop', 0),
    ],
    passives: [item(118, 'Brimstone')],
    familiars: [],
    heldActive: item(105, 'The D6'),
    achievements: [achievement(491, 'Dead God')],
  },
  // An older launch read before the app kept dates: the row says so instead of inventing one.
  {
    source: { kind: 'launch', id: 3, writtenUnix: null },
    ordinal: 1,
    character: 'Isaac',
    characterId: 0,
    characterHeadUrl: null,
    seedWords: 'KLMN 5555',
    online: false,
    outcome: { kind: 'abandoned' },
    floors: 0,
    floorDetails: [],
    startingItems: [],
    collected: [],
    passives: [],
    familiars: [],
    heldActive: null,
    achievements: [],
  },
]

export const runsAnswer = (): RunsView => ({
  runs,
  totals: { runs: 6, won: 2, died: 1, abandoned: 2, open: 1 },
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
      secondLevel: null,
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
