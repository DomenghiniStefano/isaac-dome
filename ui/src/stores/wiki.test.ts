import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type {
  Entry,
  IpcError,
  PoolMembershipView,
  Target,
  WikiIndex,
} from '@/lib/ipc/types'

const wikiIndex = vi.fn<() => Promise<WikiIndex>>()
const wikiEntry = vi.fn<(target: Target) => Promise<Entry | null>>()
const wikiItemPools =
  vi.fn<(target: Target) => Promise<PoolMembershipView[] | null>>()
vi.mock('@/lib/ipc/wiki', () => ({ wikiIndex, wikiEntry, wikiItemPools }))

const { LoadStatus } = await import('./loadStatus')
const { useWikiStore } = await import('./wiki')

const d6: Target = { kind: 'item', id: 105 }
const d6Key = 'item:105'
const refusal: IpcError = { kind: 'wikiUnavailable' }
const index = { pages: [], info: { kind: 'missing', reason: 'malformed' } }

beforeEach(() => {
  setActivePinia(createPinia())
  wikiIndex.mockReset()
  wikiEntry.mockReset()
  wikiItemPools.mockReset()
})

// Card #80, R9: a page that would not load marked the whole index `Failed`, and every screen
// that reads the wiki's status — the landing, the lists, every other page — showed the failure.
describe('a page that does not load', () => {
  it('leaves the index as it was', async () => {
    wikiIndex.mockResolvedValue(index as unknown as WikiIndex)
    wikiEntry.mockRejectedValue(refusal)
    const wiki = useWikiStore()
    await wiki.loadIndex()
    await wiki.loadEntry(d6)
    expect(wiki.status).toBe(LoadStatus.Ready)
    expect(wiki.error).toBeNull()
  })

  it('has its own error', async () => {
    wikiEntry.mockRejectedValue(refusal)
    const wiki = useWikiStore()
    await wiki.loadEntry(d6)
    expect(wiki.pageFailed(d6Key)).toBe(true)
    expect(wiki.pageError(d6Key)).toEqual(refusal)
    expect(wiki.entry(d6Key)).toBeUndefined()
  })

  it('is read again when asked again, and forgets the failure once it loads', async () => {
    wikiEntry.mockRejectedValueOnce(refusal).mockResolvedValueOnce(null)
    const wiki = useWikiStore()
    await wiki.loadEntry(d6)
    await wiki.loadEntry(d6)
    expect(wikiEntry).toHaveBeenCalledTimes(2)
    expect(wiki.pageFailed(d6Key)).toBe(false)
    expect(wiki.entry(d6Key)).toBeNull()
  })

  it('keeps no error of a shape that is not ours, and still says it failed', async () => {
    wikiEntry.mockRejectedValue(new Error('boom'))
    const wiki = useWikiStore()
    await wiki.loadEntry(d6)
    expect(wiki.pageFailed(d6Key)).toBe(true)
    expect(wiki.pageError(d6Key)).toBeNull()
  })
})

describe('a page that loads', () => {
  it('is read once', async () => {
    wikiEntry.mockResolvedValue(null)
    const wiki = useWikiStore()
    await wiki.loadEntry(d6)
    await wiki.loadEntry(d6)
    expect(wikiEntry).toHaveBeenCalledTimes(1)
    expect(wiki.pageFailed(d6Key)).toBe(false)
  })
})

// Design decision 4: the item's pools come from the game, joined at the IPC boundary, not
// from the wiki entry itself.
describe("an item's pools", () => {
  it('are undefined until asked, then cached once read', async () => {
    wikiItemPools.mockResolvedValue([
      { label: 'Treasure Room (Item Pool)', target: null, weight: 1 },
    ])
    const wiki = useWikiStore()
    expect(wiki.poolsFor(d6Key)).toBeUndefined()
    await wiki.loadPools(d6)
    await wiki.loadPools(d6)
    expect(wikiItemPools).toHaveBeenCalledTimes(1)
    expect(wiki.poolsFor(d6Key)).toEqual([
      { label: 'Treasure Room (Item Pool)', target: null, weight: 1 },
    ])
  })

  it('cache `null` as a real answer — no game, or no pools concept at all', async () => {
    wikiItemPools.mockResolvedValue(null)
    const wiki = useWikiStore()
    await wiki.loadPools(d6)
    expect(wiki.poolsFor(d6Key)).toBeNull()
  })

  it('leave nothing cached on a failed read, so the next ask tries again', async () => {
    wikiItemPools.mockRejectedValueOnce(new Error('boom'))
    const wiki = useWikiStore()
    await wiki.loadPools(d6)
    expect(wiki.poolsFor(d6Key)).toBeUndefined()
    wikiItemPools.mockResolvedValueOnce(null)
    await wiki.loadPools(d6)
    expect(wikiItemPools).toHaveBeenCalledTimes(2)
    expect(wiki.poolsFor(d6Key)).toBeNull()
  })
})
