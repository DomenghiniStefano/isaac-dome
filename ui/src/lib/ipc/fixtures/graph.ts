import type { NextSteps, UnlockNode, UnlockTarget, UnlockView } from '../types'
import { packIconUrl } from './graphArt'

// Development only: the design pack's committed payloads, the reference profile on 2026-09-08
// (contracts/payload/unlock.json and next_steps.json), read through a glob so that no file
// outside src/ joins the TypeScript project.
const payloads = import.meta.glob<unknown>(
  '../../../../../design-export/isaacdome-design-pack/contracts/payload/{unlock,next_steps}.json',
  { eager: true, import: 'default' },
)

// A missing payload is refused loudly, like a command with no fixture: a screen must not
// render something plausible built on nothing.
const payload = <T>(name: string): T => {
  const found = Object.entries(payloads).find(([path]) =>
    path.endsWith(`/${name}.json`),
  )
  if (!found) throw new Error(`the design pack has no ${name}.json`)
  return found[1] as T
}

type IconOf = (url: string | null) => string | null

const targetWithIcon = (target: UnlockTarget, icon: IconOf): UnlockTarget =>
  target.kind === 'item' ? { ...target, iconUrl: icon(target.iconUrl) } : target

const nodeWithIcons = (node: UnlockNode, icon: IconOf): UnlockNode => ({
  ...node,
  achievement:
    node.achievement.kind === 'known'
      ? { ...node.achievement, iconUrl: icon(node.achievement.iconUrl) }
      : node.achievement,
  unlocks: node.unlocks.map((t) => targetWithIcon(t, icon)),
})

const slotOf = (node: UnlockNode): number =>
  node.achievement.kind === 'known'
    ? node.achievement.id
    : node.achievement.slot

// What crates/ipc/src/graph.rs `unlock_view` builds without a catalog: every slot an unknown
// achievement with the save's done, nothing unlocked, no origin, nothing missing, and a
// partial graph with one unknown — never computed, which would read as nothing in the way.
const withoutCatalog = (view: UnlockView): UnlockView => ({
  nodes: view.nodes.map((node) => ({
    achievement: { kind: 'unknown', slot: slotOf(node) },
    done: node.done,
    unlocks: [],
    origin: null,
    missing: [],
    graph: { kind: 'partial', blockedBy: 0, fanOut: 0, unknown: 1 },
  })),
  totals: {
    slots: view.totals.slots,
    done: view.totals.done,
    known: 0,
    unknown: view.nodes.length,
  },
  diagnostics: [{ kind: 'noCatalog' }],
})

export interface GraphAnswerOptions {
  withArt: boolean
  withCatalog: boolean
}

export interface GraphAnswers {
  unlock: UnlockView
  steps: NextSteps
}

export const graphAnswers = ({
  withArt,
  withCatalog,
}: GraphAnswerOptions): GraphAnswers => {
  const unlock = payload<UnlockView>('unlock')
  const steps = payload<NextSteps>('next_steps')
  if (!withCatalog)
    return {
      unlock: withoutCatalog(unlock),
      steps: { steps: [], basis: steps.basis },
    }
  const icon: IconOf = (url) => (withArt ? packIconUrl(url) : null)
  return {
    unlock: {
      ...unlock,
      nodes: unlock.nodes.map((n) => nodeWithIcons(n, icon)),
    },
    steps: { ...steps, steps: steps.steps.map((n) => nodeWithIcons(n, icon)) },
  }
}
