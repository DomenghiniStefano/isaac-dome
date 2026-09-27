import { describe, expect, it } from 'vitest'
import type { PageFacts, PageProgress, WikiPageRef } from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'
import { filterPages } from './listFilter'
import {
  ProfileValue,
  WikiFacet,
  emptyWikiListFilter,
  wikiFaceting,
} from './listFacets'

// This test only checks title/category/facet filtering, never a fact drawn from a page: every
// kind reads as "not stated", the same shape a page with an empty infobox gets from the real
// `facts()`.
const factsFor = (target: WikiPageRef['target']): PageFacts => {
  switch (target.kind) {
    case 'item':
      return {
        kind: 'item',
        quality: null,
        activated: false,
        recharge: null,
        shopPrice: null,
        devilPrice: null,
        tags: [],
      }
    case 'trinket':
      return { kind: 'trinket', tags: [] }
    case 'achievement':
      return { kind: 'achievement', requirement: '', unlocks: null }
    case 'entity':
      return { kind: 'boss', baseHp: null, floors: [] }
    case 'challenge':
      return {
        kind: 'challenge',
        character: null,
        goal: '',
        blindfolded: false,
        curse: '',
      }
    case 'character':
      return {
        kind: 'character',
        health: '',
        damage: '',
        tears: '',
        range: '',
        speed: '',
        luck: '',
        shotSpeed: '',
        tainted: false,
      }
    case 'transformation':
    case 'stage':
    case 'room':
    case 'concept':
    case 'article':
      return { kind: 'article', category: null, version: null }
  }
}

const page = (
  target: WikiPageRef['target'],
  title: string,
  category: WikiPageRef['category'],
): WikiPageRef => ({
  target,
  title,
  iconUrl: null,
  category,
  dlc: [],
  facts: factsFor(target),
})
const pages: WikiPageRef[] = [
  page({ kind: 'item', id: 105 }, 'The D6', WikiCategory.Items),
  page({ kind: 'item', id: 27 }, 'Wooden Spoon', WikiCategory.Items),
  page(
    { kind: 'entity', id: 20, variant: 0, subtype: 0 },
    'Monstro',
    WikiCategory.Bosses,
  ),
  page({ kind: 'item', id: 2 }, 'Brimstone', WikiCategory.Items),
  // Both an `entity` target: `page.category` is what tells the boss from the common
  // enemy, since the target's own kind cannot (design decision 2).
  page(
    { kind: 'entity', id: 45, variant: 0, subtype: 0 },
    'Gaper',
    WikiCategory.Monsters,
  ),
]
const titles = (list: WikiPageRef[]) => list.map((p) => p.title)

const noSave = wikiFaceting(() => null)

const withQuery = (query: string) => ({ ...emptyWikiListFilter(), query })

describe('filterPages', () => {
  it("lists a category's pages, in the order they arrive", () => {
    expect(
      titles(filterPages(pages, WikiCategory.Items, withQuery(''), noSave)),
    ).toEqual(['The D6', 'Wooden Spoon', 'Brimstone'])
    expect(
      titles(filterPages(pages, WikiCategory.Bosses, withQuery(''), noSave)),
    ).toEqual(['Monstro'])
    expect(
      titles(filterPages(pages, WikiCategory.Monsters, withQuery(''), noSave)),
    ).toEqual(['Gaper'])
    expect(
      filterPages(pages, WikiCategory.Trinkets, withQuery(''), noSave),
    ).toEqual([])
  })

  it('matches the title, case-insensitively, anywhere in it', () => {
    expect(
      titles(filterPages(pages, WikiCategory.Items, withQuery('d6'), noSave)),
    ).toEqual(['The D6'])
    expect(
      titles(
        filterPages(pages, WikiCategory.Items, withQuery(' SPOON '), noSave),
      ),
    ).toEqual(['Wooden Spoon'])
    expect(
      filterPages(pages, WikiCategory.Items, withQuery('monstro'), noSave),
    ).toEqual([])
  })

  // Review focus 1: with no save chosen every page's profile facet reads as "no data", so a
  // reader who picks "done" gets an empty list rather than every row.
  it('is empty when the profile state is picked and no save is chosen', () => {
    const filter = {
      ...emptyWikiListFilter(),
      picks: {
        ...emptyWikiListFilter().picks,
        [WikiFacet.Profile]: [ProfileValue.Done],
      },
    }
    expect(filterPages(pages, WikiCategory.Items, filter, noSave)).toEqual([])
  })

  it('matches the profile state a save does answer', () => {
    const done: PageProgress = { kind: 'achievement', done: true }
    const faceting = wikiFaceting(() => done)
    const filter = {
      ...emptyWikiListFilter(),
      picks: {
        ...emptyWikiListFilter().picks,
        [WikiFacet.Profile]: [ProfileValue.Done],
      },
    }
    expect(
      titles(filterPages(pages, WikiCategory.Items, filter, faceting)),
    ).toEqual(['The D6', 'Wooden Spoon', 'Brimstone'])
  })
})
