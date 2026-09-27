import { describe, expect, it } from 'vitest'
import type { WikiPageRef } from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'
import { filterPages } from './listFilter'

const page = (
  target: WikiPageRef['target'],
  title: string,
  category: WikiPageRef['category'],
): WikiPageRef => ({
  target,
  title,
  iconUrl: null,
  category,
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
