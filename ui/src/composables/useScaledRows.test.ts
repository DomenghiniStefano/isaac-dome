import { describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import { remeasureOnScale } from './useScaledRows'

describe('remeasureOnScale', () => {
  it('remeasures when the interface changes size, and only then', async () => {
    // A virtualizer keeps the sizes it measured: change the scale with the table already on
    // screen and the rows stay the old number of pixels apart while they are drawn the new
    // height — they overlap, and nothing fails. The size is a token, so the scale has to say
    // when to measure again.
    const scale = ref(100)
    const measure = vi.fn()
    remeasureOnScale(scale, measure)
    expect(measure).not.toHaveBeenCalled()
    scale.value = 200
    await Promise.resolve()
    expect(measure).toHaveBeenCalledTimes(1)
    scale.value = 200
    await Promise.resolve()
    expect(measure).toHaveBeenCalledTimes(1)
  })
})
