import { afterEach, describe, expect, it, vi } from 'vitest'
import { warnOnce } from './warnOnce'

afterEach(() => {
  vi.restoreAllMocks()
})

describe('warnOnce', () => {
  it('says its message the first time and never again', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const declare = warnOnce('fixture: made up')
    declare()
    declare()
    declare()
    expect(warn).toHaveBeenCalledTimes(1)
    expect(warn).toHaveBeenCalledWith('fixture: made up')
  })

  it('says nothing until it is used', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    warnOnce('fixture: never used')
    expect(warn).not.toHaveBeenCalled()
  })

  it('each fixture keeps its own once', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    warnOnce('first')()
    warnOnce('second')()
    expect(warn.mock.calls).toEqual([['first'], ['second']])
  })
})
