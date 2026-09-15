import type { Component } from 'vue'
import {
  ActivityIcon,
  AppWindowIcon,
  FlagIcon,
  GemIcon,
  Grid2x2Icon,
  Grid3x3Icon,
  LayersIcon,
  ListChecksIcon,
  LockOpenIcon,
  MapIcon,
  MonitorDotIcon,
  PackageIcon,
  PlayIcon,
  SaveIcon,
  SkullIcon,
  SlidersHorizontalIcon,
  SparklesIcon,
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
  Goals: 'goals',
  Completion: 'completion',
  Unlock: 'unlock',
  Plan: 'plan',
  Collection: 'collection',
  Runs: 'runs',
  Live: 'live',
  Floor: 'floor',
  Wiki: 'wiki',
  Profile: 'profile',
  Appearance: 'appearance',
  Background: 'background',
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
  // B46: the sixteen transformation pages entered the dataset on 2026-09-13 and had no
  // category, so `pageLocation` could not build one and no route opened them.
  Transformations: 'transformations',
} as const
export type WikiCategory = (typeof WikiCategory)[keyof typeof WikiCategory]

// A tab's identity: a route and its query, never the view's content (B6).
// `state` is the same kind of thing as `q`: a list opened with a filter already applied, so
// that a link can land on Unlock showing only what is unlockable now.
export interface TabLocation {
  name: RouteName
  query?: {
    category?: WikiCategory
    page?: string
    q?: string
    state?: string
    /** B37: what you said you want, keyed exactly the way `page` is. */
    want?: string
  }
}

type Message = MessageKey<MessageSchema>

export const defaultLocation: TabLocation = { name: RouteName.Goals }

export const routePath: Record<RouteName, string> = {
  [RouteName.Search]: '/search',
  [RouteName.Goals]: '/progress/goals',
  [RouteName.Completion]: '/progress/completion',
  [RouteName.Unlock]: '/progress/unlock',
  [RouteName.Plan]: '/progress/plan',
  [RouteName.Collection]: '/progress/collection',
  [RouteName.Runs]: '/tool/runs',
  [RouteName.Live]: '/tool/live',
  [RouteName.Floor]: '/tool/floor',
  [RouteName.Wiki]: '/wiki',
  [RouteName.Profile]: '/settings/profile',
  [RouteName.Appearance]: '/settings/appearance',
  [RouteName.Background]: '/settings/background',
  [RouteName.TabsSettings]: '/settings/tabs',
}

export const routeTitle: Record<RouteName, Message> = {
  [RouteName.Search]: 'routes.search',
  [RouteName.Goals]: 'routes.goals',
  [RouteName.Completion]: 'routes.completion',
  [RouteName.Unlock]: 'routes.unlock',
  [RouteName.Plan]: 'routes.plan',
  [RouteName.Collection]: 'routes.collection',
  [RouteName.Runs]: 'routes.runs',
  [RouteName.Live]: 'routes.live',
  [RouteName.Floor]: 'routes.floor',
  [RouteName.Wiki]: 'routes.wiki',
  [RouteName.Profile]: 'routes.profile',
  [RouteName.Appearance]: 'routes.appearance',
  [RouteName.Background]: 'routes.background',
  [RouteName.TabsSettings]: 'routes.tabsSettings',
}

export const routeOrigin: Record<RouteName, TabOrigin> = {
  [RouteName.Search]: TabOrigin.Search,
  [RouteName.Goals]: TabOrigin.Progress,
  [RouteName.Completion]: TabOrigin.Progress,
  [RouteName.Unlock]: TabOrigin.Progress,
  [RouteName.Plan]: TabOrigin.Progress,
  [RouteName.Collection]: TabOrigin.Progress,
  // The three that answer without the save: the log, the archive, and your own drawing.
  // This is what puts them outside the profile gate — `routes.ts` derives it from here.
  [RouteName.Runs]: TabOrigin.Tool,
  [RouteName.Live]: TabOrigin.Tool,
  [RouteName.Floor]: TabOrigin.Tool,
  [RouteName.Wiki]: TabOrigin.Wiki,
  [RouteName.Profile]: TabOrigin.Settings,
  [RouteName.Appearance]: TabOrigin.Settings,
  [RouteName.Background]: TabOrigin.Settings,
  [RouteName.TabsSettings]: TabOrigin.Settings,
}

export const routeIcon: Record<RouteName, Component> = {
  [RouteName.Search]: tabOriginIcon[TabOrigin.Search],
  [RouteName.Goals]: ListChecksIcon,
  [RouteName.Completion]: Grid2x2Icon,
  [RouteName.Unlock]: LockOpenIcon,
  [RouteName.Plan]: MapIcon,
  [RouteName.Collection]: LayersIcon,
  [RouteName.Runs]: PlayIcon,
  [RouteName.Live]: ActivityIcon,
  [RouteName.Floor]: Grid3x3Icon,
  [RouteName.Wiki]: tabOriginIcon[TabOrigin.Wiki],
  [RouteName.Profile]: SaveIcon,
  [RouteName.Appearance]: SlidersHorizontalIcon,
  [RouteName.Background]: MonitorDotIcon,
  [RouteName.TabsSettings]: AppWindowIcon,
}

// Which sub-project brings a screen that is still a placeholder; absent once it's real.
export const routeArrives: Partial<Record<RouteName, Message>> = {
  [RouteName.TabsSettings]: 'placeholder.tabs',
}

export const wikiCategoryTitle: Record<WikiCategory, Message> = {
  [WikiCategory.Items]: 'wikiCategories.items',
  [WikiCategory.Trinkets]: 'wikiCategories.trinkets',
  [WikiCategory.Achievements]: 'wikiCategories.achievements',
  [WikiCategory.Bosses]: 'wikiCategories.bosses',
  [WikiCategory.Challenges]: 'wikiCategories.challenges',
  [WikiCategory.Characters]: 'wikiCategories.characters',
  [WikiCategory.Transformations]: 'wikiCategories.transformations',
}

export const wikiCategoryIcon: Record<WikiCategory, Component> = {
  [WikiCategory.Items]: PackageIcon,
  [WikiCategory.Trinkets]: GemIcon,
  [WikiCategory.Achievements]: TrophyIcon,
  [WikiCategory.Bosses]: SkullIcon,
  [WikiCategory.Challenges]: FlagIcon,
  [WikiCategory.Characters]: UserIcon,
  [WikiCategory.Transformations]: SparklesIcon,
}

// A tab's label: a wiki category names itself, every other location is its route.
export const locationTitle = (location: TabLocation): Message => {
  const category = location.query?.category
  return location.name === RouteName.Wiki && category
    ? wikiCategoryTitle[category]
    : routeTitle[location.name]
}
