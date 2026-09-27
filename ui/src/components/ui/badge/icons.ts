import type { Component, FunctionalComponent } from 'vue'
import { CheckIcon, LockIcon, TargetIcon, TriangleAlertIcon } from '@lucide/vue'
import { h } from 'vue'
import { BadgeVariant } from './variants'

// Unknown is drawn with a "?", as in the kit: a question mark reads as "we can't tell",
// where one more icon would read as one more state.
const QuestionMark: FunctionalComponent = () =>
  h('span', { 'aria-hidden': 'true' }, '?')

// A state is never colour alone (Tokens.dc.html: every state is told apart by shape), so
// each state variant carries its mark; tags have none. A record rather than a switch: a
// variant added without a mark doesn't compile.
export const badgeIcons: Record<BadgeVariant, Component | null> = {
  [BadgeVariant.Done]: CheckIcon,
  // Within reach now — doable, available, open, met and not yet beaten: a target to go for.
  // Not a star, which reads as a favourite, a thing someone chose rather than a state.
  [BadgeVariant.Now]: TargetIcon,
  [BadgeVariant.Blocked]: LockIcon,
  // A partial node is locked like a blocked one; its dashed edge, not its mark, says the
  // graph couldn't interpret everything. It must never borrow the target of Now.
  [BadgeVariant.Partial]: LockIcon,
  [BadgeVariant.Unknown]: QuestionMark,
  [BadgeVariant.Unexpected]: TriangleAlertIcon,
  [BadgeVariant.Tag]: null,
  [BadgeVariant.Challenge]: null,
  [BadgeVariant.Wanted]: null,
}
