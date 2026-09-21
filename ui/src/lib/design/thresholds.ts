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
// reader can name a threshold without parsing CSS. Moving one means moving both.
//
// The section sidebar is **not** on this scale: it collapses at `--container-sidebar-room`, its
// own token, because where it stops being worth its width is a fact about the sidebar and not
// about the page (spec 3.13a §5). That number is also written by hand inside `utilities.css`'s
// two `sidebar-collapsed-*` utilities, which a `@utility` cannot express as a variant — three
// copies, and nothing here can see the third.
export const ThresholdPx: Record<Threshold, number> = {
  [Threshold.Compact]: 800,
  [Threshold.Regular]: 960,
  [Threshold.Wide]: 1280,
}
