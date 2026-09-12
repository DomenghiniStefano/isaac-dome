import { onMounted, onUnmounted } from 'vue'

// A keyboard shortcut for the whole window, undone when the component goes. The handler
// decides whether the event is one of ours; only then is the browser's own behaviour
// stopped, so nothing else in the app loses a key it was listening for.
export const useShortcut = (
  handle: (event: KeyboardEvent) => boolean,
): void => {
  const onKeyDown = (event: KeyboardEvent) => {
    if (handle(event)) event.preventDefault()
  }
  onMounted(() => window.addEventListener('keydown', onKeyDown))
  onUnmounted(() => window.removeEventListener('keydown', onKeyDown))
}
