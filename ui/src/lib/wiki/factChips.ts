import type { Message } from '@/i18n/message'
import { assertNever } from '@/lib/assertNever'
import type { MessagePart } from '@/lib/ipc/errorText'
import { ArticleCategory } from '@/lib/ipc/types'
import type { PageFacts, Target, WikiPageRef } from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'
import { pageKey } from './pageKey'
import { Tone, toneOfQuality } from './tone'

// One chip for one fact a page carries: `value` is the raw datum (a table sorts or tests
// against it), `label` is the translated sentence a `Chip` shows (design decision 3,
// `2026-09-27-wiki-restyle-design.md`). Reuses `MessagePart`, the pair `ipc/errorText.ts`
// already defined for "a key, and the values the translation puts inside it" — the same shape,
// so a chip's label is not a second definition of it.
export interface FactChip {
  key: string
  label: MessagePart
  value: string | number | null
  tone: Tone
}

// One sortable column a table view shows for a category (Task 5): the header and a getter
// over the page. `value` reads `null` for a page whose `facts` is not the kind this column
// belongs to, the same way a filter reads a page that doesn't apply.
export interface FactColumn {
  key: string
  label: Message
  value: (page: WikiPageRef) => string | number | null
}

const chip = (
  key: string,
  label: MessagePart,
  value: string | number | null,
  tone: Tone,
): FactChip => ({ key, label, value, tone })

// The target a challenge names as its character, read by id through the caller's own
// `titleOf` (the wiki store's, in practice) — never by name: a Tainted form shares its base
// form's title, and only the id tells them apart (CLAUDE.md, "resolve by id first").
const targetLabel = (
  target: Target,
  titleOf: (key: string) => string | null,
): string | null => {
  const key = pageKey(target)
  return key === null ? null : (titleOf(key) ?? key)
}

type Facts<K extends PageFacts['kind']> = Extract<PageFacts, { kind: K }>

const itemChips = (f: Facts<'item'>): FactChip[] => {
  const chips: FactChip[] = []
  if (f.quality !== null) {
    chips.push(
      chip(
        'quality',
        { key: 'wiki.qualityChip', params: { quality: f.quality } },
        f.quality,
        toneOfQuality(f.quality),
      ),
    )
  }
  // Active or passive is a defining trait, known for every item, so it is the one chip that
  // never depends on the entry saying more — unlike recharge, which only an active has.
  chips.push(
    chip(
      'template',
      { key: f.activated ? 'wiki.facts.activated' : 'wiki.facts.passive' },
      null,
      Tone.QualityNone,
    ),
  )
  if (f.recharge !== null) {
    chips.push(
      chip(
        'recharge',
        { key: 'wiki.facts.recharge', params: { value: f.recharge } },
        f.recharge,
        Tone.QualityNone,
      ),
    )
  }
  if (f.shopPrice !== null) {
    chips.push(
      chip(
        'shopPrice',
        { key: 'wiki.facts.shopPrice', params: { value: f.shopPrice } },
        f.shopPrice,
        Tone.QualityNone,
      ),
    )
  }
  if (f.devilPrice !== null) {
    chips.push(
      chip(
        'devilPrice',
        { key: 'wiki.facts.devilPrice', params: { value: f.devilPrice } },
        f.devilPrice,
        Tone.QualityNone,
      ),
    )
  }
  for (const tag of f.tags) chips.push(tagChip(tag))
  return chips
}

const tagChip = (tag: string): FactChip =>
  chip(
    `tag:${tag}`,
    { key: 'wiki.facts.tag', params: { value: tag } },
    tag,
    Tone.QualityNone,
  )

const trinketChips = (f: Facts<'trinket'>): FactChip[] => f.tags.map(tagChip)

const achievementChips = (f: Facts<'achievement'>): FactChip[] =>
  f.requirement === ''
    ? []
    : [
        chip(
          'requirement',
          { key: 'wiki.facts.requirement', params: { value: f.requirement } },
          f.requirement,
          Tone.QualityNone,
        ),
      ]

// Boss and entity carry the same two facts (design decision 3's table), so one function reads
// both — `factChips` still matches each variant on its own, since the enum is tagged.
const bossOrEntityChips = (f: {
  baseHp: number | null
  floors: string[]
}): FactChip[] => {
  const chips: FactChip[] = []
  if (f.baseHp !== null) {
    chips.push(
      chip(
        'baseHp',
        { key: 'wiki.facts.baseHp', params: { value: f.baseHp } },
        f.baseHp,
        Tone.QualityNone,
      ),
    )
  }
  if (f.floors.length > 0) {
    const shown = shortList(f.floors, FLOORS_ON_A_CHIP)
    chips.push(
      chip(
        'floors',
        { key: 'wiki.facts.floors', params: { value: shown } },
        shown,
        Tone.QualityNone,
      ),
    )
  }
  return chips
}

// How many floors a chip names before it counts the rest: a boss can appear on a dozen, and a
// chip is one line of a card.
const FLOORS_ON_A_CHIP = 2

// The first `n` names, and how many more there are: `Basement, Caves +10`.
export const shortList = (names: string[], n: number): string => {
  const head = names.slice(0, n).join(', ')
  const rest = names.length - n
  return rest > 0 ? `${head} +${rest}` : head
}

const challengeChips = (
  f: Facts<'challenge'>,
  titleOf: (key: string) => string | null,
): FactChip[] => {
  const chips: FactChip[] = []
  const characterName =
    f.character === null ? null : targetLabel(f.character, titleOf)
  if (characterName !== null) {
    chips.push(
      chip(
        'character',
        { key: 'wiki.facts.character', params: { value: characterName } },
        characterName,
        Tone.QualityNone,
      ),
    )
  }
  if (f.goal !== '') {
    chips.push(
      chip(
        'goal',
        { key: 'wiki.facts.goal', params: { value: f.goal } },
        f.goal,
        Tone.QualityNone,
      ),
    )
  }
  // Only when true, the same reading `InfoboxChallenge`'s restriction list already gives it:
  // "not blindfolded" is the default and says nothing.
  if (f.blindfolded) {
    chips.push(
      chip(
        'blindfolded',
        { key: 'wiki.infobox.blindfolded' },
        null,
        Tone.QualityNone,
      ),
    )
  }
  if (f.curse !== '') {
    chips.push(
      chip(
        'curse',
        { key: 'wiki.facts.curse', params: { value: f.curse } },
        f.curse,
        Tone.QualityNone,
      ),
    )
  }
  return chips
}

// One table for the seven stats, read by both the chips and the columns below: the fact is
// defined once (CLAUDE.md, "one definition per concept").
export const CHARACTER_STAT_KEYS = [
  'health',
  'damage',
  'tears',
  'range',
  'speed',
  'luck',
  'shotSpeed',
] as const
type CharacterStatKey = (typeof CHARACTER_STAT_KEYS)[number]
export const characterStatLabel: Record<CharacterStatKey, Message> = {
  health: 'wiki.facts.health',
  damage: 'wiki.facts.damage',
  tears: 'wiki.facts.tears',
  range: 'wiki.facts.range',
  speed: 'wiki.facts.speed',
  luck: 'wiki.facts.luck',
  shotSpeed: 'wiki.facts.shotSpeed',
}

const characterChips = (f: Facts<'character'>): FactChip[] => {
  const chips: FactChip[] = []
  for (const key of CHARACTER_STAT_KEYS) {
    const value = f[key]
    if (value !== '') {
      chips.push(
        chip(
          key,
          { key: characterStatLabel[key], params: { value } },
          value,
          Tone.QualityNone,
        ),
      )
    }
  }
  if (f.tainted)
    chips.push(
      chip('tainted', { key: 'wiki.facts.tainted' }, null, Tone.QualityNone),
    )
  return chips
}

const transformationChips = (f: Facts<'transformation'>): FactChip[] => {
  const chips: FactChip[] = []
  if (f.requires !== null) {
    chips.push(
      chip(
        'requires',
        { key: 'wiki.facts.requires', params: { value: f.requires } },
        f.requires,
        Tone.QualityNone,
      ),
    )
  }
  chips.push(
    chip(
      'contributors',
      { key: 'wiki.facts.contributors', params: { value: f.contributors } },
      f.contributors,
      Tone.QualityNone,
    ),
  )
  return chips
}

const articleCategoryLabel: Record<ArticleCategory, Message> = {
  [ArticleCategory.Card]: 'wiki.facts.categoryCard',
  [ArticleCategory.Rune]: 'wiki.facts.categoryRune',
  [ArticleCategory.Pickup]: 'wiki.kind.pickup',
  [ArticleCategory.Stage]: 'wiki.kind.stage',
  [ArticleCategory.Version]: 'wiki.kind.version',
}
const articleCategoryTone: Record<ArticleCategory, Tone> = {
  [ArticleCategory.Card]: Tone.CategoryCardsAndRunes,
  [ArticleCategory.Rune]: Tone.CategoryCardsAndRunes,
  [ArticleCategory.Pickup]: Tone.CategoryPickups,
  [ArticleCategory.Stage]: Tone.CategoryStages,
  [ArticleCategory.Version]: Tone.CategoryVersions,
}

const articleChips = (f: Facts<'article'>): FactChip[] => {
  const chips: FactChip[] = []
  if (f.category !== null) {
    chips.push(
      chip(
        'category',
        { key: articleCategoryLabel[f.category] },
        f.category,
        articleCategoryTone[f.category],
      ),
    )
  }
  if (f.version !== null) {
    chips.push(
      chip(
        'versionNumber',
        {
          key: 'wiki.facts.versionNumber',
          params: { value: f.version.number },
        },
        f.version.number,
        Tone.CategoryVersions,
      ),
      chip(
        'versionDate',
        { key: 'wiki.facts.versionDate', params: { value: f.version.date } },
        f.version.date,
        Tone.CategoryVersions,
      ),
    )
  }
  return chips
}

// One chip per meaningful fact a page's `PageFacts` carries (design decision 3). `titleOf`
// resolves a challenge's character reference the same way `refsOf` resolves any other
// (`screens/wiki/infoboxRefs.ts`); every other variant needs none, so it defaults to a
// resolver that answers nothing.
export const factChips = (
  facts: PageFacts,
  titleOf: (key: string) => string | null = () => null,
): FactChip[] => {
  switch (facts.kind) {
    case 'item':
      return itemChips(facts)
    case 'trinket':
      return trinketChips(facts)
    case 'achievement':
      return achievementChips(facts)
    case 'boss':
      return bossOrEntityChips(facts)
    case 'entity':
      return bossOrEntityChips(facts)
    case 'challenge':
      return challengeChips(facts, titleOf)
    case 'character':
      return characterChips(facts)
    case 'transformation':
      return transformationChips(facts)
    case 'article':
      return articleChips(facts)
    default:
      return assertNever(facts)
  }
}

// A column reads its own fact straight off the page, `null` when the page's own `facts` isn't
// the kind this column belongs to — checked at runtime, then narrowed with a cast, since a
// generic `K` cannot narrow `page.facts` by itself the way a literal comparison would.
const scalarColumn = <K extends PageFacts['kind']>(
  kind: K,
  key: string,
  label: Message,
  get: (facts: Facts<K>) => string | number | null,
): FactColumn => ({
  key,
  label,
  value: (page) =>
    page.facts.kind === kind ? get(page.facts as Facts<K>) : null,
})

const itemColumns: FactColumn[] = [
  scalarColumn('item', 'quality', 'wiki.infobox.quality', (f) => f.quality),
  scalarColumn('item', 'template', 'wiki.facts.template', (f) =>
    f.activated ? 1 : 0,
  ),
  scalarColumn('item', 'recharge', 'wiki.infobox.recharge', (f) => f.recharge),
  scalarColumn(
    'item',
    'shopPrice',
    'wiki.infobox.shopPrice',
    (f) => f.shopPrice,
  ),
  scalarColumn(
    'item',
    'devilPrice',
    'wiki.infobox.devilPrice',
    (f) => f.devilPrice,
  ),
]

const achievementColumns: FactColumn[] = [
  scalarColumn(
    'achievement',
    'requirement',
    'wiki.infobox.requirements',
    (f) => f.requirement,
  ),
]

const bossColumns: FactColumn[] = [
  scalarColumn('boss', 'baseHp', 'wiki.infobox.baseHp', (f) => f.baseHp),
  scalarColumn('boss', 'floors', 'wiki.infobox.environment', (f) =>
    f.floors.length > 0 ? f.floors.join(', ') : null,
  ),
]

const entityColumns: FactColumn[] = [
  scalarColumn('entity', 'baseHp', 'wiki.infobox.baseHp', (f) => f.baseHp),
  scalarColumn('entity', 'floors', 'wiki.infobox.environment', (f) =>
    f.floors.length > 0 ? f.floors.join(', ') : null,
  ),
]

const challengeColumns: FactColumn[] = [
  scalarColumn('challenge', 'character', 'wiki.facts.character', (f) =>
    f.character !== null && f.character.kind === 'character'
      ? f.character.id
      : null,
  ),
  scalarColumn('challenge', 'goal', 'wiki.infobox.goal', (f) => f.goal),
  scalarColumn('challenge', 'blindfolded', 'wiki.infobox.blindfolded', (f) =>
    f.blindfolded ? 1 : 0,
  ),
  scalarColumn('challenge', 'curse', 'wiki.infobox.curse', (f) => f.curse),
]

const characterColumns: FactColumn[] = [
  ...CHARACTER_STAT_KEYS.map((key) =>
    scalarColumn('character', key, characterStatLabel[key], (f) => f[key]),
  ),
  scalarColumn('character', 'tainted', 'wiki.facts.tainted', (f) =>
    f.tainted ? 1 : 0,
  ),
]

const transformationColumns: FactColumn[] = [
  scalarColumn(
    'transformation',
    'requires',
    'wiki.infobox.requires',
    (f) => f.requires,
  ),
  scalarColumn(
    'transformation',
    'contributors',
    'wiki.infobox.contributors',
    (f) => f.contributors,
  ),
]

const articleCategoryColumns: FactColumn[] = [
  scalarColumn('article', 'category', 'wiki.facts.category', (f) => f.category),
]

const articleVersionColumns: FactColumn[] = [
  scalarColumn(
    'article',
    'versionNumber',
    'wiki.facts.versionNumber',
    (f) => f.version?.number ?? null,
  ),
  scalarColumn(
    'article',
    'versionDate',
    'wiki.facts.versionDate',
    (f) => f.version?.date ?? null,
  ),
]

// The ordered, sortable columns a table view shows for a category (Task 5), read from the
// same field definitions `factChips` reads: a fact is one table, not two. Tags carry no
// column: a list is not a sortable scalar, and belongs to the list's own filter instead.
export const factColumns = (category: WikiCategory): FactColumn[] => {
  switch (category) {
    case WikiCategory.Items:
      return itemColumns
    case WikiCategory.Trinkets:
      return []
    case WikiCategory.Achievements:
      return achievementColumns
    case WikiCategory.Bosses:
      return bossColumns
    case WikiCategory.Monsters:
      return entityColumns
    case WikiCategory.Challenges:
      return challengeColumns
    case WikiCategory.Characters:
      return characterColumns
    case WikiCategory.Transformations:
      return transformationColumns
    case WikiCategory.CardsAndRunes:
    case WikiCategory.Pickups:
    case WikiCategory.Stages:
      return articleCategoryColumns
    case WikiCategory.Versions:
      return articleVersionColumns
    default:
      return assertNever(category)
  }
}
