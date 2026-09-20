import type { Cell, MarkArtView } from '@/lib/ipc/types'
import { assertNever } from '@/lib/assertNever'

export const MarkTier = { Normal: 'normal', Hard: 'hard' } as const
export type MarkTier = (typeof MarkTier)[keyof typeof MarkTier]

export type MarkVisual =
  | { kind: 'empty'; online: boolean }
  | { kind: 'marked'; tier: MarkTier; online: boolean }
  | { kind: 'unknown' }
  | { kind: 'unexpected'; value: number }

// The fallback outfit's bar, as a share of the cell's height (Tokens.dc.html): a third for
// the normal mark, two thirds for the hard one.
export const markBarShare: Record<MarkTier, string> = {
  [MarkTier.Normal]: '33.333%',
  [MarkTier.Hard]: '66.667%',
}

// The symbol URLs of one column. Normal and hard are two different sprites in the game.
export interface MarkArt {
  normal: string
  hard: string
}

// A column's art as the IPC sends it, drawn only when both tiers have a URL: a column that
// showed one tier as a sprite and the other as bars would read as two different codes.
export const markArtOf = (view: MarkArtView | undefined): MarkArt | null => {
  const normal = view?.normalUrl
  const hard = view?.hardUrl
  return normal && hard ? { normal, hard } : null
}

// A cell is a bitmask: bit 0 the normal mark, bit 1 the hard one, bit 2 **the boss beaten
// online** — measured 2026-09-12 on a matched window around an online co-op run, and
// confirmed by the owner on 2026-09-20 against the game's own completion screen. It was
// called "a third level whose meaning is unconfirmed" here for eight days after the
// measurement settled it: a name taken from a guess outliving the guess
// (`docs/save-format.md`, "Counters and marks").
//
// Hard wins whether or not bit 0 is set: `2` is common on real profiles and the game draws
// the hard sprite for it. The online flag travels beside the tier, never folded into it —
// it says *where* a mark was taken, not how high it is. A higher bit is outside what the
// save is known to store, so the cell reads as unexpected instead of being drawn from a
// guess.
const Bit = { Normal: 1, Hard: 2, Online: 4 } as const
const knownBits = Bit.Normal | Bit.Hard | Bit.Online

export const markVisual = (cell: Cell): MarkVisual => {
  switch (cell.kind) {
    case 'unknown':
      return { kind: 'unknown' }
    case 'unexpected':
      return { kind: 'unexpected', value: cell.value }
    case 'known': {
      const { bits } = cell
      if ((bits & ~knownBits) !== 0) return { kind: 'unexpected', value: bits }
      const online = (bits & Bit.Online) !== 0
      if ((bits & Bit.Hard) !== 0)
        return { kind: 'marked', tier: MarkTier.Hard, online }
      if ((bits & Bit.Normal) !== 0)
        return { kind: 'marked', tier: MarkTier.Normal, online }
      return { kind: 'empty', online }
    }
    default:
      return assertNever(cell)
  }
}
