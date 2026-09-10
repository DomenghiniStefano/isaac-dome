import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

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
