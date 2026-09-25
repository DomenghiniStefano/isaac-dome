import { assertNever } from '@/lib/assertNever'
import { knownId } from '@/lib/graph/achievementNode'
import { WantDiagnostic } from '../types'
import type {
  NextSteps,
  Target,
  UnlockNode,
  UnlockTarget,
  UnlockView,
  WantState,
  WantView,
  WantedView,
} from '../types'

// Development only: the committed payloads under ui/fixtures/, the reference profile as it
// stood on 2026-09-08, read through a glob so that no file outside src/ joins the TypeScript
// project. They are a frozen snapshot: the tool that wrote them is gone, so where they
// predate a field the fixture fills it in below rather than waiting for a newer export.
const payloads = import.meta.glob<unknown>(
  '../../../../fixtures/payload/{unlock,next_steps}.json',
  { eager: true, import: 'default' },
)

// A missing payload is refused loudly, like a command with no fixture: a screen must not
// render something plausible built on nothing.
const payload = <T>(name: string): T => {
  const found = Object.entries(payloads).find(([path]) =>
    path.endsWith(`/${name}.json`),
  )
  if (!found) throw new Error(`the fixtures carry no ${name}.json`)
  return found[1] as T
}

// Every icon is null here. The app cuts its sprites from the user's own copy of the game at
// runtime; the development server has no copy, so the payload's `isaac://` links resolve to
// nothing — the same answer a machine without the game gets.
const targetWithoutIcon = (target: UnlockTarget): UnlockTarget =>
  target.kind === 'item' ? { ...target, iconUrl: null } : target

// The payload predates the resolved condition: it carries the game file's `hint`, and
// the wiki's requirement is filled in by Rust, which the fixtures do not run. So the old key
// becomes the new one where it has something to say — and where the file was silent the line
// is `null`, which is a real state of the card, only far more common here than in the app:
// measured 2026-09-13, the file answers for 283 of 637 achievements and the wiki for the rest.
let warnedAboutConditions = false
const withCondition = (
  a: UnlockNode['achievement'],
): UnlockNode['achievement'] => {
  if (a.kind !== 'known' || a.condition !== undefined) return a
  if (!warnedAboutConditions) {
    warnedAboutConditions = true
    console.warn(
      'graph fixture: the payloads predate the resolved condition; only the achievements the game file itself describes show one, where the app shows all of them',
    )
  }
  const { hint } = a as unknown as { hint: string | null | undefined }
  return { ...a, condition: hint ?? null }
}

const nodeResolved = (node: UnlockNode): UnlockNode => ({
  ...node,
  achievement:
    node.achievement.kind === 'known'
      ? withCondition({ ...node.achievement, iconUrl: null })
      : node.achievement,
  unlocks: node.unlocks.map(targetWithoutIcon),
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
  withCatalog: boolean
}

export interface GraphAnswers {
  unlock: UnlockView
  steps: NextSteps
}

export const graphAnswers = ({
  withCatalog,
}: GraphAnswerOptions): GraphAnswers => {
  const unlock = payload<UnlockView>('unlock')
  const { sections } = payload<NextSteps>('next_steps')
  // No catalog, nothing to recommend: no sections at all, which is what Rust answers too.
  if (!withCatalog)
    return { unlock: withoutCatalog(unlock), steps: { sections: [] } }
  return {
    unlock: {
      ...unlock,
      nodes: unlock.nodes.map(nodeResolved),
    },
    steps: {
      sections: sections.map((s) => ({
        ...s,
        steps: s.steps.map(nodeResolved),
      })),
    },
  }
}

// B37, development only: the same question the `want` command answers, over the payload the
// other two fixtures read. Deliberately simple — it resolves the name and then takes the
// first two not-done nodes as the chain — but it never invents a state: done is done, and
// without a catalog nothing resolves, exactly as Rust answers.
const namesTarget = (node: UnlockNode, target: Target): boolean => {
  if (target.kind === 'achievement') return knownId(node) === target.id
  return node.unlocks.some((u) => {
    switch (u.kind) {
      case 'item':
        return (
          u.id ===
          (target.kind === 'item' || target.kind === 'trinket' ? target.id : -1)
        )
      case 'character':
        return target.kind === 'character' && u.id === target.id
      case 'challenge':
        return target.kind === 'challenge' && u.id === target.number
      case 'boss':
        return false
      default:
        return assertNever(u)
    }
  })
}

export const wantAnswer = (
  { withCatalog }: GraphAnswerOptions,
  target: Target,
): WantView => {
  if (!withCatalog)
    return {
      wanted: { kind: 'unresolved' },
      routes: [],
      diagnostics: [WantDiagnostic.NoCatalog],
    }
  const nodes = payload<UnlockView>('unlock').nodes.map(nodeResolved)
  const node = nodes.find((n) => namesTarget(n, target))
  if (node === undefined)
    return {
      wanted: { kind: 'unresolved' },
      routes: [],
      diagnostics: [WantDiagnostic.NothingUnlocks],
    }
  const first = node.unlocks[0]
  const wanted: WantedView =
    target.kind === 'achievement' || first === undefined
      ? { kind: 'achievement', achievement: node.achievement }
      : { kind: 'target', target: first }
  const state: WantState = node.done
    ? { kind: 'done' }
    : {
        kind: 'chain',
        steps: nodes.filter((n) => !n.done).slice(0, 2),
        unknown: 0,
      }
  return { wanted, routes: [{ node, state }], diagnostics: [] }
}
