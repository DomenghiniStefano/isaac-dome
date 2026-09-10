// Pinia store ids.
export const StoreId = { Tabs: 'tabs', Profile: 'profile' } as const
export type StoreId = (typeof StoreId)[keyof typeof StoreId]
