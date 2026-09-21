// What a sprite ends up drawing, and why. Three layers, tried in order, and the order is
// the decision: the picture itself, then the game's own "unknown" question mark, then
// whatever the caller put in the `fallback` slot — or nothing at all.
//
// It lives beside the component rather than inside it because the order is the thing worth
// checking: B69 is a bug about which of these was drawn when.
export const SpriteLayer = {
  Picture: 'picture',
  StandIn: 'standIn',
  Nothing: 'nothing',
} as const

export type SpriteLayer = (typeof SpriteLayer)[keyof typeof SpriteLayer]

export type SpriteState = {
  /** What the row asked for. `null` when the app could not resolve one. */
  url: string | null
  /** The picture was named and did not load: the same absence, found later. */
  failed: boolean
  /** The question mark, or `null` when the game is not there to serve it. */
  standIn: string | null
  standInFailed: boolean
  /** The caller brought a drawing of its own for the empty case. */
  hasOwnFallback: boolean
}

export const layerFor = (s: SpriteState): SpriteLayer => {
  if (s.url && !s.failed) return SpriteLayer.Picture
  // The caller's own drawing wins over the stand-in: it knows what belongs in that square,
  // and the stand-in only ever means "we do not have the picture for this".
  if (s.hasOwnFallback) return SpriteLayer.Nothing
  if (s.standIn && !s.standInFailed) return SpriteLayer.StandIn
  return SpriteLayer.Nothing
}
