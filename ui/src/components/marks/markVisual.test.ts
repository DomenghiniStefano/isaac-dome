import { describe, expect, it } from 'vitest'
import { MarkTier, markArtOf, markVisual } from './markVisual'

const known = (bits: number) => markVisual({ kind: 'known', bits })

describe('markVisual', () => {
  it('reads 0 as never done', () => {
    expect(known(0)).toEqual({ kind: 'empty', online: false })
  })

  it('reads bit 0 alone as the normal mark', () => {
    expect(known(1)).toEqual({
      kind: 'marked',
      tier: MarkTier.Normal,
      online: false,
    })
  })

  it('reads bit 1 alone as the hard mark, as the game draws it', () => {
    expect(known(2)).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      online: false,
    })
  })

  it('reads both levels as the hard mark', () => {
    expect(known(3)).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      online: false,
    })
  })

  // 4 and 6 are shapes no real save has shown — the observed set is 0, 1, 2, 3, 5, 7. They
  // are handled rather than assumed away: a value the tables can represent has to draw
  // something, and "never seen" is not "cannot happen".
  it('carries the online win on a cell with no mark of its own', () => {
    expect(known(4)).toEqual({ kind: 'empty', online: true })
  })

  it('carries the online win beside the normal mark', () => {
    expect(known(5)).toEqual({
      kind: 'marked',
      tier: MarkTier.Normal,
      online: true,
    })
  })

  it('carries the online win beside the hard mark', () => {
    expect(known(6)).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      online: true,
    })
    expect(known(7)).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      online: true,
    })
  })

  it('reads a bit above bit 2 as unexpected instead of drawing a guess', () => {
    expect(known(8)).toEqual({ kind: 'unexpected', value: 8 })
  })

  it('keeps an unreadable cell unreadable', () => {
    expect(markVisual({ kind: 'unknown' })).toEqual({ kind: 'unknown' })
  })

  it("keeps the IPC's unexpected value", () => {
    expect(markVisual({ kind: 'unexpected', value: 9 })).toEqual({
      kind: 'unexpected',
      value: 9,
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
