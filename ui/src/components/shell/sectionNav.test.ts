import { describe, expect, it } from 'vitest'
import { RouteName, WikiCategory, routeOrigin } from '@/router/routeTable'
import { NavSection } from './navSection'
import {
  SidebarSection,
  firstEntry,
  isEntryActive,
  navSectionOf,
  sectionOfOrigin,
  sidebarEntries,
  sidebarSectionOf,
} from './sectionNav'
import { TabOrigin } from './tabs'

describe('sectionNav', () => {
  it('puts a settings tab in the Settings sidebar', () => {
    expect(sectionOfOrigin(TabOrigin.Settings)).toBe(SidebarSection.Settings)
  })

  it('marks no navbar section while browsing Settings', () => {
    expect(navSectionOf(SidebarSection.Settings)).toBeNull()
    expect(navSectionOf(SidebarSection.Wiki)).toBe(NavSection.Wiki)
    expect(navSectionOf(SidebarSection.Tool)).toBe(NavSection.Tool)
    expect(sidebarSectionOf(NavSection.Progress)).toBe(SidebarSection.Progress)
    expect(sidebarSectionOf(NavSection.Tool)).toBe(SidebarSection.Tool)
  })

  it('matches an entry by route', () => {
    const [nextSteps] = sidebarEntries[SidebarSection.Progress]
    expect(
      nextSteps && isEntryActive(nextSteps, { name: RouteName.Goals }),
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

describe('the first entry of a section', () => {
  it('is where clicking the section goes (B24)', () => {
    expect(firstEntry(SidebarSection.Progress).location).toEqual({
      name: RouteName.Goals,
    })
    expect(firstEntry(SidebarSection.Wiki).location).toEqual({
      name: RouteName.Wiki,
    })
    expect(firstEntry(SidebarSection.Tool).location).toEqual({
      name: RouteName.Live,
    })
    expect(firstEntry(SidebarSection.Settings).location).toEqual({
      name: RouteName.Profile,
    })
  })
})

describe('a tab that belongs to no section', () => {
  it('leaves the sidebar where it was', () => {
    // Search sits above the three sections (DESIGN-BRIEF.md §4.2): it belongs to none.
    expect(sectionOfOrigin(TabOrigin.Search)).toBeNull()
    expect(sectionOfOrigin(TabOrigin.Wiki)).toBe(SidebarSection.Wiki)
  })
})

// The Wiki is not here: its entries are categories of one route, not one entry per route.
const routeSections: [SidebarSection, TabOrigin][] = [
  [SidebarSection.Progress, TabOrigin.Progress],
  [SidebarSection.Tool, TabOrigin.Tool],
  [SidebarSection.Settings, TabOrigin.Settings],
]

describe('nothing is reachable only by typing its path', () => {
  // A route added to the table and forgotten in the sidebar is a screen that exists and
  // cannot be opened. A count would not have said so; this does. It is written over every
  // section rather than over Settings alone because the section most likely to be forgotten
  // is the one that did not exist when the test was written.
  it.each(routeSections)(
    'lists every %s route in that sidebar',
    (section, origin) => {
      const listed = sidebarEntries[section].map((e) => e.location.name)
      const routes = Object.values(RouteName).filter(
        (name) => routeOrigin[name] === origin,
      )
      expect([...listed].sort()).toEqual([...routes].sort())
    },
  )
})

describe('the three sections are three preconditions', () => {
  it('gates exactly the screens that read the save', () => {
    // DESIGN-BRIEF.md section 4: a section is not a folder, it is what a screen needs before
    // it can answer. `router/routes.ts` derives `needsProfile` from the origin and from
    // nothing else - no per-route exception - so this table *is* the list of screens behind
    // the profile gate, and adding a screen forces whoever adds it to say which it is.
    const of = (origin: TabOrigin) =>
      Object.values(RouteName)
        .filter((name) => routeOrigin[name] === origin)
        .sort()
    expect(of(TabOrigin.Progress)).toEqual(
      [
        RouteName.Goals,
        RouteName.Completion,
        RouteName.Unlock,
        RouteName.Plan,
        RouteName.Collection,
        // Challenges reads section 7 of the .dat, so it is behind the profile gate like the
        // rest of Progress — and not beside Floor, which only reads what you painted.
        RouteName.Challenges,
        // Roll reads section 2 (the marks matrix) the same way: behind the gate, not beside
        // Live, which reads the log instead.
        RouteName.Roll,
      ].sort(),
    )
    // Live reads the log, Runs reads the archive, Floor reads what you painted. None of the
    // three opens the .dat, which is why Live and Runs stopped asking for a profile.
    expect(of(TabOrigin.Tool)).toEqual(
      [RouteName.Live, RouteName.Runs, RouteName.Floor].sort(),
    )
  })

  it('draws Progress, then Tool, then Wiki', () => {
    // NavBar.vue reads `Object.values(NavSection)`: the order of the declaration is the
    // order on screen, so it is pinned where someone would think to look for it.
    expect(Object.values(NavSection)).toEqual([
      NavSection.Progress,
      NavSection.Tool,
      NavSection.Wiki,
    ])
  })
})
