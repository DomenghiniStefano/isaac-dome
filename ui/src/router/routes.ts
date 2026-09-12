import type { Component } from 'vue'
import type { RouteRecordRaw } from 'vue-router'
import { TabOrigin } from '@/components/shell/tabs'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import AppearanceScreen from '@/screens/AppearanceScreen.vue'
import CollectionScreen from '@/screens/CollectionScreen.vue'
import CompletionScreen from '@/screens/CompletionScreen.vue'
import NextStepsScreen from '@/screens/NextStepsScreen.vue'
import PlaceholderScreen from '@/screens/PlaceholderScreen.vue'
import PlanScreen from '@/screens/PlanScreen.vue'
import ProfileScreen from '@/screens/ProfileScreen.vue'
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
  [RouteName.NextSteps]: NextStepsScreen,
  [RouteName.Completion]: CompletionScreen,
  [RouteName.Unlock]: UnlockScreen,
  [RouteName.Plan]: PlanScreen,
  [RouteName.Collection]: CollectionScreen,
  [RouteName.Wiki]: WikiScreen,
  [RouteName.Profile]: ProfileScreen,
  [RouteName.Appearance]: AppearanceScreen,
}

export const routes: RouteRecordRaw[] = [
  { path: '/', redirect: { name: RouteName.NextSteps } },
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
