import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { SectionKind } from '@/lib/ipc/types'
import type { Target } from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'

type Message = MessageKey<MessageSchema>

// A section's name for the reader (Schermate.dc.html, SEC_LABEL). A record over the whole
// set: a kind with no name fails to compile.
export const sectionText: Record<SectionKind, Message> = {
  [SectionKind.Effects]: 'wiki.section.effects',
  [SectionKind.Notes]: 'wiki.section.notes',
  [SectionKind.Synergies]: 'wiki.section.synergies',
  [SectionKind.Interactions]: 'wiki.section.interactions',
  [SectionKind.Bugs]: 'wiki.section.bugs',
  [SectionKind.Behavior]: 'wiki.section.behavior',
  [SectionKind.ChampionVersions]: 'wiki.section.championVersions',
  [SectionKind.DamageScaling]: 'wiki.section.damageScaling',
  [SectionKind.Strategies]: 'wiki.section.strategies',
  [SectionKind.Difficulty]: 'wiki.section.difficulty',
  [SectionKind.Reward]: 'wiki.section.reward',
  [SectionKind.Unlockable]: 'wiki.section.unlockable',
}

// A page's kind badge: the category's singular, by the page's identity.
export const kindText: Record<WikiCategory, Message> = {
  [WikiCategory.Items]: 'wiki.kind.item',
  [WikiCategory.Trinkets]: 'wiki.kind.trinket',
  [WikiCategory.Achievements]: 'wiki.kind.achievement',
  [WikiCategory.Bosses]: 'wiki.kind.boss',
  [WikiCategory.Challenges]: 'wiki.kind.challenge',
  [WikiCategory.Characters]: 'wiki.kind.character',
}

// The id a list row prints under the title: the number the game knows the page by.
export const pageId = (target: Target): number | null => {
  switch (target.kind) {
    case 'item':
    case 'trinket':
    case 'achievement':
    case 'character':
    case 'entity':
    case 'transformation':
      return target.id
    case 'challenge':
      return target.number
    case 'stage':
    case 'room':
    case 'pickup':
      return null
  }
}
