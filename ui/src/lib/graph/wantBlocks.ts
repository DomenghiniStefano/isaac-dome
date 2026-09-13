import { assertNever } from '@/lib/assertNever'
import type {
  UnlockNode,
  WantDiagnostic,
  WantRoute,
  WantView,
} from '@/lib/ipc/types'

// One block per way in. The kind restates the route's state as what the block draws, so the
// screen switches on this and never on `steps.length` — an empty list means four different
// things and none of them is "nothing missing".
export const WantBlockKind = {
  Chain: 'chain',
  Done: 'done',
  AvailableNow: 'availableNow',
  NoProfile: 'noProfile',
} as const
export type WantBlockKind = (typeof WantBlockKind)[keyof typeof WantBlockKind]

export interface WantBlock {
  kind: WantBlockKind
  achievement: number | null
  node: UnlockNode
  steps: UnlockNode[]
  unknown: number
  queueable: boolean
}

// A node the catalog doesn't know has no id to queue. `null` and not `-1`: a sentinel number
// would be a valid argument to the queue command.
const idOf = (node: UnlockNode): number | null =>
  node.achievement.kind === 'known' ? node.achievement.id : null

const blockOf = (route: WantRoute, queued: Set<number>): WantBlock => {
  const achievement = idOf(route.node)
  const offers = achievement !== null && !queued.has(achievement)
  const base = {
    achievement,
    node: route.node,
    steps: [] as UnlockNode[],
    unknown: 0,
    queueable: false,
  }
  switch (route.state.kind) {
    case 'chain':
      return {
        ...base,
        kind: WantBlockKind.Chain,
        steps: route.state.steps,
        unknown: route.state.unknown,
        queueable: offers,
      }
    case 'availableNow':
      return { ...base, kind: WantBlockKind.AvailableNow, queueable: offers }
    case 'done':
      return { ...base, kind: WantBlockKind.Done }
    case 'noProfile':
      return { ...base, kind: WantBlockKind.NoProfile }
    default:
      return assertNever(route.state)
  }
}

export const wantBlocks = (view: WantView, queued: Set<number>): WantBlock[] =>
  view.routes.map((route) => blockOf(route, queued))

// One line above the blocks, never assembled from the rows: Rust already decided, and the
// two must not be able to disagree.
export const wantBanner = (view: WantView): WantDiagnostic | null =>
  view.diagnostics[0] ?? null
