import type { VariantProps } from 'class-variance-authority'
import type { InjectionKey, Ref } from 'vue'
import { cva } from 'class-variance-authority'

export const TableDensity = {
  Compact: 'compact',
  Normal: 'normal',
  Wide: 'wide',
} as const
export type TableDensity = (typeof TableDensity)[keyof typeof TableDensity]

// Chrome e Stati.dc.html, "Densità": only the body rows' height changes; text stays
// text-row and the header keeps its own height.
export const tableBodyVariants = cva('[&>tr:last-child]:border-b-0', {
  variants: {
    density: {
      [TableDensity.Compact]: '[&>tr]:h-row-compact',
      [TableDensity.Normal]: '[&>tr]:h-row',
      [TableDensity.Wide]: '[&>tr]:h-row-wide',
    },
  },
  defaultVariants: {
    density: TableDensity.Normal,
  },
})
export type TableBodyVariants = VariantProps<typeof tableBodyVariants>

// Set on Table, read by TableBody.
export const tableDensityKey: InjectionKey<Readonly<Ref<TableDensity>>> =
  Symbol('TableDensity')
