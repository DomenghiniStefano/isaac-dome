import type { Component } from 'vue'
import type { RouteRecordRaw } from 'vue-router'
import { TabOrigin } from '@/components/shell/tabs'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import AppearanceScreen from '@/screens/AppearanceScreen.vue'
import BackgroundScreen from '@/screens/BackgroundScreen.vue'
import ChallengesScreen from '@/screens/ChallengesScreen.vue'
import CollectionScreen from '@/screens/CollectionScreen.vue'
import CompletionScreen from '@/screens/CompletionScreen.vue'
import FloorScreen from '@/screens/FloorScreen.vue'
import GoalsScreen from '@/screens/GoalsScreen.vue'
import PlaceholderScreen from '@/screens/PlaceholderScreen.vue'
import LiveScreen from '@/screens/LiveScreen.vue'
import RunsScreen from '@/screens/RunsScreen.vue'
import PlanScreen from '@/screens/PlanScreen.vue'
import ProfileScreen from '@/screens/ProfileScreen.vue'
import SearchScreen from '@/screens/SearchScreen.vue'
import TabsSettingsScreen from '@/screens/TabsSettingsScreen.vue'
import UnlockScreen from '@/screens/UnlockScreen.vue'
import WikiScreen from '@/screens/WikiScreen.vue'
import {
  RouteName,
  routeArrives,
  routeIcon,
  routeOrigin,
  routePath,
  routeTitle,
} from './routeTable'

declare module 'vue-router' {
  interface RouteMeta {
    origin: TabOrigin
    title: MessageKey<MessageSchema>
    icon: Component
    needsProfile: boolean
    arrives?: MessageKey<MessageSchema>
  }
}

// The screens that exist. Every other route renders its placeholder until its sub-project.
const screens: Partial<Record<RouteName, Component>> = {
  [RouteName.Goals]: GoalsScreen,
  [RouteName.Completion]: CompletionScreen,
  [RouteName.Unlock]: UnlockScreen,
  [RouteName.Plan]: PlanScreen,
  [RouteName.Collection]: CollectionScreen,
  [RouteName.Challenges]: ChallengesScreen,
  [RouteName.Live]: LiveScreen,
  [RouteName.Runs]: RunsScreen,
  [RouteName.Floor]: FloorScreen,
  [RouteName.Wiki]: WikiScreen,
  [RouteName.Search]: SearchScreen,
  [RouteName.Profile]: ProfileScreen,
  [RouteName.Appearance]: AppearanceScreen,
  [RouteName.Background]: BackgroundScreen,
  [RouteName.TabsSettings]: TabsSettingsScreen,
}

export const routes: RouteRecordRaw[] = [
  { path: '/', redirect: { name: RouteName.Goals } },
  ...Object.values(RouteName).map((name): RouteRecordRaw => ({
    path: routePath[name],
    name,
    component: screens[name] ?? PlaceholderScreen,
    meta: {
      origin: routeOrigin[name],
      title: routeTitle[name],
      icon: routeIcon[name],
      needsProfile: routeOrigin[name] === TabOrigin.Progress,
      arrives: routeArrives[name],
    },
  })),
]
