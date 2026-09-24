import { EventKey } from '@/lib/constants/eventKeys'

// Folding the section sidebar from the keyboard. `Ctrl+B` because it is the gesture the editors
// people already use have taught (VS Code, Zed), and it belongs to the **shell**, like `Ctrl+K`
// and `Ctrl+F`: it can arrive from anywhere, so it is claimed once and high up (`lib/find/keyboard.ts`
// says who owns what). A held key is read once — a repeat would fold and unfold the sidebar for as
// long as the finger stayed down.
export const togglesSidebar = (event: KeyboardEvent): boolean =>
  event.ctrlKey &&
  !event.shiftKey &&
  !event.altKey &&
  !event.repeat &&
  event.key.toLowerCase() === EventKey.B
