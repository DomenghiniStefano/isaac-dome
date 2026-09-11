import { cva } from 'class-variance-authority'

// Default is the kit's bar; Micro is the KPI tile's 4px bar (Chrome e Stati.dc.html, "KPI").
export const ProgressSize = { Default: 'default', Micro: 'micro' } as const
export type ProgressSize = (typeof ProgressSize)[keyof typeof ProgressSize]

// The fill's colour: progress, something done, or a count of what can't be read.
export const ProgressTone = {
  Primary: 'primary',
  Done: 'done',
  Muted: 'muted',
} as const
export type ProgressTone = (typeof ProgressTone)[keyof typeof ProgressTone]

export const progressVariants = cva(
  'flex w-full overflow-hidden border bg-data',
  {
    variants: {
      size: {
        [ProgressSize.Default]: 'h-3.5 border-input',
        [ProgressSize.Micro]: 'h-1 border-secondary',
      },
    },
    defaultVariants: { size: ProgressSize.Default },
  },
)

export const progressIndicatorVariants = cva('h-full w-(--progress-value)', {
  variants: {
    tone: {
      [ProgressTone.Primary]: 'bg-primary',
      [ProgressTone.Done]: 'bg-state-done',
      [ProgressTone.Muted]: 'bg-faint-foreground',
    },
  },
  defaultVariants: { tone: ProgressTone.Primary },
})
