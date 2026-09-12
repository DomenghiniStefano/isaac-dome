import type { Component } from 'vue'
import {
  ActivityIcon,
  AppWindowIcon,
  FlagIcon,
  GemIcon,
  Grid2x2Icon,
  LayersIcon,
  ListChecksIcon,
  LockOpenIcon,
  MapIcon,
  PackageIcon,
  PlayIcon,
  SaveIcon,
  SkullIcon,
  SlidersHorizontalIcon,
  TrophyIcon,
  UserIcon,
} from '@lucide/vue'
import { TabOrigin } from '@/components/shell/tabs'
import { tabOriginIcon } from '@/components/shell/tabOriginIcon'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'

// Everything a location needs to be named, drawn and placed, with no screen component in
// sight: tab labels and the sidebar are built and tested from this alone.
export const RouteName = {
  Search: 'search',
  NextSteps: 'nextSteps',
  Completion: 'completion',
  Unlock: 'unlock',
  Plan: 'plan',
  Collection: 'collection',
  Runs: 'runs',
  Live: 'live',
  Wiki: 'wiki',
  Profile: 'profile',
  Appearance: 'appearance',
  TabsSettings: 'tabsSettings',
} as const
export type RouteName = (typeof RouteName)[keyof typeof RouteName]

export const WikiCategory = {
  Items: 'items',
  Trinkets: 'trinkets',
  Achievements: 'achievements',
  Bosses: 'bosses',
  Challenges: 'challenges',
  Characters: 'characters',
} as const
export type WikiCategory = (typeof WikiCategory)[keyof typeof WikiCategory]

// A tab's identity: a route and its query, never the view's content (B6).
export interface TabLocation {
  name: RouteName
  query?: { category?: WikiCategory; page?: string; q?: string }
}

type Message = MessageKey<MessageSchema>

export const defaultLocation: TabLocation = { name: RouteName.NextSteps }

export const routePath: Record<RouteName, string> = {
  [RouteName.Search]: '/search',
  [RouteName.NextSteps]: '/progress/next-steps',
  [RouteName.Completion]: '/progress/completion',
  [RouteName.Unlock]: '/progress/unlock',
  [RouteName.Plan]: '/progress/plan',
  [RouteName.Collection]: '/progress/collection',
  [RouteName.Runs]: '/progress/runs',
  [RouteName.Live]: '/progress/live',
  [RouteName.Wiki]: '/wiki',
  [RouteName.Profile]: '/settings/profile',
  [RouteName.Appearance]: '/settings/appearance',
  [RouteName.TabsSettings]: '/settings/tabs',
}

export const routeTitle: Record<RouteName, Message> = {
  [RouteName.Search]: 'routes.search',
  [RouteName.NextSteps]: 'routes.nextSteps',
  [RouteName.Completion]: 'routes.completion',
  [RouteName.Unlock]: 'routes.unlock',
  [RouteName.Plan]: 'routes.plan',
  [RouteName.Collection]: 'routes.collection',
  [RouteName.Runs]: 'routes.runs',
  [RouteName.Live]: 'routes.live',
  [RouteName.Wiki]: 'routes.wiki',
  [RouteName.Profile]: 'routes.profile',
  [RouteName.Appearance]: 'routes.appearance',
  [RouteName.TabsSettings]: 'routes.tabsSettings',
}

export const routeOrigin: Record<RouteName, TabOrigin> = {
  [RouteName.Search]: TabOrigin.Search,
  [RouteName.NextSteps]: TabOrigin.Progress,
  [RouteName.Completion]: TabOrigin.Progress,
  [RouteName.Unlock]: TabOrigin.Progress,
  [RouteName.Plan]: TabOrigin.Progress,
  [RouteName.Collection]: TabOrigin.Progress,
  [RouteName.Runs]: TabOrigin.Progress,
  [RouteName.Live]: TabOrigin.Progress,
  [RouteName.Wiki]: TabOrigin.Wiki,
  [RouteName.Profile]: TabOrigin.Settings,
  [RouteName.Appearance]: TabOrigin.Settings,
  [RouteName.TabsSettings]: TabOrigin.Settings,
}

export const routeIcon: Record<RouteName, Component> = {
  [RouteName.Search]: tabOriginIcon[TabOrigin.Search],
  [RouteName.NextSteps]: ListChecksIcon,
  [RouteName.Completion]: Grid2x2Icon,
  [RouteName.Unlock]: LockOpenIcon,
  [RouteName.Plan]: MapIcon,
  [RouteName.Collection]: LayersIcon,
  [RouteName.Runs]: PlayIcon,
  [RouteName.Live]: ActivityIcon,
  [RouteName.Wiki]: tabOriginIcon[TabOrigin.Wiki],
  [RouteName.Profile]: SaveIcon,
  [RouteName.Appearance]: SlidersHorizontalIcon,
  [RouteName.TabsSettings]: AppWindowIcon,
}

// Which sub-project brings a screen that is still a placeholder; absent once it's real.
export const routeArrives: Partial<Record<RouteName, Message>> = {
  [RouteName.Runs]: 'placeholder.runArchive',
  [RouteName.Live]: 'placeholder.runArchive',
  [RouteName.TabsSettings]: 'placeholder.tabs',
}

export const wikiCategoryTitle: Record<WikiCategory, Message> = {
  [WikiCategory.Items]: 'wikiCategories.items',
  [WikiCategory.Trinkets]: 'wikiCategories.trinkets',
  [WikiCategory.Achievements]: 'wikiCategories.achievements',
  [WikiCategory.Bosses]: 'wikiCategories.bosses',
  [WikiCategory.Challenges]: 'wikiCategories.challenges',
  [WikiCategory.Characters]: 'wikiCategories.characters',
}

export const wikiCategoryIcon: Record<WikiCategory, Component> = {
  [WikiCategory.Items]: PackageIcon,
  [WikiCategory.Trinkets]: GemIcon,
  [WikiCategory.Achievements]: TrophyIcon,
  [WikiCategory.Bosses]: SkullIcon,
  [WikiCategory.Challenges]: FlagIcon,
  [WikiCategory.Characters]: UserIcon,
}

// A tab's label: a wiki category names itself, every other location is its route.
export const locationTitle = (location: TabLocation): Message => {
  const category = location.query?.category
  return location.name === RouteName.Wiki && category
    ? wikiCategoryTitle[category]
    : routeTitle[location.name]
}
