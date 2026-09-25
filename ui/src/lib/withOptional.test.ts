import { describe, expect, it } from 'vitest'
import { whenTrue, withOptional } from './withOptional'

describe('withOptional', () => {
  it('carries the field when it has a value', () => {
    expect({ a: 1, ...withOptional('box', { left: 0 }) }).toStrictEqual({
      a: 1,
      box: { left: 0 },
    })
  })

  it('leaves the key out when the value is undefined', () => {
    const built = { a: 1, ...withOptional('box', undefined) }
    expect(built).toStrictEqual({ a: 1 })
    expect('box' in built).toBe(false)
  })

  // Only `undefined` is absent: a zero, an empty string, `false` and `null` are values somebody
  // wrote, and dropping them would change what was stored.
  it('keeps every other falsy value', () => {
    expect(withOptional('n', 0)).toStrictEqual({ n: 0 })
    expect(withOptional('s', '')).toStrictEqual({ s: '' })
    expect(withOptional('b', false)).toStrictEqual({ b: false })
    expect(withOptional('z', null)).toStrictEqual({ z: null })
  })
})

describe('whenTrue', () => {
  it('keeps a switch that is on, and leaves one that is off absent', () => {
    expect({ ...withOptional('folded', whenTrue(true)) }).toStrictEqual({
      folded: true,
    })
    expect({ ...withOptional('folded', whenTrue(false)) }).toStrictEqual({})
  })
})
