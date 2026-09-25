import { describe, expect, it } from 'vitest'
import { AutostartFailure } from './types'
import type { IpcError } from './types'
import { asIpcError, isIpcError } from './errors'

describe('isIpcError', () => {
  it('accepts a rejection carrying one of the contract’s tags', () => {
    const errors: IpcError[] = [
      { kind: 'noActiveProfile' },
      { kind: 'unknownProfile', id: 'p1' },
      { kind: 'sessionTooLarge' },
      { kind: 'autostartNotWritable', reason: AutostartFailure.WriteRefused },
      { kind: 'updateNotReady' },
    ]
    expect(errors.every(isIpcError)).toBe(true)
  })

  // A plugin's own error, or anything else shaped like a tagged object, is not ours: its
  // sentence would be one of ours said about something else.
  it('refuses an object whose kind the contract does not declare', () => {
    expect(isIpcError({ kind: 'notOurs' })).toBe(false)
    expect(isIpcError({ kind: 42 })).toBe(false)
  })

  it('refuses what is not a tagged object at all', () => {
    expect(isIpcError(new Error('boom'))).toBe(false)
    expect(isIpcError('noActiveProfile')).toBe(false)
    expect(isIpcError(null)).toBe(false)
    expect(isIpcError(undefined)).toBe(false)
  })

  // `hasOwn`, not `in`: a key inherited from `Object.prototype` is no tag of ours.
  it('refuses a kind that only the prototype has', () => {
    expect(isIpcError({ kind: 'toString' })).toBe(false)
    expect(isIpcError({ kind: 'constructor' })).toBe(false)
  })
})

describe('asIpcError', () => {
  it('keeps ours as it came', () => {
    const e: IpcError = { kind: 'unknownProfile', id: 'p1' }
    expect(asIpcError(e)).toBe(e)
  })

  it('turns anything else into null', () => {
    expect(asIpcError(new Error('boom'))).toBeNull()
    expect(asIpcError({ kind: 'notOurs' })).toBeNull()
  })
})
