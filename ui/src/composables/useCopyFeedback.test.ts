import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { effectScope } from 'vue'
import { CopyFeedback, useCopyFeedback } from './useCopyFeedback'

describe('useCopyFeedback', () => {
  beforeEach(() => vi.useFakeTimers())
  afterEach(() => vi.useRealTimers())

  const setup = () => {
    const scope = effectScope()
    const feedback = scope.run(() => useCopyFeedback(1500))
    if (feedback === undefined) throw new Error('no scope')
    return { scope, feedback }
  }

  it('says copied, then goes back to idle', () => {
    const { feedback } = setup()
    feedback.report(true)
    expect(feedback.state.value).toBe(CopyFeedback.Copied)
    vi.advanceTimersByTime(1500)
    expect(feedback.state.value).toBe(CopyFeedback.Idle)
  })

  it('says it failed when the copy did not take', () => {
    const { feedback } = setup()
    feedback.report(false)
    expect(feedback.state.value).toBe(CopyFeedback.Failed)
  })

  // A second click restarts the wait: the first timer must not cut the second message short.
  it('keeps the second message its whole time', () => {
    const { feedback } = setup()
    feedback.report(true)
    vi.advanceTimersByTime(1000)
    feedback.report(true)
    vi.advanceTimersByTime(1000)
    expect(feedback.state.value).toBe(CopyFeedback.Copied)
    vi.advanceTimersByTime(500)
    expect(feedback.state.value).toBe(CopyFeedback.Idle)
  })

  // A page closed before the message is over leaves no timer behind it.
  it('leaves no timer when its scope ends', () => {
    const { scope, feedback } = setup()
    feedback.report(true)
    scope.stop()
    expect(vi.getTimerCount()).toBe(0)
  })
})
