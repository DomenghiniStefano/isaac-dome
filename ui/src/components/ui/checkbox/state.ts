import type { CheckboxCheckedState } from 'reka-ui'

// Reka's checked state is a boolean or 'indeterminate': named here so no template writes
// the string.
export const CheckboxState = {
  Checked: true,
  Unchecked: false,
  Indeterminate: 'indeterminate',
} as const satisfies Record<string, CheckboxCheckedState>
export type CheckboxState = (typeof CheckboxState)[keyof typeof CheckboxState]
