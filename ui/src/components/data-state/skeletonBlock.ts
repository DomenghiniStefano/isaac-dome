// The blocks a screen's skeleton stands in for, under the title bar every one of them opens on.
// Named for what they hold the place of, so a screen's skeleton reads as its layout.
export const SkeletonBlock = {
  FilterBar: 'h-12 w-full',
  List: 'h-150 w-full',
  Card: 'h-40 w-full',
  ShortCard: 'h-32 w-full',
} as const
export type SkeletonBlock = (typeof SkeletonBlock)[keyof typeof SkeletonBlock]
