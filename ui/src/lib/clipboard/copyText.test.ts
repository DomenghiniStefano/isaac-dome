import { afterEach, describe, expect, it, vi } from 'vitest'
import { copyText } from './copyText'

describe('copyText', () => {
  afterEach(() => vi.unstubAllGlobals())

  it('answers true when the clipboard took the text', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    vi.stubGlobal('navigator', { clipboard: { writeText } })
    await expect(copyText('YKF6 QDN6')).resolves.toBe(true)
    expect(writeText).toHaveBeenCalledWith('YKF6 QDN6')
  })

  // A refusal is an answer, never an exception: the button says it did not copy, and the page
  // around it carries on.
  it('answers false when the clipboard refused', async () => {
    vi.stubGlobal('navigator', {
      clipboard: { writeText: vi.fn().mockRejectedValue(new Error('denied')) },
    })
    await expect(copyText('YKF6 QDN6')).resolves.toBe(false)
  })

  it('answers false where there is no clipboard at all', async () => {
    vi.stubGlobal('navigator', {})
    await expect(copyText('YKF6 QDN6')).resolves.toBe(false)
  })
})
