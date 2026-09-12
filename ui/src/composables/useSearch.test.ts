import { describe, expect, it, vi } from 'vitest'
import type { SearchView } from '@/lib/ipc/types'
import { searchFrom } from './useSearch'

const view = (query: string): SearchView => ({
  query,
  hits: [],
  total: 0,
  diagnostics: [],
})

describe('searchFrom', () => {
  it('keeps the answer to the last question, whatever order they come back in', async () => {
    // The slow first answer must not overwrite the fast second one.
    let release: (v: SearchView) => void = () => {}
    const pending = new Promise<SearchView>((resolve) => {
      release = resolve
    })
    const call = vi
      .fn()
      .mockImplementationOnce(() => pending)
      .mockImplementationOnce(() => Promise.resolve(view('br')))
    const state = searchFrom(call, 10)
    const slow = state.run('b')
    await state.run('br')
    expect(state.view.value?.query).toBe('br')
    release(view('b'))
    await slow
    expect(state.view.value?.query).toBe('br')
  })

  it('an empty query clears the answer without asking', async () => {
    const call = vi.fn()
    const state = searchFrom(call, 10)
    await state.run('   ')
    expect(call).not.toHaveBeenCalled()
    expect(state.view.value).toBeNull()
  })
})
