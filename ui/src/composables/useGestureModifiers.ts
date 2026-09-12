import { onMounted, onUnmounted, ref } from 'vue'
import type { Ref } from 'vue'

// Whether the gesture happening right now holds `Ctrl`.
//
// A list built on Reka answers with `select`, which says *that* a row was chosen and not
// *how*: the click can't be read there, and reading the click instead of `select` fires
// twice, because Reka replays it on the item. The modifier is therefore taken from the
// window, in capture, before the row's own handler runs — the same place `useShortcut`
// listens, and for the same reason: the gesture belongs to the window, not to one element.
export const useGestureModifiers = (): { ctrl: Ref<boolean> } => {
  const ctrl = ref(false)
  const remember = (event: MouseEvent | PointerEvent | KeyboardEvent) => {
    ctrl.value = event.ctrlKey
  }
  onMounted(() => {
    window.addEventListener('pointerdown', remember, true)
    window.addEventListener('keydown', remember, true)
  })
  onUnmounted(() => {
    window.removeEventListener('pointerdown', remember, true)
    window.removeEventListener('keydown', remember, true)
  })
  return { ctrl }
}
