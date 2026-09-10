// Reka's two placements for the list. The kit draws it as a panel below the trigger:
// popper. Item-aligned would cover the trigger with the list.
export const SelectPosition = {
  Popper: 'popper',
  ItemAligned: 'item-aligned',
} as const
export type SelectPosition =
  (typeof SelectPosition)[keyof typeof SelectPosition]
