// The navbar's two sections (DESIGN-BRIEF.md §4): two preconditions, two places.
export const NavSection = { Wiki: 'wiki', Progress: 'progress' } as const
export type NavSection = (typeof NavSection)[keyof typeof NavSection]
