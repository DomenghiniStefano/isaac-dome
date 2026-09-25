import { TargetView } from '@/lib/ipc/types'
import { RankStep } from './cellView'

// How a target's answer is painted on a cell: **the hue says which target, the height says
// which place in the order**. Both are written out rather than assembled from a name —
// Tailwind reads these strings out of the source, and a class built at runtime generates no
// CSS at all, so the cell would simply come out empty with nothing to say why.

/**
 * One colour per target, and only one: the height already says which place in the order, more
 * plainly than walking the hue down three steps would. What the hue is for is telling you which
 * of the three you are looking at without going back to read the switch.
 *
 * The three are far apart on purpose. Secret was blue and Super was teal, about forty degrees
 * between them, and the first person to open this screen could not tell them apart. Blue,
 * green and magenta sit a little over a hundred degrees from one another.
 */
export const targetFill: Record<TargetView, string> = {
  [TargetView.Secret]: 'bg-floor-target-secret',
  [TargetView.SuperSecret]: 'bg-floor-target-super',
  [TargetView.UltraSecret]: 'bg-floor-target-ultra',
}

/**
 * The cell under the level: a veil of the same hue over the whole square, a lit edge around
 * it, and the glow inside that edge which makes the square read as switched on.
 *
 * The level says how good a place is and cannot say that a place *is* one — the third step
 * fills less than a third of a 2rem square, and on a dark grid that reads as a stripe rather
 * than as an answer.
 *
 * The veil is flat. A gradient rising from the floor was tried first and it dissolved the
 * cell's own top edge into the grid: a wash that fades out has no boundary, and a boundary is
 * what "this square, not that one" is made of.
 */
export const targetAura: Record<TargetView, string> = {
  [TargetView.Secret]:
    'border border-floor-target-secret bg-floor-aura-secret shadow-floor-glow-secret',
  [TargetView.SuperSecret]:
    'border border-floor-target-super bg-floor-aura-super shadow-floor-glow-super',
  [TargetView.UltraSecret]:
    'border border-floor-target-ultra bg-floor-aura-ultra shadow-floor-glow-ultra',
}

/**
 * How much of the cell the answer fills: full for the best place the rules allow, less for
 * each step after it.
 *
 * **Three heights, not a percentage.** The rules put candidates in an order and never say by
 * how much — the wiki has no number for "how much likelier than the next one" and neither has
 * this. So the height is that order drawn as a quantity, and it stops at the third exactly
 * where the order stops meaning anything. Reading `--floor-level-third` as "thirty per cent
 * likely" would be reading a number nobody measured.
 *
 * One class per step (`h-floor-level-*` in `assets/utilities.css`), which reads the height
 * token once: binding `var(--floor-level-…)` from a string would read it here too, which the
 * scan forbids.
 */
export const levelHeight: Record<RankStep, string> = {
  [RankStep.First]: 'h-floor-level-first',
  [RankStep.Second]: 'h-floor-level-second',
  [RankStep.Third]: 'h-floor-level-third',
}
