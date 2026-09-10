import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const ButtonVariant = {
  Default: 'default',
  Secondary: 'secondary',
  Outline: 'outline',
  Ghost: 'ghost',
  Link: 'link',
  // Cycle 2: the chrome and the wiki (Chrome e Stati.dc.html).
  Nav: 'nav',
  Section: 'section',
  Ref: 'ref',
  Chrome: 'chrome',
  ChromeDanger: 'chromeDanger',
  Field: 'field',
} as const
export type ButtonVariant = (typeof ButtonVariant)[keyof typeof ButtonVariant]

export const ButtonSize = {
  Default: 'default',
  Icon: 'icon',
  Micro: 'micro',
  Row: 'row',
  Inline: 'inline',
  Window: 'window',
  Compact: 'compact',
} as const
export type ButtonSize = (typeof ButtonSize)[keyof typeof ButtonSize]

// Shadcn Kit.dc.html, "Button": a flat edge, no radius, and no transition — hover and
// active change colour at once (Motion.dc.html, 0ms). Disabled is its own surface, not an
// opacity. Sizes only size and variants only colour: cva writes the size's classes after
// the variant's, so a height inside a variant would lose.
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
        // A sidebar item: the red bar on the left says which one, the fill alone doesn't
        // on an already dark row.
        [ButtonVariant.Nav]:
          'justify-start border-0 border-l-3 border-transparent bg-transparent text-foreground-soft hover:bg-secondary aria-[current=page]:border-selection-edge aria-[current=page]:bg-primary aria-[current=page]:text-foreground',
        // A navbar section: a cream top edge on the data surface.
        [ButtonVariant.Section]:
          'border-0 border-t-2 border-transparent bg-transparent tracking-nav text-subtle-foreground hover:text-foreground aria-[current=page]:border-highlight aria-[current=page]:bg-data aria-[current=page]:text-highlight',
        // A resolved wiki reference: cream over a solid underline edge.
        [ButtonVariant.Ref]:
          'border-0 border-b border-secondary-edge bg-transparent text-highlight hover:border-highlight',
        [ButtonVariant.Chrome]:
          'border-transparent bg-transparent text-muted-foreground hover:bg-secondary',
        [ButtonVariant.ChromeDanger]:
          'border-transparent bg-transparent text-muted-foreground hover:bg-primary hover:text-primary-foreground',
        // Looks like a field, opens something else: the navbar's search trigger.
        [ButtonVariant.Field]:
          'justify-start border-secondary bg-data text-faint-foreground hover:border-input',
      },
      size: {
        [ButtonSize.Default]: 'h-control px-4',
        [ButtonSize.Icon]: 'size-control',
        [ButtonSize.Micro]:
          'size-3.5 border-0 p-0 [&_svg:not([class*=size-])]:size-2',
        [ButtonSize.Row]:
          'h-auto w-full justify-start px-2.75 py-1.75 text-row',
        // Inline, not inline-flex: in a flex box an icon's bottom edge becomes the baseline
        // and lifts the label off the line of the surrounding text.
        [ButtonSize.Inline]: 'inline h-auto p-0 align-baseline text-row',
        [ButtonSize.Window]: 'h-full w-window-control border-0',
        [ButtonSize.Compact]: 'h-7 gap-1.75 px-2.25 text-caption',
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
