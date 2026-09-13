import type { Target, UnlockNode, UnlockView } from '@/lib/ipc/types'

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
  return (
    unlock.nodes.find(
      (n) => n.achievement.kind === 'known' && n.achievement.id === target.id,
    ) ?? null
  )
}
