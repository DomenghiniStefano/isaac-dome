// The three widths a screen changes shape at (spec 3.13a §5). Three and not seven: a shared scale
// is the only thing that keeps twenty-two screens telling the same story, and each extra step is
// another state nobody will ever look at.
export const Threshold = {
  Compact: 'compact',
  Regular: 'regular',
  Wide: 'wide',
} as const
export type Threshold = (typeof Threshold)[keyof typeof Threshold]

// The px each one is declared at in `assets/theme/containers.css`, kept here so a test and any
// reader can name a threshold without parsing CSS. Moving one means moving both — and, for
// compact, also the literal inside `utilities.css`'s `sidebar-collapsed-hidden`, which a
// `@utility` cannot express as a variant.
export const ThresholdPx: Record<Threshold, number> = {
  [Threshold.Compact]: 560,
  [Threshold.Regular]: 960,
  [Threshold.Wide]: 1280,
}
