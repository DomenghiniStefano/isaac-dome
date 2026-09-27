import { sortBy, uniq } from 'lodash-es'
import type { Message, Translate } from '@/i18n/message'
import { assertNever } from '@/lib/assertNever'
import { createFaceting, emptyFilter } from '@/lib/facets/faceting'
import type { Faceting, FacetFilter } from '@/lib/facets/faceting'
import type { FacetSlot } from '@/lib/facets/facetOptions'
import type { FilterBarDescriptor, StateRow } from '@/lib/facets/filterBar'
import type { FilterBarLabels } from '@/lib/facets/labels'
import { StateTone, stateDots } from '@/lib/facets/stateTone'
import { ArticleCategory, Dlc } from '@/lib/ipc/types'
import type { PageProgress, Target, WikiPageRef } from '@/lib/ipc/types'
import { oneOf } from '@/lib/oneOf'
import { WikiCategory } from '@/router/routeTable'
import { dlcNames } from './dlcNames'
import { factColumns } from './factChips'
import { isComplete } from './progress'

// A category's own filters, over `PageFacts` and the save's `PageProgress`:
// the edition every page carries, the save's state where the category has
// one, and each kind's own facet. One union for all twelve categories — a facet a page's own
// kind does not carry simply answers no value, the same way an inapplicable fact chip draws
// nothing — so only `wikiSlotsForCategory` below decides which of them a given list offers.
export const WikiFacet = {
  Profile: 'profile',
  Edition: 'edition',
  Quality: 'quality',
  Template: 'template',
  Tag: 'tag',
  Tainted: 'tainted',
  ArticleCategory: 'articleCategory',
} as const
export type WikiFacet = (typeof WikiFacet)[keyof typeof WikiFacet]

export const wikiFacetOrder: WikiFacet[] = [
  WikiFacet.Profile,
  WikiFacet.Edition,
  WikiFacet.Quality,
  WikiFacet.Template,
  WikiFacet.Tag,
  WikiFacet.Tainted,
  WikiFacet.ArticleCategory,
]

export type WikiListFilter = FacetFilter<WikiFacet>
export const emptyWikiListFilter = (): WikiListFilter =>
  emptyFilter(wikiFacetOrder)

export const ProfileValue = {
  Done: 'done',
  NotDone: 'notDone',
  NoData: 'noData',
} as const
export type ProfileValue = (typeof ProfileValue)[keyof typeof ProfileValue]
const profileOrder: string[] = Object.values(ProfileValue)

// No save, no progress entry, or a kind whose `isComplete` answers nothing (a transformation,
// a stage): all the same "no data" a reader can pick, never a guessed "not done" (`progress.ts`).
const profileValue = (progress: PageProgress | null): ProfileValue => {
  if (progress === null) return ProfileValue.NoData
  const complete = isComplete(progress)
  return complete === null
    ? ProfileValue.NoData
    : complete
      ? ProfileValue.Done
      : ProfileValue.NotDone
}

// An entry with no restriction has existed since Rebirth and still does — present in every
// edition — the same reading `editionAdded`/`editionRemoved` give the badge (`edition.ts`).
const editionValues = (dlc: Dlc[]): Dlc[] =>
  dlc.length === 0 ? Object.values(Dlc) : dlc

// One page's values for one facet, `progressFor` closed over the caller's own save lookup
// (the wiki store's, in practice) so this stays a pure function of its three arguments.
export const wikiFacetValues = (
  page: WikiPageRef,
  facet: WikiFacet,
  progressFor: (target: Target) => PageProgress | null,
): string[] => {
  switch (facet) {
    case WikiFacet.Profile:
      return [profileValue(progressFor(page.target))]
    case WikiFacet.Edition:
      return editionValues(page.dlc)
    case WikiFacet.Quality:
      return page.facts.kind === 'item'
        ? [page.facts.quality === null ? 'unrated' : String(page.facts.quality)]
        : []
    case WikiFacet.Template:
      return page.facts.kind === 'item'
        ? [page.facts.activated ? 'activated' : 'passive']
        : []
    case WikiFacet.Tag:
      return page.facts.kind === 'item' || page.facts.kind === 'trinket'
        ? page.facts.tags
        : []
    case WikiFacet.Tainted:
      return page.facts.kind === 'character' ? [String(page.facts.tainted)] : []
    case WikiFacet.ArticleCategory:
      return page.facts.kind === 'article' && page.facts.category !== null
        ? [page.facts.category]
        : []
    default:
      return assertNever(facet)
  }
}

const qualityOrder: string[] = ['4', '3', '2', '1', '0', '-1', 'unrated']
const taintedOrder: string[] = ['false', 'true']
const articleCategoryOrder: string[] = [
  ArticleCategory.Card,
  ArticleCategory.Rune,
]

const tagsOf = (page: WikiPageRef): string[] =>
  page.facts.kind === 'item' || page.facts.kind === 'trinket'
    ? page.facts.tags
    : []

// A facet's offered values: fixed sets for everything the wiki or the catalog already closes
// (the five editions, the profile's three states, a quality tier, active or passive, tainted
// or not, card or rune), and only the tags read off the pages actually shown — the game's
// vocabulary is open by nature (`PageFacts.tags`'s own doc comment).
export const wikiFacetOptions = (
  pages: WikiPageRef[],
  facet: WikiFacet,
): string[] => {
  switch (facet) {
    case WikiFacet.Profile:
      return profileOrder
    case WikiFacet.Edition:
      return Object.values(Dlc)
    case WikiFacet.Quality:
      return qualityOrder
    case WikiFacet.Template:
      return ['activated', 'passive']
    case WikiFacet.Tag:
      return sortBy(uniq(pages.flatMap(tagsOf)))
    case WikiFacet.Tainted:
      return taintedOrder
    case WikiFacet.ArticleCategory:
      return articleCategoryOrder
    default:
      return assertNever(facet)
  }
}

// A category's half of a faceted list, over every page the window holds — `filterPages`
// (`listFilter.ts`) narrows to the one category first, the same order `unlockFaceting` and
// `collectionFaceting` already read their own rows in.
export const wikiFaceting = (
  progressFor: (target: Target) => PageProgress | null,
): Faceting<WikiPageRef, WikiFacet> =>
  createFaceting<WikiPageRef, WikiFacet>({
    order: wikiFacetOrder,
    values: (page, facet) => wikiFacetValues(page, facet, progressFor),
    text: (page) => page.title,
    options: wikiFacetOptions,
  })

// Which of the non-profile facets a category offers, and where (design decision 4's table):
// quality and active/passive lead for items, tags for items and trinkets, tainted for
// characters, card-or-rune for the one landing tile that mixes the two. Every category gets
// the edition, behind the fold — it is the one facet every kind carries, so it is never the
// reason a list looks empty of filters.
export const wikiSlotsForCategory = (
  category: WikiCategory,
): FacetSlot<WikiFacet>[] => {
  switch (category) {
    case WikiCategory.Items:
      return [
        { facet: WikiFacet.Quality, inView: true },
        { facet: WikiFacet.Template, inView: true },
        { facet: WikiFacet.Tag, inView: false },
        { facet: WikiFacet.Edition, inView: false },
      ]
    case WikiCategory.Trinkets:
      return [
        { facet: WikiFacet.Tag, inView: true },
        { facet: WikiFacet.Edition, inView: false },
      ]
    case WikiCategory.Characters:
      return [
        { facet: WikiFacet.Tainted, inView: true },
        { facet: WikiFacet.Edition, inView: false },
      ]
    case WikiCategory.CardsAndRunes:
      return [
        { facet: WikiFacet.ArticleCategory, inView: true },
        { facet: WikiFacet.Edition, inView: false },
      ]
    case WikiCategory.Achievements:
    case WikiCategory.Bosses:
    case WikiCategory.Challenges:
    case WikiCategory.Transformations:
    case WikiCategory.Monsters:
    case WikiCategory.Pickups:
    case WikiCategory.Stages:
    case WikiCategory.Versions:
      return [{ facet: WikiFacet.Edition, inView: false }]
    default:
      return assertNever(category)
  }
}

const facetTitle: Record<WikiFacet, Message> = {
  [WikiFacet.Profile]: 'wiki.list.facet.profile',
  [WikiFacet.Edition]: 'wiki.list.facet.edition',
  [WikiFacet.Quality]: 'wiki.list.facet.quality',
  [WikiFacet.Template]: 'wiki.list.facet.template',
  [WikiFacet.Tag]: 'wiki.list.facet.tag',
  [WikiFacet.Tainted]: 'wiki.list.facet.tainted',
  [WikiFacet.ArticleCategory]: 'wiki.list.facet.articleCategory',
}

const profileText: Record<string, Message> = {
  [ProfileValue.Done]: 'wiki.progress.done',
  [ProfileValue.NotDone]: 'wiki.progress.notDone',
  [ProfileValue.NoData]: 'wiki.list.noData',
}

const profileDot = stateDots<ProfileValue>({
  [ProfileValue.Done]: StateTone.Done,
  [ProfileValue.NotDone]: StateTone.Blocked,
  [ProfileValue.NoData]: StateTone.Unknown,
})

const barLabels: FilterBarLabels = {
  rows: 'wiki.list.rows',
  search: 'wiki.search',
  sortBy: 'wiki.list.sortBy',
}

// A picked value in words: everything here is a value the wiki, the game or the save already
// states, so most of it is a lookup and not a sentence of its own.
export const wikiFacetValueLabel = (
  t: Translate,
  facet: WikiFacet,
  value: string,
): string => {
  switch (facet) {
    case WikiFacet.Profile: {
      const picked = oneOf(ProfileValue, value)
      return picked ? t(profileText[picked]) : value
    }
    case WikiFacet.Edition: {
      const dlc = oneOf(Dlc, value)
      return dlc ? dlcNames[dlc] : value
    }
    case WikiFacet.Quality:
      return value === 'unrated'
        ? t('wiki.list.qualityUnrated')
        : t('wiki.qualityChip', { quality: Number(value) })
    case WikiFacet.Template:
      return t(
        value === 'activated' ? 'wiki.facts.activated' : 'wiki.facts.passive',
      )
    case WikiFacet.Tag:
      return value
    case WikiFacet.Tainted:
      return t(value === 'true' ? 'wiki.facts.tainted' : 'wiki.list.baseForm')
    case WikiFacet.ArticleCategory: {
      const category = oneOf(ArticleCategory, value)
      return category === ArticleCategory.Rune
        ? t('wiki.facts.categoryRune')
        : t('wiki.facts.categoryCard')
    }
    default:
      return assertNever(facet)
  }
}

// The three sort keys every category offers before its own fact columns (design decision 4):
// a page's name, its id — left out where the kind has none, an article among them — and the
// edition it was added in.
export const WikiSort = {
  Name: 'name',
  Id: 'id',
  Edition: 'edition',
} as const
export type WikiSortBuiltin = (typeof WikiSort)[keyof typeof WikiSort]
export type WikiSortKey = string

const builtinSortText: Record<WikiSortBuiltin, Message> = {
  [WikiSort.Name]: 'wiki.list.sort.name',
  [WikiSort.Id]: 'wiki.list.sort.id',
  [WikiSort.Edition]: 'wiki.list.sort.edition',
}

// The four article categories have no numeric id (`pageId`'s own doc comment): an article is
// keyed by its title, so a sort — or a table column — that named "id" would always read
// "missing" for every row.
export const categoryHasId = (category: WikiCategory): boolean => {
  switch (category) {
    case WikiCategory.CardsAndRunes:
    case WikiCategory.Pickups:
    case WikiCategory.Stages:
    case WikiCategory.Versions:
      return false
    case WikiCategory.Items:
    case WikiCategory.Trinkets:
    case WikiCategory.Achievements:
    case WikiCategory.Bosses:
    case WikiCategory.Challenges:
    case WikiCategory.Characters:
    case WikiCategory.Transformations:
    case WikiCategory.Monsters:
      return true
    default:
      return assertNever(category)
  }
}

export const wikiSortOrder = (category: WikiCategory): WikiSortKey[] => [
  WikiSort.Name,
  ...(categoryHasId(category) ? [WikiSort.Id] : []),
  WikiSort.Edition,
  ...factColumns(category).map((column) => column.key),
]

export const wikiSortText = (
  category: WikiCategory,
): Record<string, Message> => ({
  ...builtinSortText,
  ...Object.fromEntries(
    factColumns(category).map((column) => [column.key, column.label]),
  ),
})

// A category's whole filter bar: the faceting the screen built
// over its own `progressFor`, which facets this category offers and where, the profile's state
// row — empty when the category has no save state to show, which draws nothing rather than a
// row of squares that would always read "no data" — and the sort keys.
export const wikiBar = (
  category: WikiCategory,
  hasProgress: boolean,
  faceting: Faceting<WikiPageRef, WikiFacet>,
): FilterBarDescriptor<WikiPageRef, WikiFacet, WikiSortKey> => ({
  faceting,
  facets: wikiSlotsForCategory(category),
  state: {
    facet: WikiFacet.Profile,
    order: hasProgress ? profileOrder : [],
    dot: profileDot,
    text: profileText,
  } satisfies StateRow<WikiFacet>,
  title: facetTitle,
  labels: barLabels,
  sorts: {
    order: wikiSortOrder(category),
    text: wikiSortText(category),
  },
})
