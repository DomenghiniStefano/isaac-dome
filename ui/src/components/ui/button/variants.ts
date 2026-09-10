import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const ButtonVariant = {
  Default: 'default',
  Secondary: 'secondary',
  Outline: 'outline',
  Ghost: 'ghost',
  Link: 'link',
} as const
export type ButtonVariant = (typeof ButtonVariant)[keyof typeof ButtonVariant]

export const ButtonSize = { Default: 'default', Icon: 'icon' } as const
export type ButtonSize = (typeof ButtonSize)[keyof typeof ButtonSize]

// Shadcn Kit.dc.html, "Button": a flat edge, no radius, and no transition — hover and
// active change colour at once (Motion.dc.html, 0ms). Disabled is its own surface, not an
// opacity.
export const buttonVariants = cva(
  'inline-flex shrink-0 cursor-pointer items-center justify-center gap-2 border text-control whitespace-nowrap select-none disabled:pointer-events-none disabled:border-secondary disabled:bg-muted disabled:text-faint-foreground [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*=size-])]:size-4',
  {
    variants: {
      variant: {
        [ButtonVariant.Default]:
          'border-primary-edge bg-primary text-primary-foreground hover:bg-primary-hover active:bg-primary-active',
        [ButtonVariant.Secondary]:
          'border-secondary-edge bg-secondary text-secondary-foreground hover:border-input hover:bg-secondary-hover active:bg-band',
        [ButtonVariant.Outline]:
          'border-input bg-transparent text-foreground hover:bg-secondary active:bg-band',
        [ButtonVariant.Ghost]:
          'border-transparent bg-transparent text-foreground hover:border-secondary-edge hover:bg-secondary active:bg-band',
        [ButtonVariant.Link]:
          'border-transparent bg-transparent text-highlight underline disabled:border-transparent disabled:bg-transparent',
      },
      size: {
        [ButtonSize.Default]: 'h-control px-4',
        [ButtonSize.Icon]: 'size-control',
      },
    },
    compoundVariants: [
      { variant: ButtonVariant.Link, size: ButtonSize.Default, class: 'px-1' },
    ],
    defaultVariants: {
      variant: ButtonVariant.Default,
      size: ButtonSize.Default,
    },
  },
)
export type ButtonVariants = VariantProps<typeof buttonVariants>
