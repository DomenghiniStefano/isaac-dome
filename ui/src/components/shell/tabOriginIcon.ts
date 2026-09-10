import type { Component } from 'vue'
import { BookIcon, CogIcon, InfoIcon, ListXIcon } from '@lucide/vue'
import { TabOrigin } from './tabs'

// A tab's origin by shape, not colour. A record over the whole set: an origin with no icon
// fails to compile.
export const tabOriginIcon: Record<TabOrigin, Component> = {
  [TabOrigin.Wiki]: BookIcon,
  [TabOrigin.Progress]: ListXIcon,
  [TabOrigin.Settings]: CogIcon,
  [TabOrigin.About]: InfoIcon,
}
