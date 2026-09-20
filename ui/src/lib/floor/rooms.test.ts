import { describe, expect, it } from 'vitest'
import { RoomKindView } from '@/lib/ipc/types'
import {
  ERASE_KEY,
  brushFor,
  paletteKey,
  paletteRows,
  roomFill,
  roomSymbol,
} from './rooms'

const kinds = Object.values(RoomKindView)

describe('roomFill', () => {
  it('paints every kind the game has, because a kind with no colour is an undrawable floor', () => {
    for (const kind of kinds) expect(roomFill[kind]).toBeTruthy()
  })

  it('gives each kind a fill of its own: two kinds one colour is a grid that lies', () => {
    const fills = kinds.map((kind) => roomFill[kind])
    expect(new Set(fills).size).toBe(kinds.length)
  })

  it('carries the ink with the fill, so no cell is ever drawn on without one', () => {
    for (const kind of kinds) {
      expect(roomFill[kind]).toMatch(/\bbg-floor-room-/)
      expect(roomFill[kind]).toMatch(/\btext-floor-room-/)
    }
  })
})

describe('roomSymbol', () => {
  it('knows every kind, drawing or not', () => {
    for (const kind of kinds) expect(roomSymbol[kind]).toBeDefined()
  })

  it('leaves the normal room bare: it is the most frequent, and a mark on each would be noise', () => {
    expect(roomSymbol[RoomKindView.Normal]).toBe('')
  })

  it('draws the start, which is the whole reason it is not just a normal room', () => {
    expect(roomSymbol[RoomKindView.Start]).not.toBe('')
  })

  it('marks every other kind: only the normal room goes without', () => {
    const bare = kinds.filter((kind) => roomSymbol[kind] === '')
    expect(bare).toEqual([RoomKindView.Normal])
  })
})

describe('paletteRows', () => {
  it('offers every kind exactly once', () => {
    const offered = paletteRows.flat()
    expect([...offered].sort()).toEqual([...kinds].sort())
  })

  it('stays three rows, which is what makes it a palette and not a list of fourteen', () => {
    expect(paletteRows).toHaveLength(3)
    for (const row of paletteRows) expect(row.length).toBeLessThanOrEqual(5)
  })
})

describe('paletteKey', () => {
  it('gives every kind a key, because a palette half reachable by hand is one nobody trusts', () => {
    for (const kind of kinds) expect(paletteKey[kind]).toBeTruthy()
  })

  it('never hands the same key to two kinds', () => {
    const keys = kinds.map((kind) => paletteKey[kind])
    expect(new Set(keys).size).toBe(keys.length)
  })

  it('runs the keys in the order the palette is read, so left to right is 1 to 9', () => {
    const read = paletteRows.flat().map((kind) => paletteKey[kind])
    expect(read).toEqual([
      '1',
      '2',
      '3',
      '4',
      '5',
      '6',
      '7',
      '8',
      '9',
      '0',
      'Q',
      'W',
      'E',
      'R',
    ])
  })
})

describe('brushFor', () => {
  it('answers the kind whose key was pressed', () => {
    expect(brushFor('3')).toBe(RoomKindView.Boss)
    expect(brushFor('W')).toBe(RoomKindView.Secret)
  })

  it('reads a letter typed in either case, which is how it is actually typed', () => {
    expect(brushFor('q')).toBe(RoomKindView.Sacrifice)
  })

  it('answers the eraser as null, a brush that paints nothing', () => {
    expect(brushFor(ERASE_KEY)).toBeNull()
  })

  it('answers undefined for a key that is not ours, never the eraser', () => {
    // Folding "not ours" into "erase" would rub out a room the day a field lands on
    // this screen, and nothing here would fail.
    expect(brushFor('a')).toBeUndefined()
    expect(brushFor('Enter')).toBeUndefined()
    expect(brushFor('')).toBeUndefined()
  })
})
