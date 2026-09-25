import type { Message } from '@/i18n/message'
import type { Component } from 'vue'
import type { RouteComponent, RouteRecordRaw } from 'vue-router'
import { TabOrigin } from '@/lib/shell/tabs'
import CompletionScreen from '@/screens/CompletionScreen.vue'
import {
  RouteName,
  routeIcon,
  routeOrigin,
  routePath,
  routeTitle,
} from './routeTable'

declare module 'vue-router' {
  interface RouteMeta {
    origin: TabOrigin
    title: Message
    icon: Component
    needsProfile: boolean
  }
}

// One screen per route, and the `Record` holds every one: a route added to `RouteName` without
// its screen fails the build here instead of rendering nothing.
//
// **Each loads when it is first opened**, except the one the app opens on: sixteen screens
// imported up front made one 725 kB chunk that every launch parsed whole, for a first paint
// that draws one of them. Completamento stays in the entry chunk so the first screen does not
// wait on a second file.
const screens: Record<
  RouteName,
  RouteComponent | (() => Promise<RouteComponent>)
> = {
  [RouteName.Completion]: CompletionScreen,
  [RouteName.Goals]: () => import('@/screens/GoalsScreen.vue'),
  [RouteName.Unlock]: () => import('@/screens/UnlockScreen.vue'),
  [RouteName.Collection]: () => import('@/screens/CollectionScreen.vue'),
  [RouteName.Challenges]: () => import('@/screens/ChallengesScreen.vue'),
  [RouteName.Roll]: () => import('@/screens/RollScreen.vue'),
  [RouteName.Live]: () => import('@/screens/LiveScreen.vue'),
  [RouteName.Runs]: () => import('@/screens/RunsScreen.vue'),
  [RouteName.Floor]: () => import('@/screens/FloorScreen.vue'),
  [RouteName.Wiki]: () => import('@/screens/WikiScreen.vue'),
  [RouteName.Search]: () => import('@/screens/SearchScreen.vue'),
  [RouteName.Profile]: () => import('@/screens/ProfileScreen.vue'),
  [RouteName.Appearance]: () => import('@/screens/AppearanceScreen.vue'),
  [RouteName.Background]: () => import('@/screens/BackgroundScreen.vue'),
  [RouteName.TabsSettings]: () => import('@/screens/TabsSettingsScreen.vue'),
  [RouteName.Updates]: () => import('@/screens/UpdatesScreen.vue'),
}

export const routes: RouteRecordRaw[] = [
  { path: '/', redirect: { name: RouteName.Completion } },
  // The URL a link or a bookmark may still carry, from when the Plan was a screen of its own.
  // **No `name`**, so it is not a location and cannot become a tab: a stored tab is carried by
  // `sessionDocument`'s retired-name map, which is a different mechanism because it answers a
  // different question — one is an address somebody typed, the other is a window somebody left
  // open. The path is written out because it no longer has an entry in `routePath` to read.
  { path: '/progress/plan', redirect: { name: RouteName.Goals } },
  ...Object.values(RouteName).map((name): RouteRecordRaw => ({
    path: routePath[name],
    name,
    component: screens[name],
    meta: {
      origin: routeOrigin[name],
      title: routeTitle[name],
      icon: routeIcon[name],
      needsProfile: routeOrigin[name] === TabOrigin.Progress,
    },
  })),
]
