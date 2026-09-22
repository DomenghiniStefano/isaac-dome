// The tables that fold below `--container-compact` (spec 3.13a §7). Each is a pair of
// `@utility` declarations in `assets/utilities.css`: the full template, and the narrow one the
// header and the rows take under the threshold.
export const FoldingTable = {
  Unlock: 'unlock',
  Collection: 'collection',
  Challenges: 'challenges',
} as const
export type FoldingTable = (typeof FoldingTable)[keyof typeof FoldingTable]

// How many tracks each template declares, kept here for the reason `thresholds.ts` keeps the
// three widths: so a test can hold what a stylesheet cannot state about itself — that the
// narrow set is genuinely shorter — and so a reader finds the pairs without grepping CSS.
//
// **What survives a fold was decided in 3.13b**: who the row is, how it is doing, and the
// button that acts on it. What falls is what explains the row, and what is derived from it.
export const TableTracks: Record<
  FoldingTable,
  { full: number; narrow: number }
> = {
  [FoldingTable.Unlock]: { full: 7, narrow: 4 },
  [FoldingTable.Collection]: { full: 6, narrow: 3 },
  [FoldingTable.Challenges]: { full: 6, narrow: 4 },
}
