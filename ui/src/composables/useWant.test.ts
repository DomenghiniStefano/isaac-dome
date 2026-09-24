import { WantDiagnostic } from '@/lib/ipc/types'
import { describe, expect, it } from 'vitest'
import { nextTick, ref } from 'vue'
import type { Target, WantView } from '@/lib/ipc/types'
import { LoadStatus } from '@/stores/loadStatus'
import { wantFrom } from './useWant'

const answer = (kind: string): WantView => ({
  wanted: { kind: 'unresolved' },
  routes: [],
  diagnostics: [WantDiagnostic.NothingUnlocks],
  // The tag is what the test reads back; the rest is the smallest legal view.
  ...({ probe: kind } as object),
})

describe('wantFrom', () => {
  it('asks for the want it was given and reports ready', async () => {
    const target = ref<Target | null>({ kind: 'item', id: 105 })
    const asked: Target[] = []
    const { view, status } = wantFrom(async (t) => {
      asked.push(t)
      return answer('first')
    }, target)
    await nextTick()
    await new Promise((r) => setTimeout(r, 0))
    expect(asked).toEqual([{ kind: 'item', id: 105 }])
    expect(view.value).not.toBeNull()
    expect(status.value).toBe(LoadStatus.Ready)
  })

  // The failure this guards: you type a second want while the first is still in flight, and
  // the slower answer lands last and describes a question you are no longer asking.
  it('drops an answer to a want that is no longer the one asked', async () => {
    const target = ref<Target | null>({ kind: 'item', id: 1 })
    let release: (v: WantView) => void = () => {}
    const { view } = wantFrom(
      (t) =>
        t.kind === 'item' && t.id === 1
          ? new Promise<WantView>((r) => {
              release = r
            })
          : Promise.resolve(answer('second')),
      target,
    )
    await nextTick()
    target.value = { kind: 'item', id: 2 }
    await nextTick()
    await new Promise((r) => setTimeout(r, 0))
    const current = view.value
    release(answer('first'))
    await new Promise((r) => setTimeout(r, 0))
    expect(view.value).toBe(current)
  })

  it('clears itself when nothing is wanted', async () => {
    const target = ref<Target | null>({ kind: 'item', id: 105 })
    const { view, status } = wantFrom(async () => answer('first'), target)
    await new Promise((r) => setTimeout(r, 0))
    target.value = null
    await nextTick()
    await new Promise((r) => setTimeout(r, 0))
    expect(view.value).toBeNull()
    expect(status.value).toBe(LoadStatus.Idle)
  })
})
