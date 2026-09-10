import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const FieldOrientation = {
  Vertical: 'vertical',
  Horizontal: 'horizontal',
} as const
export type FieldOrientation =
  (typeof FieldOrientation)[keyof typeof FieldOrientation]

export const fieldVariants = cva('flex w-full', {
  variants: {
    orientation: {
      [FieldOrientation.Vertical]: 'flex-col gap-1.5',
      [FieldOrientation.Horizontal]: 'flex-row items-center gap-2',
    },
  },
  defaultVariants: {
    orientation: FieldOrientation.Vertical,
  },
})
export type FieldVariants = VariantProps<typeof fieldVariants>
