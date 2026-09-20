import { TargetView } from '@/lib/ipc/types'
import { Corner, RankStep } from './cellView'

// Where a pip sits and what colour it is. Both are written out rather than assembled from the
// target's name: Tailwind reads these strings out of the source, and a class built at runtime
// generates no CSS — the pip would simply not be there, with nothing to say why.

/**
 * One hue per target, three steps down it. The hue is the corner said a second time: someone
 * who has learned that top-left means the Secret Room reads it from the position, and someone
 * who has learned the blue reads it from the colour.
 */
export const pipFill: Record<TargetView, Record<RankStep, string>> = {
  [TargetView.Secret]: {
    [RankStep.First]: 'bg-floor-pip-secret-first',
    [RankStep.Second]: 'bg-floor-pip-secret-second',
    [RankStep.Third]: 'bg-floor-pip-secret-third',
  },
  [TargetView.SuperSecret]: {
    [RankStep.First]: 'bg-floor-pip-super-first',
    [RankStep.Second]: 'bg-floor-pip-super-second',
    [RankStep.Third]: 'bg-floor-pip-super-third',
  },
  [TargetView.UltraSecret]: {
    [RankStep.First]: 'bg-floor-pip-ultra-first',
    [RankStep.Second]: 'bg-floor-pip-ultra-second',
    [RankStep.Third]: 'bg-floor-pip-ultra-third',
  },
}

/**
 * The three corners in use. Bottom-right is not here on purpose: its emptiness is visible, and
 * a fourth target would have somewhere to go without moving the other three.
 */
export const cornerAt: Record<Corner, string> = {
  [Corner.TopLeft]: 'top-0 left-0',
  [Corner.TopRight]: 'top-0 right-0',
  [Corner.BottomLeft]: 'bottom-0 left-0',
}
