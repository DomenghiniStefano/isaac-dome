// How large `WikiFigure` draws a page's picture: a list row, a card in the grid, a landing
// tile, or the one leading a page's opening band. `Thumb` — a bare
// sprite with no frame — left with the list rows it was for: every size now gets the same
// frame, so a picture reads as the same object in a row, a card and a band.
export const FigureSize = {
  Row: 'row',
  Card: 'card',
  Tile: 'tile',
  Hero: 'hero',
} as const
export type FigureSize = (typeof FigureSize)[keyof typeof FigureSize]
