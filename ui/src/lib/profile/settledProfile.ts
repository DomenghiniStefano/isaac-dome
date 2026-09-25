import type { ActiveProfile, SetupState } from '@/lib/ipc/types'

// The one variant of `ActiveProfile` a screen can read a save through.
export type SettledProfile = Extract<ActiveProfile, { kind: 'active' }>

// The profile the window has settled on, or `null` while there is none: nothing read yet, no
// save found, or a choice still to make.
export const settledProfile = (
  setup: SetupState | null,
): SettledProfile | null =>
  setup?.active.kind === 'active' ? setup.active : null
