import { describe, expect, it } from 'vitest'
import type { WikiPageRef } from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'
import { filterPages } from './listFilter'

const page = (target: WikiPageRef['target'], title: string): WikiPageRef => ({
  target,
  title,
  iconUrl: null,
})
const pages: WikiPageRef[] = [
  page({ kind: 'item', id: 105 }, 'The D6'),
  page({ kind: 'item', id: 27 }, 'Wooden Spoon'),
  page({ kind: 'entity', id: 20, variant: 0, subtype: 0 }, 'Monstro'),
  page({ kind: 'item', id: 2 }, 'Brimstone'),
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
