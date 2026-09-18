import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

// How loudly assistive tech reads it. `role="alert"` is an assertive live region: a screen
// reader drops what it was saying mid-word. That is right for the answer to something the
// user just did and wrong for a diagnostic that was on the page before the reader arrived —
// and most Alerts here are the second kind, drawn with their screen.
export const AlertLive = { Polite: 'status', Assertive: 'alert' } as const
export type AlertLive = (typeof AlertLive)[keyof typeof AlertLive]

export const AlertVariant = {
  Default: 'default',
  Destructive: 'destructive',
} as const
export type AlertVariant = (typeof AlertVariant)[keyof typeof AlertVariant]

// A diagnostic, not a notification: no dismiss control, it goes when the cause does. The
// default icon is cream, not cyan: cyan is reserved to focus.
export const alertVariants = cva(
  'relative grid w-full gap-0.5 border p-3 text-left text-body has-[>svg]:grid-cols-[auto_1fr] has-[>svg]:gap-x-2.5 *:[svg]:row-span-2 *:[svg]:mt-0.5 *:[svg:not([class*=size-])]:size-4',
  {
    variants: {
      variant: {
        [AlertVariant.Default]:
          'border-border bg-data text-foreground *:[svg]:text-highlight',
        [AlertVariant.Destructive]:
          'border-destructive bg-destructive-surface text-foreground *:[svg]:text-destructive *:data-[slot=alert-description]:text-destructive-foreground',
      },
    },
    defaultVariants: {
      variant: AlertVariant.Default,
    },
  },
)
export type AlertVariants = VariantProps<typeof alertVariants>
