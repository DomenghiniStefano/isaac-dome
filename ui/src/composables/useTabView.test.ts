import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { effectScope, nextTick } from 'vue'
import type { Ref } from 'vue'
import { RouteName } from '@/router/routeTable'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { useTabsStore } from '@/stores/tabs'
import { useTabView } from './useTabView'

interface Reading {
  sort: string
  query?: string
}

const spec: TabViewSpec<Reading> = {
  empty: () => ({ sort: 'fanOut' }),
  read: (value) =>
    typeof value === 'object' && value !== null && 'sort' in value
      ? { sort: String((value as { sort: unknown }).sort) }
      : null,
}

// No component: the composable uses `ref` and `watch` and no lifecycle hook, so a scope is
// everything it needs. Mounting would want a DOM and a dependency this repo does not have.
const inScopeWithUpdate = () => {
  const scope = effectScope()
  const view = scope.run(() => useTabView(spec))
  if (!view) throw new Error('the scope ran nothing')
  return view
}
const inScope = (): Ref<Reading> => inScopeWithUpdate().reading

const tabs = () => useTabsStore()

beforeEach(() => {
  setActivePinia(createPinia())
  vi.useFakeTimers()
})

describe('a screen reaching its own reading', () => {
  it('opens on the empty reading when the entry has none', () => {
    tabs().seed(
      [{ entries: [{ location: { name: RouteName.Unlock } }], index: 0 }],
      0,
    )
    expect(inScope().value).toEqual({ sort: 'fanOut' })
  })

  it('opens on what the entry carries', () => {
    tabs().seed(
      [
        {
          entries: [
            { location: { name: RouteName.Unlock }, view: { sort: 'name' } },
          ],
          index: 0,
        },
      ],
      0,
    )
    expect(inScope().value).toEqual({ sort: 'name' })
  })

  it('opens on the empty reading when the record cannot be read', () => {
    tabs().seed(
      [
        {
          entries: [
            { location: { name: RouteName.Unlock }, view: { nothing: 1 } },
          ],
          index: 0,
        },
      ],
      0,
    )
    expect(inScope().value).toEqual({ sort: 'fanOut' })
  })

  it('writes what changed back into the entry, once', async () => {
    const store = tabs()
    store.seed(
      [{ entries: [{ location: { name: RouteName.Unlock } }], index: 0 }],
      0,
    )
    const reading = inScope()
    reading.value = { sort: 'name' }
    reading.value = { sort: 'id' }
    await nextTick()
    // Nothing written yet: a burst of changes is one write, as the session's own is.
    expect(store.active?.entries[0]?.view).toBeUndefined()
    await vi.runAllTimersAsync()
    expect(store.active?.entries[0]?.view).toEqual({ sort: 'id' })
  })

  // Going back is going back to what you were looking at. The screen stays mounted when the
  // route does not change, so the composable — not the component's lifecycle — is what has to
  // notice.
  it('re-reads when the tab moves to another entry under it', async () => {
    const store = tabs()
    store.seed(
      [
        {
          entries: [
            { location: { name: RouteName.Unlock }, view: { sort: 'name' } },
            { location: { name: RouteName.Unlock, query: { q: 'brim' } } },
          ],
          index: 1,
        },
      ],
      0,
    )
    const reading = inScope()
    expect(reading.value).toEqual({ sort: 'fanOut' })
    store.back()
    await nextTick()
    expect(reading.value).toEqual({ sort: 'name' })
  })

  it('an update moves one field and keeps the rest', () => {
    tabs().seed(
      [
        {
          entries: [
            { location: { name: RouteName.Unlock }, view: { sort: 'name' } },
          ],
          index: 0,
        },
      ],
      0,
    )
    const { reading, update } = inScopeWithUpdate()
    update({ query: 'brim' })
    expect(reading.value).toEqual({ sort: 'name', query: 'brim' })
    update({ sort: 'id' })
    expect(reading.value).toEqual({ sort: 'id', query: 'brim' })
  })
})
