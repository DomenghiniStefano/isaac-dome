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

interface Debt extends Owed {
  paid: () => void
}

const owed = new Map<string, Debt>()

// How long a creator waits to hear that its newborn has taken the seed. It matters because the
// creator may be about to close — a window that gave away its last tab — and **the debt lives
// in the creator's memory**: if it dies first, the new window asks and nobody answers, and it
// opens with an empty bar. Seen on the machine, 2026-09-13.
const PayTimeout = 5000

export const oweSeed = (
  label: string,
  tabs: TabSeed[],
  activeIndex: number,
): Promise<void> =>
  new Promise((resolve) => {
    const done = () => {
      window.clearTimeout(timer)
      resolve()
    }
    const timer = window.setTimeout(() => {
      // Nobody came for it. Forget the debt rather than hold a window open forever: the
      // newborn falls back to its landing tab, which is a worse outcome but not a stuck one.
      owed.delete(label)
      resolve()
    }, PayTimeout)
    owed.set(label, { tabs, activeIndex, paid: done })
  })

// Reading it is also forgetting it: a seed is paid once, and a window that asks twice is a
// window that already has its tabs.
export const takeSeed = (label: string): Owed | null => {
  const debt = owed.get(label)
  if (!debt) return null
  owed.delete(label)
  debt.paid()
  return { tabs: debt.tabs, activeIndex: debt.activeIndex }
}
