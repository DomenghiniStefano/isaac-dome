import {
  CellLevel,
  SecondLevelView,
  type Cell,
  type CharacterRow,
  type MarksMatrix,
} from '../types'

// The file's three blocks, as CharacterGroup serializes them.
const Group = {
  Original: 'original',
  Forgotten: 'forgotten',
  Later: 'later',
} as const
type Group = (typeof Group)[keyof typeof Group]

const bosses = [
  "Mom's Heart",
  'Isaac',
  'Satan',
  'Boss Rush',
  'Blue Baby',
  'The Lamb',
  'Mega Satan',
  'Greed',
  'Hush',
  'Delirium',
  'Mother',
  'The Beast',
]

// DESIGN-BRIEF.md §5.4's reference profile (Rep+ slot 1), as the design export's
// completion.json carries it: one digit per boss in the order above, `?` where the column
// isn't located for that character. Its totals: 166 with a level and 152 hard out of 368
// readable, 40 unknown.
//
// Those totals are counted on the digit strings below and describe **this fixture**, which
// is a fixture of its era. They are not the layout's current count: since 2026-09-20 the
// tables locate Mother for the last twenty rows, so a real profile now has 20 unknown cells
// and not 40. Turning these `?` into digits would be inventing marks the reference profile
// was never measured to have.
const rows: [string, string, Group, boolean][] = [
  ['Isaac', '773223227331', Group.Original, false],
  ['Magdalene', '737333333333', Group.Original, false],
  ['Cain', '333323323233', Group.Original, false],
  ['Judas', '333233333300', Group.Original, false],
  ['Blue Baby', '333230013302', Group.Original, false],
  ['Eve', '333333313300', Group.Original, false],
  ['Samson', '735000003000', Group.Original, false],
  ['Azazel', '332332312200', Group.Original, false],
  ['Lazarus', '332233013300', Group.Original, false],
  ['Eden', '737332312302', Group.Original, false],
  ['The Lost', '333330013300', Group.Original, false],
  ['Lilith', '330530010000', Group.Original, false],
  ['Keeper', '333330030000', Group.Original, false],
  ['Apollyon', '732232213300', Group.Original, false],
  ['The Forgotten', '7272220122??', Group.Forgotten, false],
  ['Bethany', '3230233133??', Group.Later, false],
  ['Jacob & Esau', '3033030003??', Group.Later, false],
  ['T. Isaac', '2000000000??', Group.Later, true],
  ['T. Magdalene', '0000000000??', Group.Later, true],
  ['T. Cain', '3300300503??', Group.Later, true],
  ['T. Judas', '0000000000??', Group.Later, true],
  ['T. Blue Baby', '0000000000??', Group.Later, true],
  ['T. Eve', '0000000000??', Group.Later, true],
  ['T. Samson', '0000000000??', Group.Later, true],
  ['T. Azazel', '3303300033??', Group.Later, true],
  ['T. Lazarus', '0000000000??', Group.Later, true],
  ['T. Eden', '7300300000??', Group.Later, true],
  ['T. The Lost', '0000000000??', Group.Later, true],
  ['T. Lilith', '0000000000??', Group.Later, true],
  ['T. Keeper', '0000000000??', Group.Later, true],
  ['T. Apollyon', '0000000000??', Group.Later, true],
  ['T. Forgotten', '0000000000??', Group.Later, true],
  ['T. Bethany', '0000000000??', Group.Later, true],
  ['T. Jacob & Esau', '0000000000??', Group.Later, true],
]

// The digits are the raw counter values the export was read from, so this is the one place
// in the frontend that still turns a mask into a reading — and it does it **because it is
// standing in for the backend**, not because a screen needs to. It mirrors
// `ipc::marks::cell_at`: bit 1 decides the level whether or not bit 0 stands with it (a
// bare 2 is the first level overwritten, B58), and bit 2 is the online win, which is not a
// level. If the two ever disagree the fixture is the one that is wrong.
const cellOf = (digit: string): Cell => {
  if (digit === '?') return { kind: 'unknown' }
  const bits = Number(digit)
  const level =
    (bits & 2) !== 0
      ? CellLevel.Hard
      : (bits & 1) !== 0
        ? CellLevel.Normal
        : CellLevel.Empty
  return { kind: 'known', bits, level, online: (bits & 4) !== 0 }
}

// The two counts as the backend makes them (`ipc::totals_of`), read off the level the cell
// now carries: a level is any level reached, and hard is a subset of it, because a mark
// taken on hard counts as taken on normal too.
const hasLevel = (cell: Cell): boolean =>
  cell.kind === 'known' && cell.level !== CellLevel.Empty
const isHard = (cell: Cell): boolean =>
  cell.kind === 'known' && cell.level === CellLevel.Hard

const noArt = { normalUrl: null, hardUrl: null }

// The matrix the `completion` command answers for that profile. Every drawing is null: the
// app cuts its sprites from the user's own copy of the game at runtime, and the development
// server has no copy to cut from — which is also what a machine without the game receives.
export const completionMatrix = (): MarksMatrix => {
  const characters: CharacterRow[] = rows.map(
    ([character, digits, group, tainted]) => ({
      character,
      group,
      tainted,
      cells: [...digits].map(cellOf),
      headUrl: null,
    }),
  )
  const cells = characters.flatMap((r) => r.cells)
  const count = (keep: (cell: Cell) => boolean): number =>
    cells.filter(keep).length
  return {
    characters,
    bosses,
    art: bosses.map(() => noArt),
    // As `ipc::second_level` names them: Greed's second level is Ultra Greedier, every
    // other column's is hard.
    secondLevels: bosses.map((boss) =>
      boss === 'Greed' ? SecondLevelView.UltraGreedier : SecondLevelView.Hard,
    ),
    totals: {
      cells: cells.length,
      readable: count((c) => c.kind === 'known'),
      unknown: count((c) => c.kind === 'unknown'),
      unexpected: count((c) => c.kind === 'unexpected'),
      normal: count(hasLevel),
      hard: count(isHard),
    },
    // Null for the same reason every other drawing here is: the emblem is composed by the
    // icon protocol out of the game's own sheet, and the development server has no copy to
    // cut from — which is also what a machine without the game receives. The band then
    // draws no frame at all and the text takes the width, so this is the shape the browser
    // can check; the one with the picture is only visible in the app.
    widgetUrl: null,
  }
}
