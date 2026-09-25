import { describe, expect, it } from 'vitest'
import { RouteName, WikiCategory } from '@/router/routeTable'
import { TabOrigin } from './tabs'
import { tabViewOf } from './tabViews'

const t = (key: string): string => `«${key}»`
const titles = new Map([['item:105', 'The D6']])
const titleOf = (key: string): string | null => titles.get(key) ?? null

describe('tabViewOf', () => {
  it("names a screen tab by its route's message, and says where it comes from", () => {
    const view = tabViewOf(
      {
        id: 'tab-1',
        entries: [{ location: { name: RouteName.Unlock } }],
        index: 0,
      },
      titleOf,
      t,
    )
    expect(view).toEqual({
      id: 'tab-1',
      label: '«routes.unlock»',
      origin: TabOrigin.Progress,
    })
  })

  it("names a page tab by the page's title once the index knows it", () => {
    const view = tabViewOf(
      {
        id: 'tab-2',
        entries: [
          {
            location: {
              name: RouteName.Wiki,
              query: { category: WikiCategory.Items, page: 'item:105' },
            },
          },
        ],
        index: 0,
      },
      titleOf,
      t,
    )
    expect(view.label).toBe('The D6')
    expect(view.origin).toBe(TabOrigin.Wiki)
  })

  it('reads the entry the tab is showing, not the last one it reached', () => {
    const view = tabViewOf(
      {
        id: 'tab-3',
        entries: [
          { location: { name: RouteName.Unlock } },
          { location: { name: RouteName.Search } },
        ],
        index: 0,
      },
      titleOf,
      t,
    )
    expect(view.label).toBe('«routes.unlock»')
  })
})
