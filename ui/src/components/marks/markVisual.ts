import { CellLevel, type Cell, type MarkArtView } from '@/lib/ipc/types'
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
// **The mask is not decoded here.** `ipc::marks::cell_at` reads it once and sends the
// reading — the level and the online flag as their own fields — and a value outside the
// mask never arrives as `known` at all. This file used to hold its own copy of the bit
// rules, and so did `lib/completion/completionView.ts`: two copies of one measurement, in
// the layer furthest from where it was measured. That is how bit 2 kept its wrong name
// here for eight days after `docs/save-format.md` had the right one.
//
// The online flag stays beside the tier, never folded into it: it says *where* a mark was
// taken, not how high it is.
const tierOf = (level: CellLevel): MarkTier | null => {
  switch (level) {
    case CellLevel.Empty:
      return null
    case CellLevel.Normal:
      return MarkTier.Normal
    case CellLevel.Hard:
      return MarkTier.Hard
    default:
      return assertNever(level)
  }
}

export const markVisual = (cell: Cell): MarkVisual => {
  switch (cell.kind) {
    case 'unknown':
      return { kind: 'unknown' }
    case 'unexpected':
      return { kind: 'unexpected', value: cell.value }
    case 'known': {
      const { level, online } = cell
      const tier = tierOf(level)
      return tier === null
        ? { kind: 'empty', online }
        : { kind: 'marked', tier, online }
    }
    default:
      return assertNever(cell)
  }
}
