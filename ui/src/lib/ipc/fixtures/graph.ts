import { assertNever } from '@/lib/assertNever'
import { knownId } from '@/lib/graph/achievementNode'
import { WantDiagnostic } from '../types'
import type {
  NextSteps,
  RequirementView,
  StepsBasis,
  StepsSection,
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

// The same story as `withPage` below, on the other half of the row (B35): a payload written
// before a target carried its page has none, and an absent key would read in a template
// exactly like "the dataset has no page". Filled with `null` — which is that sentence, said
// on purpose — and declared once in the console. All four variants, because all four carry it.
let warnedAboutTargetPages = false
const targetWithPage = (target: UnlockTarget): UnlockTarget => {
  if (target.page !== undefined) return target
  if (!warnedAboutTargetPages) {
    warnedAboutTargetPages = true
    console.warn(
      'graph fixture: unlock.json predates the page a target links to; nothing a node unlocks links on the development server',
    )
  }
  return { ...target, page: null }
}

// The payload was written before a character carried its form (`docs/BACKLOG.md` B28), so
// the field is absent there: absent reads as the base form, which is right for every base
// character and wrong for the Tainted ones. Declared once in the console.
let warnedAboutForms = false
const withForm = <T extends { kind: string; tainted?: boolean }>(
  value: T,
): T => {
  if (value.kind !== 'character' || typeof value.tainted === 'boolean')
    return value
  if (!warnedAboutForms) {
    warnedAboutForms = true
    console.warn(
      "graph fixture: unlock.json predates the character's tainted flag; every character reads as its base form",
    )
  }
  return { ...value, tainted: false }
}

// Same story for the page a requirement links to (spec 3.5d): a payload written before the
// field has none, and an absent key would read in a template exactly like "the dataset has no
// page". It is filled with `null` — which is that sentence, said on purpose — and declared
// once on the console.
let warnedAboutPages = false
const withPage = (requirement: RequirementView): RequirementView => {
  switch (requirement.kind) {
    case 'gate':
    case 'mark':
    case 'counter':
    case 'unknown':
      return requirement
    // A payload written before the transformations has no threshold in it at all, so there
    // is nothing to fill in: what arrives already carries its own page.
    case 'threshold':
      return requirement
    case 'character':
    case 'boss':
    case 'challenge':
    case 'item':
      if (requirement.page !== undefined) return requirement
      if (!warnedAboutPages) {
        warnedAboutPages = true
        console.warn(
          "graph fixture: unlock.json predates the requirement's page; nothing links on the development server",
        )
      }
      return { ...requirement, page: null }
    default:
      return assertNever(requirement)
  }
}

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
  unlocks: node.unlocks.map((t) =>
    targetWithPage(withForm(targetWithoutIcon(t))),
  ),
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
  withCatalog: boolean
}

export interface GraphAnswers {
  unlock: UnlockView
  steps: NextSteps
}

// The payload predates the sections: it is one flat list and the basis that made it. That
// shape *is* one section — the one it always was — so it is read as such and declared once.
interface FlatNextSteps {
  steps: UnlockNode[]
  basis: StepsBasis
}
let warnedAboutSections = false
const sectionsOf = (value: NextSteps | FlatNextSteps): StepsSection[] => {
  if ('sections' in value) return value.sections
  if (!warnedAboutSections) {
    warnedAboutSections = true
    console.warn(
      "graph fixture: next_steps.json predates the steps' sections; the whole list reads as its one basis",
    )
  }
  // Never a heading over nothing: an empty list is no section, exactly as in Rust.
  return value.steps.length === 0
    ? []
    : [{ basis: value.basis, steps: value.steps }]
}

export const graphAnswers = ({
  withCatalog,
}: GraphAnswerOptions): GraphAnswers => {
  const unlock = payload<UnlockView>('unlock')
  const sections = sectionsOf(payload<NextSteps | FlatNextSteps>('next_steps'))
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
