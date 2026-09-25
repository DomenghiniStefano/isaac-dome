import { describe, expect, it, vi } from 'vitest'
import { subscription, subscriptions } from './subscriptions'

// A subscribe that resolves when the test says so, the way `listen` resolves a beat after it is
// asked: the window between the two is where a view can unmount (card #80, R8).
const deferredStart = () => {
  const unlisten = vi.fn()
  const pending: { resolve: () => void } = { resolve: () => undefined }
  const start = () =>
    new Promise<() => void>((resolve) => {
      pending.resolve = () => resolve(unlisten)
    })
  return { start, unlisten, resolve: () => pending.resolve() }
}

const settle = () => new Promise((resolve) => setTimeout(resolve, 0))

describe('one subscription', () => {
  it('detaches when stopped after it started', async () => {
    const s = deferredStart()
    const stop = subscription(s.start)
    s.resolve()
    await settle()
    stop()
    await settle()
    expect(s.unlisten).toHaveBeenCalledTimes(1)
  })

  it('detaches as soon as it starts, when stopped before', async () => {
    const s = deferredStart()
    const stop = subscription(s.start)
    stop()
    await settle()
    expect(s.unlisten).not.toHaveBeenCalled()
    s.resolve()
    await settle()
    expect(s.unlisten).toHaveBeenCalledTimes(1)
  })

  it('detaches once, however many times it is stopped', async () => {
    const s = deferredStart()
    const stop = subscription(s.start)
    s.resolve()
    stop()
    stop()
    await settle()
    expect(s.unlisten).toHaveBeenCalledTimes(1)
  })

  it('stopping one that failed to start is not an error', async () => {
    const stop = subscription(() => Promise.reject(new Error('no Tauri')))
    stop()
    await settle()
  })
})

describe('a set of subscriptions', () => {
  it('stops every one it holds', async () => {
    const [a, b] = [deferredStart(), deferredStart()]
    const set = subscriptions()
    set.add(a.start)
    set.add(b.start)
    a.resolve()
    b.resolve()
    set.stop()
    await settle()
    expect(a.unlisten).toHaveBeenCalledTimes(1)
    expect(b.unlisten).toHaveBeenCalledTimes(1)
  })

  // `useWindowSession` broadcasts `Ready` only once its listener is there to hear the answer:
  // adding one has to be something it can wait for.
  it('lets the caller wait until one has started', async () => {
    const s = deferredStart()
    const set = subscriptions()
    const state = { started: false }
    const added = set.add(s.start).then(() => {
      state.started = true
    })
    await settle()
    expect(state.started).toBe(false)
    s.resolve()
    await added
    expect(state.started).toBe(true)
  })

  // The shape of `useWindowSession`: the subscriptions are made one after another across
  // awaits, and the window can close between any two of them.
  it('stops one added after the set was stopped, the moment it starts', async () => {
    const late = deferredStart()
    const set = subscriptions()
    set.stop()
    set.add(late.start)
    late.resolve()
    await settle()
    expect(late.unlisten).toHaveBeenCalledTimes(1)
  })
})
