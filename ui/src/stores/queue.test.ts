import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { resetFixtures } from '@/lib/ipc/fixtures'
// The queue's fixtures read the Unlock fixture's nodes, which the fixtures import on first use,
// and parsing the recorded payload is not free. Imported here, the cost is paid while the file
// loads, not inside the first test's five seconds.
import '@/lib/ipc/fixtures/graph'
import { LoadStatus } from './loadStatus'
import { useQueueStore } from './queue'

// Through the development fixtures, as the browser runs them: Vitest is a development build with
// no Tauri, so every command answers from lib/ipc/fixtures.
const ids = (store: ReturnType<typeof useQueueStore>) =>
  store.view?.rows.map((r) =>
    r.node.achievement.kind === 'known' ? r.node.achievement.id : null,
  )

beforeEach(() => {
  setActivePinia(createPinia())
  resetFixtures()
})

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('useQueueStore', () => {
  it('loads the queue', async () => {
    const store = useQueueStore()
    await store.load()
    expect(store.status).toBe(LoadStatus.Ready)
    expect(ids(store)).toEqual([480, 55, 69])
  })

  it("replaces the view with a move's answer and remembers the move", async () => {
    const store = useQueueStore()
    await store.load()
    await store.move(480, 69)
    expect(ids(store)).toEqual([69, 480, 55])
    expect(store.lastMove).toEqual({ achievement: 480, after: 69 })
    expect(store.busy).toBe(false)
  })

  it('forgets the last move once something else is written', async () => {
    const store = useQueueStore()
    await store.load()
    await store.move(55, null)
    await store.add(484)
    expect(store.lastMove).toBeNull()
    expect(ids(store)).toEqual([480, 55, 69, 484])
  })

  it('keeps the view and says why when a write is refused', async () => {
    const store = useQueueStore()
    await store.load()
    vi.stubGlobal('location', { search: '?catalog=none' })
    await store.add(484)
    expect(ids(store)).toEqual([480, 55, 69])
    expect(store.mutationFailed).toBe(true)
    expect(store.mutationError).toEqual({ kind: 'catalogUnavailable' })
    expect(store.busy).toBe(false)
  })

  // A refused write's reason stays on screen until the next write has answered: it is the
  // answer that clears it, not the asking.
  it('keeps the last refusal while the next write is in flight, and clears it once it lands', async () => {
    const store = useQueueStore()
    await store.load()
    vi.stubGlobal('location', { search: '?catalog=none' })
    await store.add(484)
    vi.unstubAllGlobals()
    const next = store.add(484)
    expect(store.busy).toBe(true)
    expect(store.mutationFailed).toBe(true)
    expect(store.mutationError).toEqual({ kind: 'catalogUnavailable' })
    await next
    expect(store.busy).toBe(false)
    expect(store.mutationFailed).toBe(false)
    expect(store.mutationError).toBeNull()
  })
})
