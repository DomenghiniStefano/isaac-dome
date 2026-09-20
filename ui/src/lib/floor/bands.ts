import { TargetView } from '@/lib/ipc/types'
import { RankStep } from './cellView'

// The colour a band of a cell is painted, written out rather than assembled from the target's
// name: Tailwind reads these strings out of the source, and a class built at runtime generates
// no CSS — the band would simply not be there, with nothing to say why.

/**
 * One hue per target, three steps down it, **and the ink that goes with each step** — the pair
 * is never split, exactly like `roomFill`. A band prints its rank, and a fill without an ink
 * of its own is the unreadable number this screen started from: the brightest step needs dark
 * text on it and the darkest needs light, so one foreground for all nine cannot be right.
 *
 * The three hues are far apart on purpose. Before this, Secret was blue and Super was teal —
 * about forty degrees between them — and the owner's first look at the screen said they could
 * not be told apart. Blue, green and magenta sit a little over a hundred degrees from each
 * other, which is the largest spread three hues can have.
 */
export const bandFill: Record<TargetView, Record<RankStep, string>> = {
  [TargetView.Secret]: {
    [RankStep.First]:
      'bg-floor-band-secret-first text-floor-band-secret-first-foreground',
    [RankStep.Second]:
      'bg-floor-band-secret-second text-floor-band-secret-second-foreground',
    [RankStep.Third]:
      'bg-floor-band-secret-third text-floor-band-secret-third-foreground',
  },
  [TargetView.SuperSecret]: {
    [RankStep.First]:
      'bg-floor-band-super-first text-floor-band-super-first-foreground',
    [RankStep.Second]:
      'bg-floor-band-super-second text-floor-band-super-second-foreground',
    [RankStep.Third]:
      'bg-floor-band-super-third text-floor-band-super-third-foreground',
  },
  [TargetView.UltraSecret]: {
    [RankStep.First]:
      'bg-floor-band-ultra-first text-floor-band-ultra-first-foreground',
    [RankStep.Second]:
      'bg-floor-band-ultra-second text-floor-band-ultra-second-foreground',
    [RankStep.Third]:
      'bg-floor-band-ultra-third text-floor-band-ultra-third-foreground',
  },
}
