import type { Cell } from '@/lib/ipc/types'
import { assertNever } from '@/lib/assertNever'

export const MarkTier = { Normal: 'normal', Hard: 'hard' } as const
export type MarkTier = (typeof MarkTier)[keyof typeof MarkTier]

export type MarkVisual =
  | { kind: 'empty'; third: boolean }
  | { kind: 'marked'; tier: MarkTier; third: boolean }
  | { kind: 'unknown' }
  | { kind: 'unexpected'; value: number }

// The symbol URLs of one column. Normal and hard are two different sprites in the game.
export interface MarkArt {
  normal: string
  hard: string
}

// A cell is a bitmask (DESIGN-BRIEF.md §5.3): bit 0 the normal mark, bit 1 the hard one,
// bit 2 a third level whose meaning is unconfirmed. Hard wins whether or not bit 0 is set:
// `2` is common on real profiles and the game draws the hard sprite for it. Bit 2 travels
// beside the tier, never folded into it. A higher bit is outside what the save is known to
// store, so the cell reads as unexpected instead of being drawn from a guess.
const Bit = { Normal: 1, Hard: 2, Third: 4 } as const
const knownBits = Bit.Normal | Bit.Hard | Bit.Third

export const markVisual = (cell: Cell): MarkVisual => {
  switch (cell.kind) {
    case 'unknown':
      return { kind: 'unknown' }
    case 'unexpected':
      return { kind: 'unexpected', value: cell.value }
    case 'known': {
      const { bits } = cell
      if ((bits & ~knownBits) !== 0) return { kind: 'unexpected', value: bits }
      const third = (bits & Bit.Third) !== 0
      if ((bits & Bit.Hard) !== 0)
        return { kind: 'marked', tier: MarkTier.Hard, third }
      if ((bits & Bit.Normal) !== 0)
        return { kind: 'marked', tier: MarkTier.Normal, third }
      return { kind: 'empty', third }
    }
    default:
      return assertNever(cell)
  }
}
