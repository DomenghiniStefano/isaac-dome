import { describe, expect, it } from 'vitest'
import { MarkTier, markArtOf, markVisual } from './markVisual'

const known = (bits: number) => markVisual({ kind: 'known', bits })

describe('markVisual', () => {
  it('reads 0 as never done', () => {
    expect(known(0)).toEqual({ kind: 'empty', third: false })
  })

  it('reads bit 0 alone as the normal mark', () => {
    expect(known(1)).toEqual({
      kind: 'marked',
      tier: MarkTier.Normal,
      third: false,
    })
  })

  it('reads bit 1 alone as the hard mark, as the game draws it', () => {
    expect(known(2)).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      third: false,
    })
  })

  it('reads both levels as the hard mark', () => {
    expect(known(3)).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      third: false,
    })
  })

  it('carries bit 2 alone on an empty cell', () => {
    expect(known(4)).toEqual({ kind: 'empty', third: true })
  })

  it('carries bit 2 beside the normal mark', () => {
    expect(known(5)).toEqual({
      kind: 'marked',
      tier: MarkTier.Normal,
      third: true,
    })
  })

  it('carries bit 2 beside the hard mark', () => {
    expect(known(6)).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      third: true,
    })
    expect(known(7)).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      third: true,
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
