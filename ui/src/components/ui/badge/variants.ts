import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const BadgeVariant = {
  Done: 'done',
  Now: 'now',
  Blocked: 'blocked',
  Partial: 'partial',
  Unknown: 'unknown',
  Unexpected: 'unexpected',
  Tag: 'tag',
  Challenge: 'challenge',
} as const
export type BadgeVariant = (typeof BadgeVariant)[keyof typeof BadgeVariant]

// State variants are pills with a mark (icons.ts); tag variants are square and bare
// (Shadcn Kit.dc.html, "Badge").
const statePill = 'rounded-full px-2.5 py-0.5 text-caption'
const squareTag = 'px-2 py-1 text-control'

export const badgeVariants = cva(
  'inline-flex w-fit shrink-0 items-center gap-1 border whitespace-nowrap [&_svg]:pointer-events-none [&_svg]:size-3',
  {
    variants: {
      variant: {
        [BadgeVariant.Done]: `${statePill} border-state-done bg-state-done-surface text-state-done-foreground`,
        [BadgeVariant.Now]: `${statePill} border-state-now bg-state-now-surface text-state-now-foreground`,
        [BadgeVariant.Blocked]: `${statePill} border-state-blocked bg-state-blocked-surface text-state-blocked-foreground`,
        [BadgeVariant.Unknown]: `${statePill} hatch-unknown border-dashed border-state-unknown text-state-unknown-foreground`,
        [BadgeVariant.Unexpected]: `${statePill} border-state-unexpected bg-state-unexpected-surface text-state-unexpected-foreground`,
        [BadgeVariant.Tag]: `${squareTag} border-input bg-data text-foreground`,
        [BadgeVariant.Challenge]: `${squareTag} border-challenge bg-challenge-surface text-challenge-foreground`,
      },
    },
    defaultVariants: {
      variant: BadgeVariant.Tag,
    },
  },
)
export type BadgeVariants = VariantProps<typeof badgeVariants>
