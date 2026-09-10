import { progressShares } from '@/components/ui/progress/progressShares'

// A KPI's bar is a ratio, so it exists only with a declared denominator (Chrome e
// Stati.dc.html, "KPI"): no denominator, no bar, rather than a bar against a guess.
export const kpiBar = (
  value: number,
  denominator: number | null,
): number | null =>
  denominator === null || !Number.isFinite(denominator) || denominator <= 0
    ? null
    : progressShares(value, 0, denominator).value
