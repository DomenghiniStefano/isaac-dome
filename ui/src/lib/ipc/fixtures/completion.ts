import type { Cell, CharacterRow, MarksMatrix } from '../types'
import { packHeadUrl, packMarkArt } from './art'

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

const cellOf = (digit: string): Cell =>
  digit === '?' ? { kind: 'unknown' } : { kind: 'known', bits: Number(digit) }

// The two counts as the backend makes them (`ipc::totals_of`): a level is bit 0 or bit 1,
// and hard is bit 1 — a subset, because a mark taken on hard counts as taken on normal too.
const hasLevel = (cell: Cell): boolean =>
  cell.kind === 'known' && (cell.bits & 3) !== 0
const isHard = (cell: Cell): boolean =>
  cell.kind === 'known' && (cell.bits & 2) !== 0

const noArt = { normalUrl: null, hardUrl: null }

// The matrix the `completion` command answers for that profile; without art it is what a
// machine without the game receives.
export const completionMatrix = (withArt: boolean): MarksMatrix => {
  const characters: CharacterRow[] = rows.map(
    ([character, digits, group, tainted], row) => ({
      character,
      group,
      tainted,
      cells: [...digits].map(cellOf),
      headUrl: withArt ? packHeadUrl(row) : null,
    }),
  )
  const cells = characters.flatMap((r) => r.cells)
  const count = (keep: (cell: Cell) => boolean): number =>
    cells.filter(keep).length
  return {
    characters,
    bosses,
    art: packMarkArt.map((art) => (withArt ? art : noArt)),
    totals: {
      cells: cells.length,
      readable: count((c) => c.kind === 'known'),
      unknown: count((c) => c.kind === 'unknown'),
      unexpected: count((c) => c.kind === 'unexpected'),
      normal: count(hasLevel),
      hard: count(isHard),
    },
  }
}
