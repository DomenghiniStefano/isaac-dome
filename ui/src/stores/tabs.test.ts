import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { RouteName } from '@/router/routeTable'
import { useTabsStore } from './tabs'

beforeEach(() => {
  setActivePinia(createPinia())
})

// The app's one gesture: a click replaces what the active tab shows, Ctrl opens it beside.
describe('go: here, or in a new tab', () => {
  const seeded = () => {
    const tabs = useTabsStore()
    tabs.seed(
      [{ entries: [{ location: { name: RouteName.Floor } }], index: 0 }],
      0,
    )
    return tabs
  }

  it('without the modifier, moves the active tab and opens none', () => {
    const tabs = seeded()
    tabs.go({ name: RouteName.Unlock }, false)
    expect(tabs.tabs).toHaveLength(1)
    expect(tabs.location).toEqual({ name: RouteName.Unlock })
  })

  it('with the modifier, opens a second tab on the location', () => {
    const tabs = seeded()
    const first = tabs.activeId
    tabs.go({ name: RouteName.Unlock }, true)
    expect(tabs.tabs).toHaveLength(2)
    expect(tabs.location).toEqual({ name: RouteName.Unlock })
    // The tab the click came from still shows what it showed.
    const origin = tabs.tabs.find((tab) => tab.id === first)
    expect(origin?.entries[origin.index]?.location).toEqual({
      name: RouteName.Floor,
    })
  })
})
