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
