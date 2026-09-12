import { describe, expect, it } from 'vitest'
import { RouteName, WikiCategory } from '@/router/routeTable'
import { NavSection } from './navSection'
import {
  SidebarSection,
  isEntryActive,
  navSectionOf,
  sectionOfOrigin,
  sidebarEntries,
  sidebarSectionOf,
} from './sectionNav'
import { TabOrigin } from './tabs'

describe('sectionNav', () => {
  it('puts About under Settings', () => {
    expect(sectionOfOrigin(TabOrigin.About)).toBe(SidebarSection.Settings)
  })

  it('marks no navbar section while browsing Settings', () => {
    expect(navSectionOf(SidebarSection.Settings)).toBeNull()
    expect(navSectionOf(SidebarSection.Wiki)).toBe(NavSection.Wiki)
    expect(sidebarSectionOf(NavSection.Progress)).toBe(SidebarSection.Progress)
  })

  it('lists the seven Progress screens', () => {
    expect(sidebarEntries[SidebarSection.Progress]).toHaveLength(7)
  })

  it('matches an entry by route', () => {
    const [nextSteps] = sidebarEntries[SidebarSection.Progress]
    expect(
      nextSteps && isEntryActive(nextSteps, { name: RouteName.NextSteps }),
    ).toBe(true)
    expect(
      nextSteps && isEntryActive(nextSteps, { name: RouteName.Plan }),
    ).toBe(false)
    expect(nextSteps && isEntryActive(nextSteps, undefined)).toBe(false)
  })

  it('matches a wiki entry by category', () => {
    const bosses = sidebarEntries[SidebarSection.Wiki].find(
      (e) => e.location.query?.category === WikiCategory.Bosses,
    )
    const at = (category: WikiCategory) => ({
      name: RouteName.Wiki,
      query: { category },
    })
    expect(bosses && isEntryActive(bosses, at(WikiCategory.Bosses))).toBe(true)
    expect(bosses && isEntryActive(bosses, at(WikiCategory.Items))).toBe(false)
  })
})

describe('the wiki overview', () => {
  it('is the first wiki entry, the bare route, lit only there', () => {
    const [overview] = sidebarEntries[SidebarSection.Wiki]
    expect(overview?.location).toEqual({ name: RouteName.Wiki })
    expect(overview && isEntryActive(overview, { name: RouteName.Wiki })).toBe(
      true,
    )
    expect(
      overview &&
        isEntryActive(overview, {
          name: RouteName.Wiki,
          query: { category: WikiCategory.Items },
        }),
    ).toBe(false)
  })
})
