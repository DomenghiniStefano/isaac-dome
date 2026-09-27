import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'
import { Tone } from '@/lib/wiki/tone'

// One class string per tone: the pair of tokens `tone.ts` named, read here and nowhere
// else. There is no third "line" token in the wiki restyle's colour families (card #90,
// decision 5 — only `-surface`/`-foreground`), so the border is the same bright colour the
// text is, the way `BadgeVariant.Unknown` already borrows its own foreground for its edge.
// Exported so a surface that wants the same tone painted large — the landing's tiles and hero
// (card #90, decision 8), never a pill of their own — read the one definition instead of a
// second hand-written map: Tailwind's class scanner needs the literal strings written
// somewhere, and this is where `Chip` already keeps them.
export const toneClass: Record<Tone, string> = {
  [Tone.EditionRebirth]:
    'border-edition-rebirth-foreground bg-edition-rebirth-surface text-edition-rebirth-foreground',
  [Tone.EditionAfterbirth]:
    'border-edition-afterbirth-foreground bg-edition-afterbirth-surface text-edition-afterbirth-foreground',
  [Tone.EditionAfterbirthPlus]:
    'border-edition-afterbirth-plus-foreground bg-edition-afterbirth-plus-surface text-edition-afterbirth-plus-foreground',
  [Tone.EditionRepentance]:
    'border-edition-repentance-foreground bg-edition-repentance-surface text-edition-repentance-foreground',
  [Tone.EditionRepentancePlus]:
    'border-edition-repentance-plus-foreground bg-edition-repentance-plus-surface text-edition-repentance-plus-foreground',
  [Tone.QualityNone]:
    'border-quality-none-foreground bg-quality-none-surface text-quality-none-foreground',
  [Tone.QualityLow]:
    'border-quality-low-foreground bg-quality-low-surface text-quality-low-foreground',
  [Tone.QualityBronze]:
    'border-quality-bronze-foreground bg-quality-bronze-surface text-quality-bronze-foreground',
  [Tone.QualitySilver]:
    'border-quality-silver-foreground bg-quality-silver-surface text-quality-silver-foreground',
  [Tone.QualityGold]:
    'border-quality-gold-foreground bg-quality-gold-surface text-quality-gold-foreground',
  [Tone.QualityHighlight]:
    'border-quality-highlight-foreground bg-quality-highlight-surface text-quality-highlight-foreground',
  [Tone.CategoryItems]:
    'border-category-items-foreground bg-category-items-surface text-category-items-foreground',
  [Tone.CategoryTrinkets]:
    'border-category-trinkets-foreground bg-category-trinkets-surface text-category-trinkets-foreground',
  [Tone.CategoryAchievements]:
    'border-category-achievements-foreground bg-category-achievements-surface text-category-achievements-foreground',
  [Tone.CategoryBosses]:
    'border-category-bosses-foreground bg-category-bosses-surface text-category-bosses-foreground',
  [Tone.CategoryChallenges]:
    'border-category-challenges-foreground bg-category-challenges-surface text-category-challenges-foreground',
  [Tone.CategoryCharacters]:
    'border-category-characters-foreground bg-category-characters-surface text-category-characters-foreground',
  [Tone.CategoryTransformations]:
    'border-category-transformations-foreground bg-category-transformations-surface text-category-transformations-foreground',
  [Tone.CategoryMonsters]:
    'border-category-monsters-foreground bg-category-monsters-surface text-category-monsters-foreground',
  [Tone.CategoryCardsAndRunes]:
    'border-category-cards-and-runes-foreground bg-category-cards-and-runes-surface text-category-cards-and-runes-foreground',
  [Tone.CategoryPickups]:
    'border-category-pickups-foreground bg-category-pickups-surface text-category-pickups-foreground',
  [Tone.CategoryStages]:
    'border-category-stages-foreground bg-category-stages-surface text-category-stages-foreground',
  [Tone.CategoryVersions]:
    'border-category-versions-foreground bg-category-versions-surface text-category-versions-foreground',
}

export const chipVariants = cva(
  'inline-flex w-fit shrink-0 items-center gap-1 rounded-full border px-2.5 py-0.5 text-caption whitespace-nowrap',
  {
    variants: { tone: toneClass },
    defaultVariants: { tone: Tone.QualityLow },
  },
)
export type ChipVariants = VariantProps<typeof chipVariants>
