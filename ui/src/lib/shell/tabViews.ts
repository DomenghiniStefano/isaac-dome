import type { Translate } from '@/i18n/message'
import { routeOrigin } from '@/router/routeTable'
import { tabLabel, tabLocation } from '@/stores/tabModel'
import type { Tab } from '@/stores/tabModel'
import type { TabView } from './tabs'

// What the strip draws for one tab: its label and the part of the app it comes from. A page
// tab reads as its page's title once the wiki index knows it; every other label is a message.
export const tabViewOf = (
  tab: Tab,
  titleOf: (key: string) => string | null,
  t: Translate,
): TabView => {
  const location = tabLocation(tab)
  const label = tabLabel(location, titleOf)
  return {
    id: tab.id,
    label: typeof label === 'string' ? t(label) : label.text,
    origin: routeOrigin[location.name],
  }
}
