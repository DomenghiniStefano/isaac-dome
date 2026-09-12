import { assertNever } from '@/lib/assertNever'
import type {
  NextSteps,
  RequirementView,
  UnlockNode,
  UnlockTarget,
  UnlockView,
} from '../types'
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

// The pack's payload was written before a character carried its form (`docs/BACKLOG.md`
// B28), so the field is absent there: absent reads as the base form, which is right for
// every base character and wrong for the Tainted ones. Declared once in the console, and it
// goes with the next `pnpm design:export` on a machine with the game.
let warnedAboutForms = false
const withForm = <T extends { kind: string; tainted?: boolean }>(
  value: T,
): T => {
  if (value.kind !== 'character' || typeof value.tainted === 'boolean')
    return value
  if (!warnedAboutForms) {
    warnedAboutForms = true
    console.warn(
      "graph fixture: the design pack's unlock.json predates the character's tainted flag; every character reads as its base form",
    )
  }
  return { ...value, tainted: false }
}

// Same story for the page a requirement links to (spec 3.5d): a pack exported before the
// field has none, and an absent key would read in a template exactly like "the dataset has no
// page". It is filled with `null` — which is that sentence, said on purpose — and declared
// once, until the next `pnpm design:export` on a machine with the game.
let warnedAboutPages = false
const withPage = (requirement: RequirementView): RequirementView => {
  switch (requirement.kind) {
    case 'gate':
    case 'mark':
    case 'counter':
    case 'unknown':
      return requirement
    case 'character':
    case 'boss':
    case 'challenge':
    case 'item':
      if (requirement.page !== undefined) return requirement
      if (!warnedAboutPages) {
        warnedAboutPages = true
        console.warn(
          "graph fixture: the design pack's unlock.json predates the requirement's page; nothing links until the next pnpm design:export on a machine with the game",
        )
      }
      return { ...requirement, page: null }
    default:
      return assertNever(requirement)
  }
}

const nodeWithIcons = (node: UnlockNode, icon: IconOf): UnlockNode => ({
  ...node,
  achievement:
    node.achievement.kind === 'known'
      ? { ...node.achievement, iconUrl: icon(node.achievement.iconUrl) }
      : node.achievement,
  unlocks: node.unlocks.map((t) => withForm(targetWithIcon(t, icon))),
  missing: node.missing.map((r) => withPage(withForm(r))),
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
