// How large a page's figure is drawn: a list row's thumbnail, the page header's picture, or
// the one that leads a page's opening band (card #57).
export const WikiFigureSize = {
  Thumb: 'thumb',
  Row: 'row',
  Card: 'card',
  Hero: 'hero',
} as const
export type WikiFigureSize =
  (typeof WikiFigureSize)[keyof typeof WikiFigureSize]
