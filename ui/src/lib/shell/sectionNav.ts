import type { Component } from 'vue'
import { LayoutGridIcon } from '@lucide/vue'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import {
  RouteName,
  WikiCategory,
  routeIcon,
  routeTitle,
  wikiCategoryIcon,
  wikiCategoryTitle,
} from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import { NavSection } from './navSection'
import { TabOrigin } from '@/lib/shell/tabs'
import { tabOriginIcon } from '@/lib/shell/tabOriginIcon'

type Message = MessageKey<MessageSchema>

// The sidebar shows one section at a time (Schermate.dc.html): the three navbar sections,
// plus Settings, reached from the cog.
export const SidebarSection = {
  Progress: 'progress',
  Tool: 'tool',
  Wiki: 'wiki',
  Settings: 'settings',
} as const
export type SidebarSection =
  (typeof SidebarSection)[keyof typeof SidebarSection]

export interface SidebarEntry {
  key: string
  location: TabLocation
  label: Message
  icon: Component
}

export interface SidebarHeader {
  title: Message
  hint: Message
  icon: Component
}

const routeEntry = (name: RouteName): SidebarEntry => ({
  key: name,
  location: { name },
  label: routeTitle[name],
  icon: routeIcon[name],
})

// The landing: what the Wiki is and where it comes from, with the six categories and their
// counts. The bare route, so it's lit only when no category is chosen.
const overviewEntry: SidebarEntry = {
  key: RouteName.Wiki,
  location: { name: RouteName.Wiki },
  label: 'sidebar.wikiOverview',
  icon: LayoutGridIcon,
}

const wikiEntry = (category: WikiCategory): SidebarEntry => ({
  key: `${RouteName.Wiki}-${category}`,
  location: { name: RouteName.Wiki, query: { category } },
  label: wikiCategoryTitle[category],
  icon: wikiCategoryIcon[category],
})

export const sidebarEntries: Record<SidebarSection, SidebarEntry[]> = {
  [SidebarSection.Progress]: [
    RouteName.Completion,
    RouteName.Goals,
    RouteName.Unlock,
    RouteName.Collection,
    RouteName.Challenges,
    RouteName.Roll,
  ].map(routeEntry),
  // Live leads, because it is the only one of the three that answers about right now and
  // clicking the section lands on its first entry (B24).
  [SidebarSection.Tool]: [RouteName.Live, RouteName.Runs, RouteName.Floor].map(
    routeEntry,
  ),
  [SidebarSection.Wiki]: [
    overviewEntry,
    ...Object.values(WikiCategory).map(wikiEntry),
  ],
  [SidebarSection.Settings]: [
    RouteName.Profile,
    RouteName.Appearance,
    RouteName.Background,
    RouteName.TabsSettings,
    RouteName.Updates,
  ].map(routeEntry),
}

// Where clicking a section goes (`docs/BACKLOG.md` B24): its first entry, which is the
// section's own landing — Completion, Live, the Wiki's overview, the profile. A section
// always has entries, so the fallback is only there to keep the type honest.
export const firstEntry = (section: SidebarSection): SidebarEntry =>
  sidebarEntries[section][0] ?? routeEntry(RouteName.Completion)

export const sidebarHeaders: Record<SidebarSection, SidebarHeader> = {
  [SidebarSection.Progress]: {
    title: 'sidebar.progressTitle',
    hint: 'sidebar.progressHint',
    icon: tabOriginIcon[TabOrigin.Progress],
  },
  [SidebarSection.Tool]: {
    title: 'sidebar.toolTitle',
    hint: 'sidebar.toolHint',
    icon: tabOriginIcon[TabOrigin.Tool],
  },
  [SidebarSection.Wiki]: {
    title: 'sidebar.wikiTitle',
    hint: 'sidebar.wikiHint',
    icon: tabOriginIcon[TabOrigin.Wiki],
  },
  [SidebarSection.Settings]: {
    title: 'sidebar.settingsTitle',
    hint: 'sidebar.settingsHint',
    icon: tabOriginIcon[TabOrigin.Settings],
  },
}

// `null` for a tab that belongs to no section: search sits above the three
// (DESIGN-BRIEF.md §4.2), and opening one leaves the sidebar where it was.
export const sectionOfOrigin = (origin: TabOrigin): SidebarSection | null => {
  switch (origin) {
    case TabOrigin.Search:
      return null
    case TabOrigin.Wiki:
      return SidebarSection.Wiki
    case TabOrigin.Progress:
      return SidebarSection.Progress
    case TabOrigin.Tool:
      return SidebarSection.Tool
    case TabOrigin.Settings:
      return SidebarSection.Settings
    default:
      return assertNever(origin)
  }
}

// The navbar marks Progress, Tool or Wiki; while Settings is shown it marks none.
export const navSectionOf = (section: SidebarSection): NavSection | null => {
  switch (section) {
    case SidebarSection.Wiki:
      return NavSection.Wiki
    case SidebarSection.Progress:
      return NavSection.Progress
    case SidebarSection.Tool:
      return NavSection.Tool
    case SidebarSection.Settings:
      return null
    default:
      return assertNever(section)
  }
}

export const sidebarSectionOf = (section: NavSection): SidebarSection => {
  switch (section) {
    case NavSection.Wiki:
      return SidebarSection.Wiki
    case NavSection.Progress:
      return SidebarSection.Progress
    case NavSection.Tool:
      return SidebarSection.Tool
    default:
      return assertNever(section)
  }
}

export const isEntryActive = (
  entry: SidebarEntry,
  location: TabLocation | undefined,
): boolean =>
  location !== undefined &&
  entry.location.name === location.name &&
  (entry.location.query?.category ?? null) ===
    (location.query?.category ?? null)
