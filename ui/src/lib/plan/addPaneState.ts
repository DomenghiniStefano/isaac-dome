import type { StepsSection } from '@/lib/ipc/types'

// What the left pane is showing. A closed set, so the template switches on a value instead
// of on a length: an empty section list and an active want are different pages, and a
// component that counted would draw "there is nothing to unlock" over a question you had
// just asked.
export const AddPaneState = {
  Want: 'want',
  Sections: 'sections',
  NoCatalog: 'noCatalog',
  Nothing: 'nothing',
} as const
export type AddPaneState = (typeof AddPaneState)[keyof typeof AddPaneState]

// `noCatalog` is the unlock view's own diagnostic, passed down rather than recomputed: two
// answers to "is the game installed" is two chances to disagree.
export const addPaneState = (
  wantActive: boolean,
  sections: StepsSection[],
  noCatalog: boolean,
): AddPaneState => {
  if (wantActive) return AddPaneState.Want
  if (sections.length > 0) return AddPaneState.Sections
  return noCatalog ? AddPaneState.NoCatalog : AddPaneState.Nothing
}
