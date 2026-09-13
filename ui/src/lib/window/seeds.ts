import type { TabSeed } from './messages'

// What a window that has just created another owes it. The debt is registered **before** the
// window is created, because the newborn broadcasts Ready the moment it mounts — which can be
// before or after `create` resolves, and neither order may lose the answer.
//
// It lives in its own module so that the store can owe a seed and the session can pay it
// without the two importing each other.
export interface Owed {
  tabs: TabSeed[]
  activeIndex: number
}

const owed = new Map<string, Owed>()

export const oweSeed = (
  label: string,
  tabs: TabSeed[],
  activeIndex: number,
): void => {
  owed.set(label, { tabs, activeIndex })
}

// Reading it is also forgetting it: a seed is paid once, and a window that asks twice is a
// window that already has its tabs.
export const takeSeed = (label: string): Owed | null => {
  const seed = owed.get(label)
  if (!seed) return null
  owed.delete(label)
  return seed
}
