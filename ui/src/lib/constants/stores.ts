// Pinia store ids.
export const StoreId = {
  Tabs: 'tabs',
  Profile: 'profile',
  Completion: 'completion',
  Graph: 'graph',
  Queue: 'queue',
  Collection: 'collection',
} as const
export type StoreId = (typeof StoreId)[keyof typeof StoreId]
