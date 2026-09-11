// How large an achievement drawing is drawn: a table row's thumbnail, or a card's picture.
export const ArtSize = { Thumb: 'thumb', Card: 'card' } as const
export type ArtSize = (typeof ArtSize)[keyof typeof ArtSize]
