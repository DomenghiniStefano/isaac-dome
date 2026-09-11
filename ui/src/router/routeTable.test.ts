import { describe, expect, it } from 'vitest'
import { RouteName, WikiCategory, locationTitle } from './routeTable'

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
