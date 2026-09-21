import { describe, expect, it } from 'vitest'
import { SpriteLayer, layerFor, type SpriteState } from './pixelSprite'

const state = (over: Partial<SpriteState> = {}): SpriteState => ({
  url: null,
  failed: false,
  standIn: 'isaac://localhost/unknown',
  standInFailed: false,
  hasOwnFallback: false,
  ...over,
})

describe('layerFor', () => {
  it('draws the picture when there is one', () => {
    expect(layerFor(state({ url: 'isaac://localhost/item/passive/92' }))).toBe(
      SpriteLayer.Picture,
    )
  })

  it('stands in for a picture that did not resolve, which is the whole point of B69', () => {
    expect(layerFor(state())).toBe(SpriteLayer.StandIn)
  })

  it('stands in for a picture that failed to load, not only for one that was never named', () => {
    // A URL that 404s is an icon we do not have, exactly like a null one: the row must say
    // the same thing in both cases or the feedback depends on which half broke.
    expect(
      layerFor(
        state({ url: 'isaac://localhost/item/passive/92', failed: true }),
      ),
    ).toBe(SpriteLayer.StandIn)
  })

  it("leaves the caller's own drawing alone", () => {
    // The Floor grid paints every room itself: a room the game has no minimap icon for is
    // not an unknown item, and a question mark there would be an answer to another question.
    expect(layerFor(state({ hasOwnFallback: true }))).toBe(SpriteLayer.Nothing)
  })

  it('draws nothing when there is no stand-in to draw', () => {
    // No game installed: the app never asks for a picture it cannot be served, so the
    // square stays as empty as it was before any of this.
    expect(layerFor(state({ standIn: null }))).toBe(SpriteLayer.Nothing)
  })

  it('draws nothing when the stand-in itself could not be loaded', () => {
    // Degrade, never fail: a fallback behind the fallback, because the question mark is the
    // user's own file too and may not be there.
    expect(layerFor(state({ standInFailed: true }))).toBe(SpriteLayer.Nothing)
  })
})
