import { describe, expect, it } from 'vitest'
import { ref } from 'vue'
import type { IpcError } from '@/lib/ipc/types'
import { attempt, clearFailure } from './tracked'

const refused: IpcError = { kind: 'catalogUnavailable' }

// A write whose failure is said, never thrown: the two refs say how the last one ended.
describe('attempt', () => {
  it('a write that lands clears what the last one said and answers true', async () => {
    const failed = ref(true)
    const error = ref<IpcError | null>(refused)
    const landed = await attempt(failed, error, async () => {})
    expect(landed).toBe(true)
    expect(failed.value).toBe(false)
    expect(error.value).toBeNull()
  })

  it('a refused write says why and answers false, without throwing', async () => {
    const failed = ref(false)
    const error = ref<IpcError | null>(null)
    const landed = await attempt(failed, error, () => Promise.reject(refused))
    expect(landed).toBe(false)
    expect(failed.value).toBe(true)
    expect(error.value).toEqual(refused)
  })

  it('a failure that is not ours fails with no error to name', async () => {
    const failed = ref(false)
    const error = ref<IpcError | null>(refused)
    const landed = await attempt(failed, error, () =>
      Promise.reject(new Error('boom')),
    )
    expect(landed).toBe(false)
    expect(failed.value).toBe(true)
    expect(error.value).toBeNull()
  })

  it('touches nothing while the write is in flight', async () => {
    const failed = ref(true)
    const error = ref<IpcError | null>(refused)
    const seen: [boolean, IpcError | null][] = []
    await attempt(failed, error, async () => {
      seen.push([failed.value, error.value])
    })
    expect(seen).toEqual([[true, refused]])
  })

  it('a store with no flag keeps only the error', async () => {
    const error = ref<IpcError | null>(null)
    expect(await attempt(null, error, () => Promise.reject(refused))).toBe(
      false,
    )
    expect(error.value).toEqual(refused)
  })
})

describe('clearFailure', () => {
  it('forgets the flag and the error', () => {
    const failed = ref(true)
    const error = ref<IpcError | null>(refused)
    clearFailure(failed, error)
    expect(failed.value).toBe(false)
    expect(error.value).toBeNull()
  })

  it('with no flag, forgets the error', () => {
    const error = ref<IpcError | null>(refused)
    clearFailure(null, error)
    expect(error.value).toBeNull()
  })
})
