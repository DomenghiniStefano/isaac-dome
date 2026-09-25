import type {
  AchievementRef,
  Target,
  UnlockNode,
  UnlockView,
} from '@/lib/ipc/types'

// An achievement the catalog can name. The other variant is a slot and nothing else: no id to
// queue, no text to show, no page to open.
export type KnownAchievement = Extract<AchievementRef, { kind: 'known' }>

export const knownAchievement = (node: UnlockNode): KnownAchievement | null =>
  node.achievement.kind === 'known' ? node.achievement : null

// `null` and not `-1`: a sentinel number would be a valid argument to the queue command.
export const knownId = (node: UnlockNode): number | null =>
  knownAchievement(node)?.id ?? null

export const knownText = (node: UnlockNode): string | null =>
  knownAchievement(node)?.text ?? null

// The node for one achievement id, never an unknown slot that happens to carry the same number:
// a slot's number is its position in the save, not an id.
export const nodeWithId = (
  nodes: readonly UnlockNode[],
  id: number,
): UnlockNode | null => nodes.find((n) => knownId(n) === id) ?? null

// The node a wiki page names, when the page is an achievement and the profile can answer for
// it. Not a join — `UnlockView` arrives already resolved; this picks the row (spec §3.2).
//
// Six answers are `null`, and each one is a state the block must not draw:
//   - no view: the wiki is reachable without a profile, and far more often than with one;
//   - no page, or a page that is not an achievement: every other kind gets its block later;
//   - an id the catalog does not know;
//   - a slot the catalog cannot name — `unknown` has no id, and its slot number is not one:
//     matching by position would put the wrong profile state on the page;
//   - a save whose achievement section was never read. There every node reads "not done",
//     which is not a fact about the profile but the absence of one
//     (`UnlockDiagnostic.noAchievementSection`), and it is the case that would otherwise
//     show a confident, wrong answer rather than nothing.
export const achievementNode = (
  unlock: UnlockView | null,
  target: Target | null,
): UnlockNode | null => {
  if (unlock === null || target === null || target.kind !== 'achievement')
    return null
  if (unlock.diagnostics.some((d) => d.kind === 'noAchievementSection'))
    return null
  return nodeWithId(unlock.nodes, target.id)
}
