import type { InjectionKey, Ref } from 'vue'

// The element a screen scrolls as a whole, handed to the `VirtualRows` inside it. A list that
// finds one scrolls with its page — the band and the filters above it go by as it does, and
// only what the screen pins stays in view — instead of in a box of its own under a band that
// never moves. A list with no page scroller around it keeps its own box, as before.
export const PageScroller: InjectionKey<Ref<HTMLElement | null>> =
  Symbol('PageScroller')
