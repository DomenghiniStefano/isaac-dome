// What a KPI's number is: progress, something done, or a count of what we can't read — a
// datum like the others, never a progress.
export const KpiTone = {
  Progress: 'progress',
  Done: 'done',
  Unknown: 'unknown',
} as const
export type KpiTone = (typeof KpiTone)[keyof typeof KpiTone]
