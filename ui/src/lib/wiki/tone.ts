// The colour a `Chip` or an `EditionBadge` draws with, for a value the wiki, the game or the
// save already gives us (card #90, decision 5): an edition, an item's quality, or a page's
// category. `Tone` names the pair of tokens (`--color-<tone>-surface`/`-foreground` in
// `theme/colors.css`) that `chip/variants.ts` reads — this module says WHICH tone a value
// gets, never how the tone is painted, so the two stay one definition each.
import { Dlc } from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'

export const Tone = {
  EditionRebirth: 'edition-rebirth',
  EditionAfterbirth: 'edition-afterbirth',
  EditionAfterbirthPlus: 'edition-afterbirth-plus',
  EditionRepentance: 'edition-repentance',
  EditionRepentancePlus: 'edition-repentance-plus',
  QualityNone: 'quality-none',
  QualityLow: 'quality-low',
  QualityBronze: 'quality-bronze',
  QualitySilver: 'quality-silver',
  QualityGold: 'quality-gold',
  QualityHighlight: 'quality-highlight',
  CategoryItems: 'category-items',
  CategoryTrinkets: 'category-trinkets',
  CategoryAchievements: 'category-achievements',
  CategoryBosses: 'category-bosses',
  CategoryChallenges: 'category-challenges',
  CategoryCharacters: 'category-characters',
  CategoryTransformations: 'category-transformations',
  CategoryMonsters: 'category-monsters',
  CategoryCardsAndRunes: 'category-cards-and-runes',
  CategoryPickups: 'category-pickups',
  CategoryStages: 'category-stages',
  CategoryVersions: 'category-versions',
} as const
export type Tone = (typeof Tone)[keyof typeof Tone]

const editionTone: Record<Dlc, Tone> = {
  [Dlc.Rebirth]: Tone.EditionRebirth,
  [Dlc.Afterbirth]: Tone.EditionAfterbirth,
  [Dlc.AfterbirthPlus]: Tone.EditionAfterbirthPlus,
  [Dlc.Repentance]: Tone.EditionRepentance,
  [Dlc.RepentancePlus]: Tone.EditionRepentancePlus,
}
export const toneOfEdition = (dlc: Dlc): Tone => editionTone[dlc]

const categoryTone: Record<WikiCategory, Tone> = {
  [WikiCategory.Items]: Tone.CategoryItems,
  [WikiCategory.Trinkets]: Tone.CategoryTrinkets,
  [WikiCategory.Achievements]: Tone.CategoryAchievements,
  [WikiCategory.Bosses]: Tone.CategoryBosses,
  [WikiCategory.Challenges]: Tone.CategoryChallenges,
  [WikiCategory.Characters]: Tone.CategoryCharacters,
  [WikiCategory.Transformations]: Tone.CategoryTransformations,
  [WikiCategory.Monsters]: Tone.CategoryMonsters,
  [WikiCategory.CardsAndRunes]: Tone.CategoryCardsAndRunes,
  [WikiCategory.Pickups]: Tone.CategoryPickups,
  [WikiCategory.Stages]: Tone.CategoryStages,
  [WikiCategory.Versions]: Tone.CategoryVersions,
}
export const toneOfCategory = (category: WikiCategory): Tone =>
  categoryTone[category]

// The catalog's own range, `items_metadata.xml`: -1 is "not in any pool", 0 to 4 are the
// four rated tiers. Keyed by a plain number and not an enum — the catalog hands out a
// number, never a closed Rust type — so a value outside -1..4 falls back to `QualityNone`
// rather than throwing: the same "we don't know" reading -1 itself gets.
const qualityTone: Record<number, Tone> = {
  [-1]: Tone.QualityNone,
  0: Tone.QualityLow,
  1: Tone.QualityBronze,
  2: Tone.QualitySilver,
  3: Tone.QualityGold,
  4: Tone.QualityHighlight,
}
export const toneOfQuality = (quality: number): Tone =>
  qualityTone[quality] ?? Tone.QualityNone
