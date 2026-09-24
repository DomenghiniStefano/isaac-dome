import { describe, expect, it } from 'vitest'
import { TabOrigin } from '@/lib/shell/tabs'
import {
  RouteName,
  WikiCategory,
  defaultLocation,
  locationTitle,
  routeOrigin,
  routePath,
} from './routeTable'
import type { TabLocation } from './routeTable'

describe('the location the app opens on', () => {
  it('is Completion, with no query to narrow it', () => {
    // A first launch with no saved tabs lands here, and so does the `/` redirect. The
    // matrix is the one reading that is whole the moment a save is picked: it needs no
    // graph, no catalog and no queue, so it is the screen least able to open on nothing.
    expect(defaultLocation).toEqual({ name: RouteName.Completion })
  })
})

describe('locationTitle', () => {
  it("is the route's title", () => {
    expect(locationTitle({ name: RouteName.Profile })).toBe('routes.profile')
  })

  it("is the category's title for a wiki category", () => {
    expect(
      locationTitle({
        name: RouteName.Wiki,
        query: { category: WikiCategory.Bosses },
      }),
    ).toBe('wikiCategories.bosses')
  })

  it('is the wiki title for the wiki without a category', () => {
    expect(locationTitle({ name: RouteName.Wiki })).toBe('routes.wiki')
  })
})

describe('the search location', () => {
  it('is a place of its own, with the query in it', () => {
    expect(routePath[RouteName.Search]).toBe('/search')
    expect(routeOrigin[RouteName.Search]).toBe(TabOrigin.Search)
    // It needs no profile: the gate is only for the Progress origin.
    expect(routeOrigin[RouteName.Search]).not.toBe(TabOrigin.Progress)
    const location: TabLocation = {
      name: RouteName.Search,
      query: { q: 'brimstone' },
    }
    expect(locationTitle(location)).toBe('routes.search')
  })
})
