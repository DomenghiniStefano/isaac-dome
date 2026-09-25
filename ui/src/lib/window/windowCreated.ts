// A window being created, as far as waiting for it goes: `WebviewWindow` is one, and so is a
// test's fake.
interface Creating {
  once: (
    event: string,
    handler: (e: { payload: unknown }) => void,
  ) => Promise<unknown>
}

/**
 * Settles when Tauri has made the window, or said it could not. Waiting on `tauri://created`
 * alone is waiting for ever on a window that failed: whoever awaited it — a drag, a tear-off —
 * hangs with it.
 */
export const windowCreated = (w: Creating): Promise<void> =>
  new Promise<void>((resolve, reject) => {
    void w.once('tauri://created', () => resolve())
    void w.once('tauri://error', (e) => reject(new Error(String(e.payload))))
  })
