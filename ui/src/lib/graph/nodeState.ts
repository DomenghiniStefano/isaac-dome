import { countBy, groupBy } from 'lodash-es'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import { MarkColumnView } from '@/lib/ipc/types'
import type { RequirementView, UnlockNode } from '@/lib/ipc/types'
import { characterLabel } from './characterName'

type Translate = (
  key: MessageKey<MessageSchema>,
  params?: Record<string, unknown>,
) => string

// What a node is, one answer for every screen that draws it (DESIGN-BRIEF.md §7.1). A partial
// node is never unlockable: the graph couldn't interpret at least one of its requirements.
export const NodeState = {
  Done: 'done',
  Now: 'now',
  Blocked: 'blocked',
  Partial: 'partial',
} as const
export type NodeState = (typeof NodeState)[keyof typeof NodeState]

// The order the states are shown in: what's behind you, what you can do, what's in the way,
// what we can't vouch for.
export const stateOrder: NodeState[] = [
  NodeState.Done,
  NodeState.Now,
  NodeState.Blocked,
  NodeState.Partial,
]

// The save wins over the graph: a node done is done, even where the graph could interpret
// only part of it.
export const nodeState = (node: UnlockNode): NodeState => {
  if (node.done) return NodeState.Done
  const { graph } = node
  switch (graph.kind) {
    case 'computed':
      return graph.availableNow ? NodeState.Now : NodeState.Blocked
    case 'partial':
      return NodeState.Partial
    default:
      return assertNever(graph)
  }
}

export const stateCounts = (nodes: UnlockNode[]): Record<NodeState, number> => {
  const counted = countBy(nodes, nodeState)
  const count = (state: NodeState): number => counted[state] ?? 0
  return {
    [NodeState.Done]: count(NodeState.Done),
    [NodeState.Now]: count(NodeState.Now),
    [NodeState.Blocked]: count(NodeState.Blocked),
    [NodeState.Partial]: count(NodeState.Partial),
  }
}

// The kinds of what stands in the way, in the order the why is told: the things the player
// can go and do first, the conditions we couldn't interpret last.
export const RequirementKind = {
  Character: 'character',
  Boss: 'boss',
  Challenge: 'challenge',
  Item: 'item',
  Gate: 'gate',
  Mark: 'mark',
  Counter: 'counter',
  Unknown: 'unknown',
} as const
export type RequirementKind =
  (typeof RequirementKind)[keyof typeof RequirementKind]

const requirementOrder: RequirementKind[] = [
  RequirementKind.Character,
  RequirementKind.Boss,
  RequirementKind.Challenge,
  RequirementKind.Item,
  RequirementKind.Gate,
  RequirementKind.Mark,
  RequirementKind.Counter,
  RequirementKind.Unknown,
]

// What the game calls each column of the completion matrix. Game names, so they stay in
// English like every other one (`DESIGN-BRIEF.md` §12) — data, not something to translate.
const columnName: Record<MarkColumnView, string> = {
  [MarkColumnView.MomsHeart]: "Mom's Heart",
  [MarkColumnView.Isaac]: 'Isaac',
  [MarkColumnView.Satan]: 'Satan',
  [MarkColumnView.BossRush]: 'Boss Rush',
  [MarkColumnView.BlueBaby]: 'Blue Baby',
  [MarkColumnView.TheLamb]: 'The Lamb',
  [MarkColumnView.MegaSatan]: 'Mega Satan',
  [MarkColumnView.Greed]: 'Greed',
  [MarkColumnView.Hush]: 'Hush',
  [MarkColumnView.Delirium]: 'Delirium',
  [MarkColumnView.Mother]: 'Mother',
  [MarkColumnView.TheBeast]: 'The Beast',
}

// A gate and an unknown requirement carry only the file's label, in English: they are
// conditions we deliberately did not guess at. A character is the one kind whose name
// isn't enough on its own — the two forms share it (`docs/BACKLOG.md` B28).
const requirementName = (
  requirement: RequirementView,
  t: Translate,
): string => {
  switch (requirement.kind) {
    case 'character':
      return characterLabel(t, requirement)
    case 'boss':
    case 'challenge':
    case 'item':
      return requirement.name
    case 'gate':
    case 'unknown':
      return requirement.label
    case 'mark':
      return t('graph.markName', {
        boss: columnName[requirement.column],
        character: requirement.characterName,
      })
    // The label is already the boss's own name, in English like every game name.
    case 'counter':
      return requirement.label
    default:
      return assertNever(requirement)
  }
}

export interface RequirementGroup {
  kind: RequirementKind
  names: string[]
}

// "1 character and 2 unknown conditions", not "blocked by 3": what a node is missing,
// grouped. `t` is here for the one name that is composed rather than quoted.
export const missingGroups = (
  node: UnlockNode,
  t: Translate,
): RequirementGroup[] => {
  const byKind = groupBy(node.missing, (requirement) => requirement.kind)
  return requirementOrder.flatMap((kind) => {
    const entries = byKind[kind]
    return entries
      ? [{ kind, names: entries.map((r) => requirementName(r, t)) }]
      : []
  })
}
