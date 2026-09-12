import { describe, expect, it } from 'vitest'
import type { Target } from '@/lib/ipc/types'
import { pageKey, parsePageKey } from './pageKey'

const pages: [Target, string][] = [
  [{ kind: 'item', id: 105 }, 'item:105'],
  [{ kind: 'trinket', id: 97 }, 'trinket:97'],
  [{ kind: 'achievement', id: 1 }, 'achievement:1'],
  [{ kind: 'challenge', number: 19 }, 'challenge:19'],
  [{ kind: 'character', id: 0 }, 'character:0'],
  [{ kind: 'entity', id: 20, variant: 0, subtype: 0 }, 'entity:20.0.0'],
]

describe('pageKey', () => {
  it.each(pages)('writes %o as %s and reads it back', (target, key) => {
    expect(pageKey(target)).toBe(key)
    expect(parsePageKey(key)).toEqual(target)
  })

  it('has no key for a target with no page', () => {
    expect(pageKey({ kind: 'stage', name: 'Basement' })).toBeNull()
    expect(pageKey({ kind: 'room', name: 'Devil Room' })).toBeNull()
    expect(pageKey({ kind: 'pickup', name: 'Chest' })).toBeNull()
    expect(pageKey({ kind: 'transformation', id: 1 })).toBeNull()
  })

  it('refuses what it never wrote', () => {
    for (const bad of [
      '',
      'item',
      'item:',
      'item:x',
      'item:1.5',
      'item:-1',
      'stage:Basement',
      'transformation:1',
      'entity:20.0',
      'entity:20.0.0.0',
      'item:1:2',
      'ITEM:1',
      'item: 1',
    ]) {
      expect(parsePageKey(bad), bad).toBeNull()
    }
  })
})
