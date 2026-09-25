// The blocks a screen's skeleton stands in for, under the title bar every one of them opens on.
// Named for what they hold the place of, so a screen's skeleton reads as its layout.
export const SkeletonBlock = {
  FilterBar: 'h-12 w-full',
  List: 'h-150 w-full',
  Card: 'h-40 w-full',
  ShortCard: 'h-32 w-full',
  /** A band across the page: a header strip, a toolbar. */
  Band: 'h-10 w-full',
  /** A card that says a few facts. */
  SummaryCard: 'h-24 w-full',
  /** The active profile's card. */
  ProfileCard: 'h-30 w-full',
  TallCard: 'h-50 w-full',
  /** A screen's hero band, the Completion summary. */
  Hero: 'h-52 w-full',
  /** Whatever height the screen has left. */
  Fill: 'flex-1',
} as const
export type SkeletonBlock = (typeof SkeletonBlock)[keyof typeof SkeletonBlock]
