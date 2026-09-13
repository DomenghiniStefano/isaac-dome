import { onMounted, onUnmounted } from 'vue'

// A mouse shortcut for the whole window, undone when the component goes. `read` says what
// the event means to us and `null` when it means nothing: only then is the browser's own
// behaviour stopped, so nothing else in the app loses a button it was listening for.
//
// A side button is two events, and both are needed. The press is where the webview would
// start a navigation of its own, so it is the one to stop; the click is where the gesture is
// complete, and acting there means a press that ends elsewhere does nothing, exactly like
// the middle click that closes a tab.
export const usePointerShortcut = <T>(
  read: (event: MouseEvent) => T | null,
  act: (action: T) => void,
): void => {
  const onMouseDown = (event: MouseEvent) => {
    if (read(event) !== null) event.preventDefault()
  }
  const onAuxClick = (event: MouseEvent) => {
    const action = read(event)
    if (action === null) return
    event.preventDefault()
    act(action)
  }
  onMounted(() => {
    window.addEventListener('mousedown', onMouseDown)
    window.addEventListener('auxclick', onAuxClick)
  })
  onUnmounted(() => {
    window.removeEventListener('mousedown', onMouseDown)
    window.removeEventListener('auxclick', onAuxClick)
  })
}
