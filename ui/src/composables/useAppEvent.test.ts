import { describe, expect, it, vi } from 'vitest'
import { effectScope } from 'vue'

const unlisten = vi.fn()
const pending: { resolve: () => void } = { resolve: () => undefined }
const watchAppEvent = vi.fn(
  () =>
    new Promise<() => void>((resolve) => {
      pending.resolve = () => resolve(unlisten)
    }),
)
vi.mock('@/lib/window/appEvents', async (original) => ({
  ...(await original<typeof import('@/lib/window/appEvents')>()),
  watchAppEvent,
}))

const { AppEvent } = await import('@/lib/window/appEvents')
const { useAppEvent } = await import('./useAppEvent')

const settle = () => new Promise((resolve) => setTimeout(resolve, 0))

describe('useAppEvent', () => {
  // Card #80, R8: the view is gone before `listen` answered, and the listener must go with it.
  it('detaches a listener that resolves after its scope was disposed', async () => {
    const scope = effectScope()
    const handler = () => undefined
    scope.run(() => useAppEvent(AppEvent.RollChanged, handler))
    expect(watchAppEvent).toHaveBeenCalledWith(AppEvent.RollChanged, handler)
    scope.stop()
    pending.resolve()
    await settle()
    expect(unlisten).toHaveBeenCalledTimes(1)
  })
})
