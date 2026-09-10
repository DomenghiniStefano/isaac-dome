import type { VariantProps } from 'class-variance-authority'
import type { InjectionKey, Ref } from 'vue'
import { cva } from 'class-variance-authority'

export const ToggleSize = { Default: 'default', Icon: 'icon' } as const
export type ToggleSize = (typeof ToggleSize)[keyof typeof ToggleSize]

export const ToggleGroupType = {
  Single: 'single',
  Multiple: 'multiple',
} as const
export type ToggleGroupType =
  (typeof ToggleGroupType)[keyof typeof ToggleGroupType]

// One edge around the group and a divider between items; the chosen item is red, like a
// selected tab (Shadcn Kit.dc.html, "Tabs · Toggle group").
export const toggleGroupItemVariants = cva(
  'inline-flex h-8 shrink-0 cursor-pointer items-center justify-center gap-1 border-l border-secondary-edge text-control text-foreground first:border-l-0 hover:bg-secondary disabled:pointer-events-none disabled:text-faint-foreground enabled:data-[state=on]:bg-primary enabled:data-[state=on]:text-primary-foreground [&_svg]:pointer-events-none [&_svg:not([class*=size-])]:size-4',
  {
    variants: {
      size: {
        [ToggleSize.Default]: 'px-3',
        [ToggleSize.Icon]: 'w-8.5',
      },
    },
    defaultVariants: {
      size: ToggleSize.Default,
    },
  },
)
export type ToggleGroupItemVariants = VariantProps<
  typeof toggleGroupItemVariants
>

// The group's size, for items that don't set their own.
export const toggleGroupSizeKey: InjectionKey<Readonly<Ref<ToggleSize>>> =
  Symbol('ToggleGroupSize')
