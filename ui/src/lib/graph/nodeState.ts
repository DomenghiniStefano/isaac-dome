import { countBy, groupBy } from 'lodash-es'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import { MarkColumnView } from '@/lib/ipc/types'
import type { RequirementView, UnlockNode } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
import type { TabLocation } from '@/router/routeTable'
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
  Threshold: 'threshold',
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
  RequirementKind.Threshold,
  RequirementKind.Unknown,
]

// What the game calls each column of the completion matrix. Game names, so they stay in
// English like every other one (`DESIGN-BRIEF.md` §12) — data, not something to translate.
// Exported since Live: the same twelve names, and a second table would be a second chance to
// disagree with the matrix about what a column is called.
export const columnName: Record<MarkColumnView, string> = {
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
    case 'threshold':
      return t('graph.thresholdName', {
        name: requirement.label,
        current: requirement.current,
        atLeast: requirement.atLeast,
      })
    default:
      return assertNever(requirement)
  }
}

// A threshold is the one requirement that is a set: naming it alone would say "you need
// three Guppy items" and leave the player to go and find out which. So it draws as its own
// row plus one per item still locked — the ones already unlocked are not in the way, and
// listing them would bury the answer in the question.
const thresholdEntries = (
  requirement: Extract<RequirementView, { kind: 'threshold' }>,
  t: Translate,
): RequirementEntry[] => [
  {
    key: `threshold-${requirement.transformation}`,
    name: requirementName(requirement, t),
    location: requirement.page ? pageLocation(requirement.page) : null,
  },
  ...requirement.of
    .filter((item) => !item.unlocked)
    .map((item) => ({
      key: `threshold-${requirement.transformation}-${item.id}`,
      name: item.name,
      location: item.page ? pageLocation(item.page) : null,
    })),
]

// A key that is stable per row and unique in the list: the kind and what identifies it. A
// gate, a counter and an uninterpreted label have only their text, and two of them never
// repeat inside one node; a mark is one cell, so it is the character and the column.
const requirementKey = (requirement: RequirementView): string => {
  switch (requirement.kind) {
    case 'character':
    case 'boss':
    case 'challenge':
    case 'item':
      return `${requirement.kind}-${requirement.id}`
    case 'mark':
      return `mark-${requirement.character}-${requirement.column}`
    case 'threshold':
      return `threshold-${requirement.transformation}`
    case 'gate':
    case 'counter':
    case 'unknown':
      return `${requirement.kind}-${requirement.label}`
    default:
      return assertNever(requirement)
  }
}

// Where to read how *this* is unlocked. `null` is "nowhere to go": either the dataset has no
// page, or the requirement is a condition and not an entity — a gate, a mark, a counter, an
// uninterpreted label. Never a link that leads nowhere.
const requirementLocation = (
  requirement: RequirementView,
): TabLocation | null => {
  switch (requirement.kind) {
    case 'character':
    case 'boss':
    case 'challenge':
    case 'item':
      return requirement.page ? pageLocation(requirement.page) : null
    case 'gate':
    case 'mark':
    case 'counter':
    case 'unknown':
      return null
    // Its own page, which the dataset has had since the sixteen were imported.
    case 'threshold':
      return requirement.page ? pageLocation(requirement.page) : null
    default:
      return assertNever(requirement)
  }
}

// What stands in the way, and where to read about it.
export interface RequirementEntry {
  key: string
  name: string
  location: TabLocation | null
}

export interface RequirementGroup {
  kind: RequirementKind
  entries: RequirementEntry[]
}

// "1 character and 2 unknown conditions", not "blocked by 3": what a node is missing,
// grouped. `t` is here for the one name that is composed rather than quoted.
export const missingGroups = (
  node: UnlockNode,
  t: Translate,
): RequirementGroup[] => {
  const byKind = groupBy(node.missing, (requirement) => requirement.kind)
  return requirementOrder.flatMap((kind) => {
    const of = byKind[kind]
    return of
      ? [
          {
            kind,
            // `flatMap`, because a threshold is a set and answers with several rows where
            // every other kind answers with one.
            entries: of.flatMap((r) =>
              r.kind === 'threshold'
                ? thresholdEntries(r, t)
                : [
                    {
                      key: requirementKey(r),
                      name: requirementName(r, t),
                      location: requirementLocation(r),
                    },
                  ],
            ),
          },
        ]
      : []
  })
}
