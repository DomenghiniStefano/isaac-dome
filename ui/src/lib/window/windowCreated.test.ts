import { describe, expect, it } from 'vitest'
import { windowCreated } from './windowCreated'

// A window being created, as far as `windowCreated` can see one: two one-shot events.
const creating = () => {
  const handlers = new Map<string, (e: { payload: unknown }) => void>()
  return {
    once: (event: string, handler: (e: { payload: unknown }) => void) => {
      handlers.set(event, handler)
      return Promise.resolve(() => undefined)
    },
    emit: (event: string, payload: unknown = null) =>
      handlers.get(event)?.({ payload }),
  }
}

describe('windowCreated', () => {
  it('resolves when the window is created', async () => {
    const w = creating()
    const done = windowCreated(w)
    w.emit('tauri://created')
    await expect(done).resolves.toBeUndefined()
  })

  // Card #80, R8: `warmPreview` waited for `created` alone, so a window Tauri refused to make
  // left the drag waiting on a promise that would never settle.
  it('rejects when Tauri says it could not create it', async () => {
    const w = creating()
    const done = windowCreated(w)
    w.emit('tauri://error', 'a webview with this label already exists')
    await expect(done).rejects.toThrow(
      'a webview with this label already exists',
    )
  })
})
