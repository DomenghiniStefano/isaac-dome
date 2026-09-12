// How large a page's figure is drawn: a list row's thumbnail, or the page header's picture.
export const WikiFigureSize = { Thumb: 'thumb', Card: 'card' } as const
export type WikiFigureSize =
  (typeof WikiFigureSize)[keyof typeof WikiFigureSize]
