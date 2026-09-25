import { clamp } from 'lodash-es'

// The section sidebar's width (Chrome e Stati.dc.html, "Sidebar di sezione"), in the
// pixels the design file is drawn in — the ones at scale 100. The template multiplies by
// the interface's scale, and the drag divides the pointer's travel by it, so the bounds
// mean the same thing at every size. Not a theme token: the width moves at runtime, and its
// bounds belong to the one function that enforces them.
export const SidebarWidth = {
  Min: 168,
  Default: 212,
  Max: 420,
  Step: 16,
} as const

// A whole pixel, because a pixel font on half pixels blurs; the default when the input
// isn't a usable number at all.
export const clampSidebarWidth = (px: number): number =>
  Number.isFinite(px)
    ? clamp(Math.round(px), SidebarWidth.Min, SidebarWidth.Max)
    : SidebarWidth.Default

// The width the sidebar is drawn at, from what the app holds. `null` is "nobody ever sized it",
// which is the default and not a stored width; anything stored goes through the clamp, so a
// number written by an older build with other bounds comes back inside today's.
export const shownSidebarWidth = (stored: number | null): number =>
  stored === null ? SidebarWidth.Default : clampSidebarWidth(stored)
