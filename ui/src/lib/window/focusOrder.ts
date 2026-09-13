import { ref } from 'vue'

// Tauri has no z-order API (tauri#5656), so with two overlapping windows nothing tells us which
// one the user would call the top one. Every window broadcasts when it takes the focus and every
// window keeps the same list, most recent first. It is not the z-order — a window can be raised
// without focus — and the hit test only reaches for it when two windows both hold the point.
export const rememberFocus = (order: string[], label: string): string[] => [
  label,
  ...order.filter((l) => l !== label),
]

export const focusOrder = ref<string[]>([])
