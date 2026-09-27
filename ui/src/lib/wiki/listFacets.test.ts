import { describe, expect, it } from 'vitest'
import type { PageFacts, PageProgress, WikiPageRef } from '@/lib/ipc/types'
import { Dlc } from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'
import {
  ProfileValue,
  WikiFacet,
  emptyWikiListFilter,
  wikiFaceting,
  wikiFacetOptions,
  wikiFacetValues,
  wikiSortOrder,
} from './listFacets'

const itemFacts = (
  over: Partial<Extract<PageFacts, { kind: 'item' }>> = {},
): PageFacts => ({
  kind: 'item',
  quality: null,
  activated: false,
  recharge: null,
  shopPrice: null,
  devilPrice: null,
  tags: [],
  ...over,
})

const page = (over: Partial<WikiPageRef> = {}): WikiPageRef => ({
  target: { kind: 'item', id: 1 },
  title: 'The D6',
  iconUrl: null,
  category: WikiCategory.Items,
  dlc: [],
  facts: itemFacts(),
  ...over,
})

const noProgress = (): PageProgress | null => null

describe('wikiFacetValues', () => {
  it('reads an unrestricted page as present in every edition', () => {
    expect(
      wikiFacetValues(page({ dlc: [] }), WikiFacet.Edition, noProgress),
    ).toEqual(Object.values(Dlc))
  })

  it('reads a restricted page as present only in the editions it names', () => {
    expect(
      wikiFacetValues(
        page({ dlc: [Dlc.Repentance, Dlc.RepentancePlus] }),
        WikiFacet.Edition,
        noProgress,
      ),
    ).toEqual([Dlc.Repentance, Dlc.RepentancePlus])
  })

  it('is nothing without a save, for the profile facet', () => {
    expect(wikiFacetValues(page(), WikiFacet.Profile, noProgress)).toEqual([
      ProfileValue.NoData,
    ])
  })

  it('reads the save as done or not done', () => {
    const done: PageProgress = { kind: 'achievement', done: true }
    const notDone: PageProgress = { kind: 'achievement', done: false }
    expect(wikiFacetValues(page(), WikiFacet.Profile, () => done)).toEqual([
      ProfileValue.Done,
    ])
    expect(wikiFacetValues(page(), WikiFacet.Profile, () => notDone)).toEqual([
      ProfileValue.NotDone,
    ])
  })

  it('reads an unrated item as its own value, never guessed', () => {
    expect(
      wikiFacetValues(
        page({ facts: itemFacts({ quality: null }) }),
        WikiFacet.Quality,
        noProgress,
      ),
    ).toEqual(['unrated'])
    expect(
      wikiFacetValues(
        page({ facts: itemFacts({ quality: 3 }) }),
        WikiFacet.Quality,
        noProgress,
      ),
    ).toEqual(['3'])
  })

  it('reads nothing for a facet the page kind does not carry', () => {
    const trinket = page({
      target: { kind: 'trinket', id: 1 },
      facts: { kind: 'trinket', tags: [] },
    })
    expect(wikiFacetValues(trinket, WikiFacet.Quality, noProgress)).toEqual([])
    expect(wikiFacetValues(trinket, WikiFacet.Tainted, noProgress)).toEqual([])
  })
})

describe('wikiFacetOptions', () => {
  it('offers the tags actually present, once each, in order', () => {
    const pages = [
      page({ facts: itemFacts({ tags: ['guppy', 'shy'] }) }),
      page({ facts: itemFacts({ tags: ['guppy'] }) }),
    ]
    expect(wikiFacetOptions(pages, WikiFacet.Tag)).toEqual(['guppy', 'shy'])
  })
})

describe('wikiFaceting', () => {
  it('matches a page picked by its profile state', () => {
    const done: PageProgress = { kind: 'achievement', done: true }
    const faceting = wikiFaceting(() => done)
    const filter = {
      ...emptyWikiListFilter(),
      picks: {
        ...emptyWikiListFilter().picks,
        [WikiFacet.Profile]: [ProfileValue.Done],
      },
    }
    expect(faceting.matches(page(), filter)).toBe(true)
    expect(
      faceting.matches(page(), {
        ...filter,
        picks: { ...filter.picks, [WikiFacet.Profile]: [ProfileValue.NotDone] },
      }),
    ).toBe(false)
  })

  it('matches the title, case-insensitively', () => {
    const faceting = wikiFaceting(() => null)
    expect(
      faceting.matches(page({ title: 'The D6' }), {
        ...emptyWikiListFilter(),
        query: 'd6',
      }),
    ).toBe(true)
    expect(
      faceting.matches(page({ title: 'The D6' }), {
        ...emptyWikiListFilter(),
        query: 'spoon',
      }),
    ).toBe(false)
  })
})

describe('wikiSortOrder', () => {
  it('offers name, id and edition before the category facts', () => {
    const order = wikiSortOrder(WikiCategory.Items)
    expect(order.slice(0, 3)).toEqual(['name', 'id', 'edition'])
    expect(order).toContain('quality')
  })

  it('leaves the id out for a category whose pages have none', () => {
    expect(wikiSortOrder(WikiCategory.Versions)).not.toContain('id')
  })
})
