import { ref } from 'vue'

// **One width, for the whole app.** §8 of the tabs session spec puts `sidebarWidth` beside
// `windows` and not inside one, so the document holds a single number — and if two windows held
// different ones the document would silently keep whichever was written last, with the user
// finding out which at the next start. So the value is shared at runtime too: dragging one
// window's edge moves the others'. That cost is real, and it is the thing to judge in a window
// rather than assume.
//
// `null` means nobody has ever sized it, which is not the same as the default. **The default and
// the bounds are not here**: they belong to the sidebar, which is a component, and nothing under
// `lib/` imports from `components/` — `pnpm scan` refuses it since 2026-09-24. What travels here
// is a number; what makes it a legal width is `clampSidebarWidth`, where it is drawn.
export const sidebarWidth = ref<number | null>(null)

export const setSidebarWidth = (px: number | null): void => {
  sidebarWidth.value = px
}

// Whether somebody folded the sidebar to its icons. Shared for the same reason as the width, and
// it is **only the asked-for half** of the collapse: a shell too narrow for the sidebar folds it in
// CSS (spec 3.13a §6) and never writes here, so narrowing a window and widening it again gives
// back the sidebar the user had, not one the window decided on.
export const sidebarCollapsed = ref(false)

export const setSidebarCollapsed = (folded: boolean): void => {
  sidebarCollapsed.value = folded
}

// The two together, as they travel between windows.
export interface Layout {
  sidebarWidth: number | null
  sidebarCollapsed: boolean
}

export const currentLayout = (): Layout => ({
  sidebarWidth: sidebarWidth.value,
  sidebarCollapsed: sidebarCollapsed.value,
})

export const setLayout = (layout: Layout): void => {
  setSidebarWidth(layout.sidebarWidth)
  setSidebarCollapsed(layout.sidebarCollapsed)
}

export const sameLayout = (a: Layout | null, b: Layout): boolean =>
  a !== null &&
  a.sidebarWidth === b.sidebarWidth &&
  a.sidebarCollapsed === b.sidebarCollapsed
