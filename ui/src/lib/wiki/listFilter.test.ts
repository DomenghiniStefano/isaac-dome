import { describe, expect, it } from 'vitest'
import type { PageFacts, WikiPageRef } from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'
import { filterPages } from './listFilter'

// This test only checks title/category filtering, never a fact: every kind reads as "not
// stated", the same shape a page with an empty infobox gets from the real `facts()`.
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
      return { kind: 'boss', baseHp: null, floors: '' }
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

describe('filterPages', () => {
  it("lists a category's pages by title", () => {
    expect(titles(filterPages(pages, WikiCategory.Items, ''))).toEqual([
      'Brimstone',
      'The D6',
      'Wooden Spoon',
    ])
    expect(titles(filterPages(pages, WikiCategory.Bosses, ''))).toEqual([
      'Monstro',
    ])
    expect(titles(filterPages(pages, WikiCategory.Monsters, ''))).toEqual([
      'Gaper',
    ])
    expect(filterPages(pages, WikiCategory.Trinkets, '')).toEqual([])
  })

  it('matches the title, case-insensitively, anywhere in it', () => {
    expect(titles(filterPages(pages, WikiCategory.Items, 'd6'))).toEqual([
      'The D6',
    ])
    expect(titles(filterPages(pages, WikiCategory.Items, ' SPOON '))).toEqual([
      'Wooden Spoon',
    ])
    expect(filterPages(pages, WikiCategory.Items, 'monstro')).toEqual([])
  })
})
