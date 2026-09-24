import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const ButtonVariant = {
  Default: 'default',
  Secondary: 'secondary',
  Outline: 'outline',
  Ghost: 'ghost',
  Link: 'link',
  // Throws something away and cannot put it back. Red is the app's warning colour and it is
  // reserved: a button wearing it is saying the action is not undoable, so it is worth a
  // confirmation and the confirmation is worth writing.
  Destructive: 'destructive',
  // Cycle 2: the chrome and the wiki (Chrome e Stati.dc.html).
  Nav: 'nav',
  Section: 'section',
  Ref: 'ref',
  Chrome: 'chrome',
  ChromeDanger: 'chromeDanger',
  Field: 'field',
  // One square of the Floor screen's 13x13 grid. The fill is the caller's: what a cell means
  // — unpainted, a room, or a cell a cited rule allows — is the answer the screen computes,
  // and cva cannot express a colour that changes per cell.
  Cell: 'cell',
  // A tab on the sidebar's inner edge that folds it: the sidebar's own surface and edge, so it
  // reads as a piece of the sidebar sticking out rather than a button laid on the page.
  SidebarTab: 'sidebarTab',
} as const
export type ButtonVariant = (typeof ButtonVariant)[keyof typeof ButtonVariant]

export const ButtonSize = {
  Default: 'default',
  Icon: 'icon',
  IconCompact: 'iconCompact',
  Micro: 'micro',
  Row: 'row',
  Inline: 'inline',
  Window: 'window',
  Compact: 'compact',
  Section: 'section',
  Cell: 'cell',
  Brush: 'brush',
  SidebarTab: 'sidebarTab',
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
        // The same two colours the destructive Alert uses, so the button and the box that
        // explains it are visibly the same warning. It fills with the red on hover, which is
        // the one moment the pointer is on top of something irreversible.
        [ButtonVariant.Destructive]:
          'border-destructive bg-destructive-surface text-destructive-foreground hover:bg-destructive hover:text-foreground active:bg-destructive',
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
        // Disabled keeps the chrome's transparency: a filled square inside the navbar or
        // the title bar would read as a surface, and the bar has none.
        [ButtonVariant.Chrome]:
          'border-transparent bg-transparent text-muted-foreground hover:bg-secondary disabled:border-transparent disabled:bg-transparent',
        [ButtonVariant.ChromeDanger]:
          'border-transparent bg-transparent text-muted-foreground hover:bg-primary hover:text-primary-foreground',
        // Looks like a field, opens something else: the navbar's search trigger.
        [ButtonVariant.Field]:
          'justify-start border-secondary bg-data text-faint-foreground hover:border-input',
        // No fill, no ink and no edge of its own: the cell is handed the colour of the room
        // that was painted on it, and 169 borders would be a lattice the eye reads before it
        // reads the answer.
        [ButtonVariant.Cell]: 'border-transparent hover:border-input',
        [ButtonVariant.SidebarTab]:
          'border-secondary bg-data text-subtle-foreground hover:border-input hover:text-foreground',
      },
      size: {
        [ButtonSize.Default]: 'h-control px-4',
        [ButtonSize.Icon]: 'size-control',
        // The tab strip's "+": a 22px square inside the 30px title bar.
        [ButtonSize.IconCompact]:
          'size-icon-compact [&_svg:not([class*=size-])]:size-3',
        [ButtonSize.Micro]:
          'size-3.5 border-0 p-0 [&_svg:not([class*=size-])]:size-2',
        [ButtonSize.Row]: 'h-auto w-full justify-start px-3 py-2 text-row',
        // Inline, not inline-flex: in a flex box an icon's bottom edge becomes the baseline
        // and lifts the label off the line of the surrounding text.
        [ButtonSize.Inline]: 'inline h-auto p-0 align-baseline text-row',
        [ButtonSize.Window]: 'h-full w-window-control border-0',
        [ButtonSize.Compact]: 'h-7 gap-2 px-2 text-caption',
        // A navbar section: the navbar's full height.
        [ButtonSize.Section]:
          'h-full gap-2 px-3 [&_svg:not([class*=size-])]:size-3.5',
        [ButtonSize.Cell]: 'size-floor-cell p-0 text-caption',
        // One room of the Floor palette: exactly a cell tall, so the palette's rows line up with
        // the grid's beside it, and edgeless, so the swatch is flush with the row's own edge.
        [ButtonSize.Brush]:
          'h-floor-cell w-full justify-start gap-2.5 border-0 p-0 pr-1.5 text-row',
        // Twice as tall as it is wide: a tab, not a square, so it reads as hanging off the edge.
        [ButtonSize.SidebarTab]:
          'h-8 w-4 p-0 [&_svg:not([class*=size-])]:size-3',
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
