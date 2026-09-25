import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { IpcError, UpdateView } from '@/lib/ipc/types'
import { UpdateReason } from '@/lib/ipc/types'
import { useUpdateStore } from './update'

const ipc = vi.hoisted(() => ({
  updateStatus: vi.fn(),
  checkUpdate: vi.fn(),
  installUpdate: vi.fn(),
}))
vi.mock('@/lib/ipc/update', () => ipc)

const held: UpdateView = {
  currentVersion: '0.1.3',
  phase: { kind: 'idle' },
  unavailable: null,
}
const found: UpdateView = {
  currentVersion: '0.1.3',
  phase: { kind: 'upToDate' },
  unavailable: null,
}
const refused: IpcError = { kind: 'catalogUnavailable' }

beforeEach(() => {
  setActivePinia(createPinia())
  vi.resetAllMocks()
  ipc.updateStatus.mockResolvedValue(held)
})

describe('useUpdateStore, the check button', () => {
  it('takes the answer, and reads nothing else', async () => {
    ipc.checkUpdate.mockResolvedValue(found)
    const store = useUpdateStore()
    await store.check()
    expect(store.view).toEqual(found)
    expect(store.error).toBeNull()
    expect(ipc.updateStatus).not.toHaveBeenCalled()
  })

  it('a refused check says why and reads the held phase again', async () => {
    ipc.checkUpdate.mockRejectedValue(refused)
    const store = useUpdateStore()
    await store.check()
    expect(store.error).toEqual(refused)
    expect(store.view).toEqual(held)
  })

  it('a check forgets the last failure as it starts', async () => {
    ipc.checkUpdate.mockRejectedValueOnce(refused)
    const store = useUpdateStore()
    await store.check()
    const during: (IpcError | null)[] = []
    ipc.checkUpdate.mockImplementation(async () => {
      during.push(store.error)
      return found
    })
    await store.check()
    expect(during).toEqual([null])
    expect(store.error).toBeNull()
  })
})

describe('useUpdateStore, the install button', () => {
  it('a refused install says why, and the phase is read again either way', async () => {
    ipc.installUpdate.mockRejectedValue(refused)
    const store = useUpdateStore()
    await store.install()
    expect(store.error).toEqual(refused)
    expect(ipc.updateStatus).toHaveBeenCalledTimes(1)
  })

  it('an install that returns reads the phase again and names no error', async () => {
    ipc.installUpdate.mockResolvedValue(undefined)
    const store = useUpdateStore()
    store.error = refused
    await store.install()
    expect(store.error).toBeNull()
    expect(ipc.updateStatus).toHaveBeenCalledTimes(1)
  })

  it('an unanswered read offers nothing, keeping the version it knew', async () => {
    ipc.installUpdate.mockRejectedValue(refused)
    ipc.updateStatus.mockRejectedValue(new Error('gone'))
    const store = useUpdateStore()
    await store.install()
    expect(store.view).toEqual({
      currentVersion: '',
      phase: { kind: 'idle' },
      unavailable: UpdateReason.NotSupported,
    })
  })
})
