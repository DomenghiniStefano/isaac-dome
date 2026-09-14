// Pinia store ids.
export const StoreId = {
  Tabs: 'tabs',
  Profile: 'profile',
  Completion: 'completion',
  Graph: 'graph',
  Queue: 'queue',
  Collection: 'collection',
  Runs: 'runs',
  Live: 'live',
  Wiki: 'wiki',
  Settings: 'settings',
} as const
export type StoreId = (typeof StoreId)[keyof typeof StoreId]
