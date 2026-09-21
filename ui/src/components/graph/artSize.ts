// How large an achievement drawing is drawn: a table row's thumbnail, a card's picture, or
// the one leading a wiki page's opening band (card #57).
export const ArtSize = { Thumb: 'thumb', Card: 'card', Hero: 'hero' } as const
export type ArtSize = (typeof ArtSize)[keyof typeof ArtSize]
