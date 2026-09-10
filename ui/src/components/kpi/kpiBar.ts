// A KPI's bar is a ratio, so it exists only with a declared denominator (Chrome e
// Stati.dc.html, "KPI"): no denominator, no bar, rather than a bar against a guess. The
// share itself is the Progress primitive's to compute.
export const hasKpiBar = (denominator: number | null): denominator is number =>
  denominator !== null && Number.isFinite(denominator) && denominator > 0
