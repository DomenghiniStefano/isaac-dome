// How large an achievement drawing is drawn. `Thumb` is every caller that isn't `WikiFigure`
// (`SearchRow`, `GoalRow`, `UnlockRow`, `ScalePreview`): a table row's small picture, at its
// own scale. `Row`, `Card`, `Tile` and `Hero` are `WikiFigure`'s own four sizes (card #90,
// decision 2) — the painting's width equals its frame's, in `AchievementArt.vue`, so the
// picture is a whole `FigureSize` box wide and only its aspect ratio decides the height.
export const ArtSize = {
  Thumb: 'thumb',
  Row: 'row',
  Card: 'card',
  Tile: 'tile',
  Hero: 'hero',
} as const
export type ArtSize = (typeof ArtSize)[keyof typeof ArtSize]
