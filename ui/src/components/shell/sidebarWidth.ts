import { clamp } from 'lodash-es'

// The section sidebar's width in pixels (Chrome e Stati.dc.html, "Sidebar di sezione").
// Not a theme token: the width moves at runtime, and its bounds belong to the one function
// that enforces them. The template binds the result as a CSS variable.
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
