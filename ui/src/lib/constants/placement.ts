// Reka UI's orientation and floating placement, as values templates and defaults can name.
export const Orientation = {
  Horizontal: 'horizontal',
  Vertical: 'vertical',
} as const
export type Orientation = (typeof Orientation)[keyof typeof Orientation]

export const Align = { Start: 'start', Center: 'center', End: 'end' } as const
export type Align = (typeof Align)[keyof typeof Align]

export const Side = {
  Top: 'top',
  Right: 'right',
  Bottom: 'bottom',
  Left: 'left',
} as const
export type Side = (typeof Side)[keyof typeof Side]
