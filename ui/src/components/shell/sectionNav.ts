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
import { TabOrigin } from './tabs'
import { tabOriginIcon } from './tabOriginIcon'

type Message = MessageKey<MessageSchema>

// The sidebar shows one section at a time (Schermate.dc.html): the two navbar sections,
// plus Settings, reached from the cog.
export const SidebarSection = {
  Wiki: 'wiki',
  Progress: 'progress',
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
    RouteName.NextSteps,
    RouteName.Completion,
    RouteName.Unlock,
    RouteName.Plan,
    RouteName.Collection,
    RouteName.Runs,
    RouteName.Live,
  ].map(routeEntry),
  [SidebarSection.Wiki]: [
    overviewEntry,
    ...Object.values(WikiCategory).map(wikiEntry),
  ],
  [SidebarSection.Settings]: [
    RouteName.Profile,
    RouteName.Appearance,
    RouteName.TabsSettings,
  ].map(routeEntry),
}

// Where clicking a section goes (`docs/BACKLOG.md` B24): its first entry, which is the
// section's own landing — Next steps, the Wiki's overview, the profile. A section always
// has entries, so the fallback is only there to keep the type honest.
export const firstEntry = (section: SidebarSection): SidebarEntry =>
  sidebarEntries[section][0] ?? routeEntry(RouteName.NextSteps)

export const sidebarHeaders: Record<SidebarSection, SidebarHeader> = {
  [SidebarSection.Progress]: {
    title: 'sidebar.progressTitle',
    hint: 'sidebar.progressHint',
    icon: tabOriginIcon[TabOrigin.Progress],
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

export const sectionOfOrigin = (origin: TabOrigin): SidebarSection => {
  switch (origin) {
    case TabOrigin.Wiki:
      return SidebarSection.Wiki
    case TabOrigin.Progress:
      return SidebarSection.Progress
    case TabOrigin.Settings:
      return SidebarSection.Settings
    default:
      return assertNever(origin)
  }
}

// The navbar marks Wiki or Progress; while Settings is shown it marks neither.
export const navSectionOf = (section: SidebarSection): NavSection | null => {
  switch (section) {
    case SidebarSection.Wiki:
      return NavSection.Wiki
    case SidebarSection.Progress:
      return NavSection.Progress
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
