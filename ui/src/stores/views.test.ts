import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { StoreId } from '@/lib/constants/stores'
import { LoadStatus } from './loadStatus'
import { defineViewStore } from './views'

beforeEach(() => {
  setActivePinia(createPinia())
})

describe('one load, written once', () => {
  it('starts idle, with nothing read and nothing wrong', () => {
    const useStore = defineViewStore(StoreId.Collection, () =>
      Promise.resolve(1),
    )
    const store = useStore()
    expect(store.status).toBe(LoadStatus.Idle)
    expect(store.view).toBeNull()
    expect(store.error).toBeNull()
  })

  it('holds what the loader answered', async () => {
    const useStore = defineViewStore(StoreId.Completion, () =>
      Promise.resolve({ n: 7 }),
    )
    const store = useStore()
    await store.load()
    expect(store.status).toBe(LoadStatus.Ready)
    expect(store.view).toEqual({ n: 7 })
  })

  // A view belongs to one profile. Loading clears it first, so it is never shown for a
  // moment under a profile it was not read from.
  it('clears the old view before asking for the new one', async () => {
    let seen: unknown = 'not called'
    const useStore = defineViewStore(StoreId.Graph, () => {
      seen = store.view
      return Promise.resolve('second')
    })
    const store = useStore()
    await store.load()
    expect(store.view).toBe('second')
    await store.load()
    expect(seen).toBeNull()
  })

  it('keeps an IpcError when the command refuses, and says it failed', async () => {
    const refusal = { kind: 'catalogUnavailable' } as const
    const useStore = defineViewStore(StoreId.Collection, () =>
      Promise.reject(refusal),
    )
    const store = useStore()
    await store.load()
    expect(store.status).toBe(LoadStatus.Failed)
    expect(store.error).toEqual(refusal)
    expect(store.view).toBeNull()
  })

  // A failure that is not an IpcError is still a failure: the status says so and the error
  // is null rather than some foreign object the UI would have to parse.
  it('fails without an error when what was thrown is not one of ours', async () => {
    const useStore = defineViewStore(StoreId.Completion, () =>
      Promise.reject(new Error('boom')),
    )
    const store = useStore()
    await store.load()
    expect(store.status).toBe(LoadStatus.Failed)
    expect(store.error).toBeNull()
  })
})

// A refresh is the same profile read again because something it depends on moved — the
// queue, for the suggestions. It keeps what is shown until the answer arrives, so the screen
// does not fall back to a skeleton for every row you add.
describe('a refresh of the same profile', () => {
  it('keeps the view while it reads, and replaces it with the answer', async () => {
    let answer = 'first'
    let seen: unknown = 'not called'
    const useStore = defineViewStore(StoreId.Graph, () => {
      seen = store.view
      return Promise.resolve(answer)
    })
    const store = useStore()
    await store.load()
    answer = 'second'
    await store.refresh()
    expect(seen).toBe('first')
    expect(store.view).toBe('second')
    expect(store.status).toBe(LoadStatus.Ready)
  })

  it('leaves the view and the status alone when the read fails', async () => {
    let fail = false
    const useStore = defineViewStore(StoreId.Graph, () =>
      fail ? Promise.reject(new Error('gone')) : Promise.resolve('first'),
    )
    const store = useStore()
    await store.load()
    fail = true
    await store.refresh()
    expect(store.view).toBe('first')
    expect(store.status).toBe(LoadStatus.Ready)
  })

  it('does nothing before a first load', async () => {
    let calls = 0
    const useStore = defineViewStore(StoreId.Graph, () => {
      calls += 1
      return Promise.resolve('x')
    })
    const store = useStore()
    await store.refresh()
    expect(calls).toBe(0)
    expect(store.view).toBeNull()
  })
})
