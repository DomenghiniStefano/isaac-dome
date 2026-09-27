import { StateTone } from '@/lib/facets/stateTone'
import { BadgeVariant } from './variants'

// The badge a state tone is painted with. `lib/` names a state's tone and may not import a
// component; a component that shows one reads this, the one place the two are joined.
export const stateBadgeVariant: Record<StateTone, BadgeVariant> = {
  [StateTone.Done]: BadgeVariant.Done,
  [StateTone.Now]: BadgeVariant.Now,
  [StateTone.Blocked]: BadgeVariant.Blocked,
  [StateTone.Partial]: BadgeVariant.Partial,
  [StateTone.Unknown]: BadgeVariant.Unknown,
}
