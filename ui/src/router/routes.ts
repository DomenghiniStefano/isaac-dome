import type { Component } from 'vue'
import type { RouteRecordRaw } from 'vue-router'
import { TabOrigin } from '@/components/shell/tabs'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import PlaceholderScreen from '@/screens/PlaceholderScreen.vue'
import ProfileScreen from '@/screens/ProfileScreen.vue'
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
  [RouteName.Profile]: ProfileScreen,
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
