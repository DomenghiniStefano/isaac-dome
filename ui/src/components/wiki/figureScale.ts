// The game's icons are 32px pixel art: at a fraction of that they smear, so a sprite drawn
// inside a box is scaled up by a whole number, never a fraction. `SpriteNativePx` is the
// game's own size; `FigureBoxPx` is each `FigureSize`'s box, in CSS px at scale 100 — the
// same nominal baseline `spacing.css`'s own comments reason from (e.g.
// `--spacing-matrix-total`). Both are token-derived constants, not measurements: the boxes
// were chosen in `spacing.css` as whole multiples of 32 for exactly this reason, so
// `integerScale` fills every one of them edge to edge.
import { FigureSize } from './figureSize'

export const SpriteNativePx = 32

export const FigureBoxPx: Record<FigureSize, number> = {
  [FigureSize.Row]: 64, // --spacing-figure-row: 4rem
  [FigureSize.Card]: 128, // --spacing-figure-card: 8rem
  [FigureSize.Tile]: 96, // --spacing-figure-tile: 6rem
  [FigureSize.Hero]: 192, // --spacing-figure-hero: 12rem
}

// The largest integer scale a `nativePx` sprite can be drawn at without spilling past
// `boxPx`. `1` and never `0`: a sprite bigger than its box is still drawn, at its own size,
// rather than vanish.
export const integerScale = (nativePx: number, boxPx: number): number =>
  Math.max(1, Math.floor(boxPx / nativePx))
