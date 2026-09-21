import { describe, expect, it } from 'vitest'
import { CellLevel, type Cell } from '@/lib/ipc/types'
import { MarkTier, markArtOf, markVisual } from './markVisual'

// The cell as the IPC sends it. `bits` is the value a real cell in that state carries, and
// nothing here reads it: since B21 the mask is decoded once, in `ipc::marks::cell_at`, and
// the tests that pin *that* reading live in `crates/ipc/tests/marks.rs`. What is left for
// this module is the mapping onto what the grid draws.
const cell = (bits: number, level: CellLevel, online: boolean): Cell => ({
  kind: 'known',
  bits,
  level,
  online,
})

describe('markVisual', () => {
  it('draws nothing for a cell that reached no level', () => {
    expect(cell(0, CellLevel.Empty, false)).toBeDefined()
    expect(markVisual(cell(0, CellLevel.Empty, false))).toEqual({
      kind: 'empty',
      online: false,
    })
  })

  it('draws the normal tier for the first level', () => {
    expect(markVisual(cell(1, CellLevel.Normal, false))).toEqual({
      kind: 'marked',
      tier: MarkTier.Normal,
      online: false,
    })
  })

  it('draws the hard tier for the second level, as the game draws it', () => {
    // 2 without bit 0 is an ordinary cell on a real profile: a mark replaces the one
    // before it (B58). The IPC has already said `hard`, and this module does not second
    // guess it.
    expect(markVisual(cell(2, CellLevel.Hard, false))).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      online: false,
    })
    expect(markVisual(cell(3, CellLevel.Hard, false))).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      online: false,
    })
  })

  it('carries the online win on a cell with no mark of its own', () => {
    // A shape no real save has shown — the observed set is 0, 1, 2, 3, 5, 7. It is handled
    // rather than assumed away: "never seen" is not "cannot happen".
    expect(markVisual(cell(4, CellLevel.Empty, true))).toEqual({
      kind: 'empty',
      online: true,
    })
  })

  it('carries the online win beside the normal mark', () => {
    expect(markVisual(cell(5, CellLevel.Normal, true))).toEqual({
      kind: 'marked',
      tier: MarkTier.Normal,
      online: true,
    })
  })

  it('carries the online win beside the hard mark', () => {
    expect(markVisual(cell(7, CellLevel.Hard, true))).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      online: true,
    })
  })

  it('keeps an unreadable cell unreadable', () => {
    expect(markVisual({ kind: 'unknown' })).toEqual({ kind: 'unknown' })
  })

  it("keeps the IPC's unexpected value", () => {
    // A value outside the mask never arrives as `known`: `ipc::marks` sends it here as
    // `unexpected`, so this module no longer has a range of its own to police.
    expect(markVisual({ kind: 'unexpected', value: 9 })).toEqual({
      kind: 'unexpected',
      value: 9,
    })
  })
})

// B21: the mask is decoded once, in `ipc::marks::cell_at`, and this module reads the
// reading. A cell whose bits disagree with its level is not a shape the IPC can send — it
// is the probe that says whether a decoder is still living here, where it would drift from
// the Rust one the way the tooltip's wording did for eight days.
describe('markVisual reads the reading and not the mask', () => {
  it('takes the tier from the level', () => {
    expect(markVisual(cell(0, CellLevel.Hard, false))).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      online: false,
    })
  })

  it('takes the online win from its own field', () => {
    expect(markVisual(cell(0, CellLevel.Empty, true))).toEqual({
      kind: 'empty',
      online: true,
    })
  })
})

describe('markArtOf', () => {
  it('draws a column only when both tiers have a URL', () => {
    expect(markArtOf({ normalUrl: 'n', hardUrl: 'h' })).toEqual({
      normal: 'n',
      hard: 'h',
    })
  })

  it('draws nothing for a column missing a tier, or missing altogether', () => {
    expect(markArtOf({ normalUrl: 'n', hardUrl: null })).toBeNull()
    expect(markArtOf({ normalUrl: null, hardUrl: null })).toBeNull()
    expect(markArtOf(undefined)).toBeNull()
  })
})
