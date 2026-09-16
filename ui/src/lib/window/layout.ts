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
// `lib/` imports from `components/` — checked, and this file keeps it that way. What travels here
// is a number; what makes it a legal width is `clampSidebarWidth`, where it is drawn.
export const sidebarWidth = ref<number | null>(null)

export const setSidebarWidth = (px: number | null): void => {
  sidebarWidth.value = px
}
