import { describe, expect, it } from 'vitest'
import { StatusView } from '@/lib/ipc/types'
import type { DeckView, DrawnView, RollRowView } from '@/lib/ipc/types'
import {
  emptyDeckReason,
  rollCardState,
  rowOptions,
  selectionOf,
  statusText,
  toggledSelection,
} from './rollText'

const deck = (over: Partial<DeckView> = {}): DeckView => ({
  size: 0,
  taken: 0,
  unreadable: 0,
  locked: 0,
  filtered: 0,
  ...over,
})

describe('statusText', () => {
  it('names each of the three states', () => {
    expect(statusText(StatusView.Missing)).toBe('roll.status.missing')
    expect(statusText(StatusView.Taken)).toBe('roll.status.taken')
    expect(statusText(StatusView.Unreadable)).toBe('roll.status.unreadable')
  })
})

describe('emptyDeckReason', () => {
  it('has nothing to explain while the deck holds something', () => {
    expect(emptyDeckReason(deck({ size: 3, taken: 400 }))).toBe(null)
  })

  it('names the exclusion that emptied the deck', () => {
    expect(emptyDeckReason(deck({ taken: 442 }))).toBe('taken')
    expect(emptyDeckReason(deck({ unreadable: 442 }))).toBe('unreadable')
    expect(emptyDeckReason(deck({ locked: 442 }))).toBe('locked')
    expect(emptyDeckReason(deck({ filtered: 442 }))).toBe('filtered')
  })

  it('names the largest when several emptied it together', () => {
    expect(
      emptyDeckReason(deck({ taken: 137, unreadable: 40, locked: 231 })),
    ).toBe('locked')
  })

  // An empty space is not an empty deck with a reason: there is nothing to be a reason about,
  // and inventing one would put a sentence on the screen the numbers do not support.
  it('names nothing when there was nothing in the space either', () => {
    expect(emptyDeckReason(deck())).toBe(null)
  })
})

describe('selectionOf', () => {
  const rows: RollRowView[] = [
    { id: 0, name: 'Isaac', selected: true, targets: 12 },
    { id: 1, name: 'Magdalene', selected: false, targets: 12 },
  ]

  it('reads the picked ids off the rows', () => {
    expect(selectionOf(rows)).toEqual(['0'])
  })
})

describe('toggledSelection', () => {
  const ids = [0, 1, 2]

  it('turns a full pick into All rather than a list of everything', () => {
    // "every character" and "these three characters" are different answers the day a patch
    // adds a fourth, which is exactly why Selection has two variants.
    expect(toggledSelection(ids, ['0', '1', '2'])).toEqual({ kind: 'all' })
  })

  it('keeps a partial pick as the ids it names', () => {
    expect(toggledSelection(ids, ['2', '0'])).toEqual({
      kind: 'only',
      ids: [0, 2],
    })
  })

  it('keeps an empty pick as an empty list, never as All', () => {
    // Ticking nothing means nothing, and All would draw from everything: the opposite.
    expect(toggledSelection(ids, [])).toEqual({ kind: 'only', ids: [] })
  })
})

describe('rowOptions', () => {
  it('carries the count the view computed, never one recounted here', () => {
    const rows: RollRowView[] = [
      { id: 0, name: 'Isaac', selected: true, targets: 11 },
      { id: 1, name: 'Magdalene', selected: false, targets: 12 },
    ]
    expect(rowOptions(rows)).toEqual([
      { value: '0', label: 'Isaac', count: 11, picked: true },
      { value: '1', label: 'Magdalene', count: 12, picked: false },
    ])
  })
})

// What the card slot shows, and the one place it is decided.
describe('rollCardState', () => {
  const drawn = { character: 'Isaac' } as DrawnView

  it('shows the drawn target whenever there is one, whatever the deck says', () => {
    expect(rollCardState(drawn, deck())).toEqual({
      kind: 'drawn',
      drawn,
    })
  })

  // An empty deck is a first-class state: it says which exclusion emptied it, and how many.
  it('says which exclusion emptied the deck, with its count', () => {
    expect(rollCardState(null, deck({ locked: 12 }))).toEqual({
      kind: 'emptyDeck',
      key: 'roll.emptyDeck.locked',
      count: 12,
    })
  })

  it('tells a space with nothing in it from a deck not drawn from yet', () => {
    expect(rollCardState(null, deck())).toEqual({
      kind: 'nothingToDeck',
    })
    expect(rollCardState(null, deck({ size: 4 }))).toEqual({
      kind: 'notDrawnYet',
    })
  })
})
