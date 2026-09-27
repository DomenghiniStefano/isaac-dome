import { describe, expect, it } from 'vitest'
import type { PageFacts, WikiPageRef } from '@/lib/ipc/types'
import { Dlc } from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'
import { SortDirection, sortPages } from './listSort'

const itemFacts = (quality: number | null): PageFacts => ({
  kind: 'item',
  quality,
  activated: false,
  recharge: null,
  shopPrice: null,
  devilPrice: null,
  tags: [],
})

const item = (
  id: number,
  title: string,
  quality: number | null,
  dlc: Dlc[] = [],
): WikiPageRef => ({
  target: { kind: 'item', id },
  title,
  iconUrl: null,
  category: WikiCategory.Items,
  dlc,
  facts: itemFacts(quality),
})

const titles = (pages: WikiPageRef[]) => pages.map((p) => p.title)

describe('sortPages', () => {
  it('sorts by name', () => {
    const pages = [item(1, 'Brimstone', 3), item(2, 'Anarchist Cookbook', 2)]
    expect(
      titles(
        sortPages(pages, WikiCategory.Items, {
          key: 'name',
          direction: SortDirection.Asc,
        }),
      ),
    ).toEqual(['Anarchist Cookbook', 'Brimstone'])
  })

  it('sorts by id, numerically and not lexically', () => {
    const pages = [item(105, 'The D6', null), item(9, 'Nine Item', null)]
    expect(
      titles(
        sortPages(pages, WikiCategory.Items, {
          key: 'id',
          direction: SortDirection.Asc,
        }),
      ),
    ).toEqual(['Nine Item', 'The D6'])
  })

  it('sorts by edition, unrestricted pages first', () => {
    const pages = [
      item(1, 'Repentance-only', 0, [Dlc.Repentance]),
      item(2, 'Always been there', 0, []),
    ]
    expect(
      titles(
        sortPages(pages, WikiCategory.Items, {
          key: 'edition',
          direction: SortDirection.Asc,
        }),
      ),
    ).toEqual(['Always been there', 'Repentance-only'])
  })

  // Review focus 5: missing values last, in both directions.
  it('puts a page missing the sorted fact last, ascending', () => {
    const pages = [
      item(1, 'Unrated', null),
      item(2, 'High quality', 4),
      item(3, 'Low quality', 0),
    ]
    expect(
      titles(
        sortPages(pages, WikiCategory.Items, {
          key: 'quality',
          direction: SortDirection.Asc,
        }),
      ),
    ).toEqual(['Low quality', 'High quality', 'Unrated'])
  })

  it('puts a page missing the sorted fact last, descending too', () => {
    const pages = [
      item(1, 'Unrated', null),
      item(2, 'High quality', 4),
      item(3, 'Low quality', 0),
    ]
    expect(
      titles(
        sortPages(pages, WikiCategory.Items, {
          key: 'quality',
          direction: SortDirection.Desc,
        }),
      ),
    ).toEqual(['High quality', 'Low quality', 'Unrated'])
  })

  it('reverses on a direction change without moving the missing values', () => {
    const pages = [
      item(1, 'Unrated', null),
      item(2, 'High quality', 4),
      item(3, 'Low quality', 0),
    ]
    const desc = sortPages(pages, WikiCategory.Items, {
      key: 'quality',
      direction: SortDirection.Desc,
    })
    expect(titles(desc).at(-1)).toBe('Unrated')
  })
})
