import { describe, expect, it } from 'vitest'
import type { Target } from '@/lib/ipc/types'
import { RouteName, WikiCategory } from '@/router/routeTable'
import { categoryOf, pageLocation } from './category'

describe('categoryOf', () => {
  it.each<[Target, WikiCategory]>([
    [{ kind: 'item', id: 1 }, WikiCategory.Items],
    [{ kind: 'trinket', id: 1 }, WikiCategory.Trinkets],
    [{ kind: 'achievement', id: 1 }, WikiCategory.Achievements],
    [{ kind: 'entity', id: 20, variant: 0, subtype: 0 }, WikiCategory.Bosses],
    [{ kind: 'challenge', number: 1 }, WikiCategory.Challenges],
    [{ kind: 'character', id: 0 }, WikiCategory.Characters],
    [{ kind: 'transformation', id: 1 }, WikiCategory.Transformations],
  ])('%o belongs to %s', (target, category) => {
    expect(categoryOf(target)).toBe(category)
  })

  it('has no category for a target with no page', () => {
    expect(categoryOf({ kind: 'stage', name: 'Basement' })).toBeNull()
    expect(categoryOf({ kind: 'room', name: 'x' })).toBeNull()
    expect(categoryOf({ kind: 'concept', name: 'x' })).toBeNull()
  })
})

describe('pageLocation', () => {
  it('is the wiki route with the category and the page key', () => {
    expect(pageLocation({ kind: 'item', id: 105 })).toEqual({
      name: RouteName.Wiki,
      query: { category: WikiCategory.Items, page: 'item:105' },
    })
  })

  it('is nothing for a target with no page', () => {
    expect(pageLocation({ kind: 'stage', name: 'Basement' })).toBeNull()
  })
})
