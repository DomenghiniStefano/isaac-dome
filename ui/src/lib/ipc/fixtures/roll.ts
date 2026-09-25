import { MarkColumnView, StatusView } from '../types'
import type { DrawnView, PresetView, RollRowView, RollView } from '../types'

// Development only: the Roll screen on the development server (card #80, item 10 — it threw
// "no fixture answers roll"). The deck is a handful of rows, not the game's matrix: enough to
// press Pesca, change the preset and watch the counts follow. It lives for the page's life,
// like the settings fixture.
const characterNames = ['Isaac', 'Magdalene', 'Cain', 'Judas']
const columnNames = ["Mom's Heart", 'Isaac', 'Satan', 'The Lamb']
const drawable: MarkColumnView[] = [
  MarkColumnView.MomsHeart,
  MarkColumnView.Isaac,
  MarkColumnView.Satan,
  MarkColumnView.TheLamb,
]

const all: PresetView = {
  characters: { kind: 'all' },
  columns: { kind: 'all' },
  includeTaken: false,
  onlyPlayable: true,
}

let preset: PresetView = all
let drawn: DrawnView | null = null
let draws = 0

export const resetRollFixture = (): void => {
  preset = all
  drawn = null
  draws = 0
}

const rows = (
  names: string[],
  selection: PresetView['characters'],
): RollRowView[] =>
  names.map((name, id) => ({
    id,
    name,
    selected: selection.kind === 'all' || selection.ids.includes(id),
    targets: 4,
  }))

const view = (): RollView => {
  const characters = rows(characterNames, preset.characters)
  const columns = rows(columnNames, preset.columns)
  const size =
    characters.filter((r) => r.selected).length *
    columns.filter((r) => r.selected).length
  return {
    drawn,
    deck: {
      size,
      taken: 0,
      unreadable: 0,
      locked: 0,
      filtered: characterNames.length * columnNames.length - size,
    },
    characters,
    columns,
    preset,
    diagnostics: [],
  }
}

export const rollAnswer = (): RollView => view()

// Walks the selected cells in order, one per press: predictable, so a screenshot of the
// second draw is the same every time.
export const rollDrawAnswer = (): RollView => {
  const current = view()
  const characters = current.characters.filter((r) => r.selected)
  const columns = current.columns.filter((r) => r.selected)
  const cells = characters.flatMap((ch) =>
    columns.map((co) => [ch, co] as const),
  )
  const cell = cells[draws % Math.max(cells.length, 1)]
  draws += 1
  drawn =
    cell === undefined
      ? null
      : {
          target: {
            kind: 'mark',
            column: drawable[cell[1].id] ?? MarkColumnView.MomsHeart,
          },
          character: cell[0].name,
          headUrl: null,
          artUrl: null,
          status: StatusView.Missing,
          deckSize: current.deck.size,
          drawnUnix: Math.floor(Date.now() / 1000),
        }
  return view()
}

export const setRollPresetAnswer = (next: PresetView): RollView => {
  preset = next
  return view()
}
