import type { TabViewSpec } from '@/lib/tabs/tabView'

// How Goals was being worked on, kept on the tab: today, what was being typed into the want bar.
//
// It was a `ref` in `WantBar` (#79). While the router reused one component for every Goals tab, a
// second tab opened with the first one's words already typed; once each entry got its own
// instance, a tab switch cleared them instead. The queue and the want itself are not here: the
// queue is the app's (the store's database), and a want that was picked is the location's `want`.
export interface GoalsReading {
  typed: string
}

export const goalsView: TabViewSpec<GoalsReading> = {
  empty: () => ({ typed: '' }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const { typed } = value as { typed?: unknown }
    return { typed: typeof typed === 'string' ? typed : '' }
  },
}
