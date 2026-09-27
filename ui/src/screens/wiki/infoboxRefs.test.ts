import { describe, expect, it } from 'vitest'
import type { Target } from '@/lib/ipc/types'
import { refOf, refsOf, titleOrKey } from './infoboxRefs'

// B40: a transformation's `contributors` are items and trinkets, in page order, that link
// like any other `Target`. The same resolution `WikiInfobox.vue` already did for one
// target (`unlockedBy`, `unlocks`) generalized to a list, so both are tested once here.
describe('refsOf', () => {
  const sarDine: Target = { kind: 'item', id: 105 }
  const swallowedPenny: Target = { kind: 'trinket', id: 1 }

  it('keeps the page order and resolves each label from the index', () => {
    const titleOf = (key: string) =>
      ({ 'item:105': 'The D6', 'trinket:1': 'Swallowed Penny' })[key] ?? null

    expect(refsOf([sarDine, swallowedPenny], titleOf)).toEqual([
      { kind: 'ref', target: sarDine, label: 'The D6' },
      { kind: 'ref', target: swallowedPenny, label: 'Swallowed Penny' },
    ])
  })

  it('falls back to the page key while the index does not know the title yet', () => {
    expect(refsOf([sarDine], () => null)).toEqual([
      { kind: 'ref', target: sarDine, label: 'item:105' },
    ])
  })

  it('falls back to an empty label for a target with no key at all', () => {
    const concept: Target = { kind: 'concept', name: 'Chest' }
    expect(refsOf([concept], () => null)).toEqual([
      { kind: 'ref', target: concept, label: '' },
    ])
  })

  it('answers an empty list for an empty list', () => {
    expect(refsOf([], () => null)).toEqual([])
  })
})

// A `Target` field (`unlockedBy`, `unlocks`) is a one-reference inline, or nothing at all.
describe('refOf', () => {
  it('draws a target as the one reference it is', () => {
    const target: Target = { kind: 'achievement', id: 3 }
    expect(refOf(target, () => 'Maggy')).toEqual([
      { kind: 'ref', target, label: 'Maggy' },
    ])
  })

  it('draws no field as no reference', () => {
    expect(refOf(null, () => 'Maggy')).toEqual([])
  })
})

// `refsOf` and `InfoboxRefRow` (design decision 9) share this one resolution, so it is
// tested once here rather than once per caller.
describe('titleOrKey', () => {
  const sarDine: Target = { kind: 'item', id: 105 }

  it('prefers the index title when it knows one', () => {
    expect(titleOrKey(sarDine, () => 'The D6')).toBe('The D6')
  })

  it('falls back to the page key while the index does not know the title yet', () => {
    expect(titleOrKey(sarDine, () => null)).toBe('item:105')
  })

  it('falls back to an empty string for a target with no key at all', () => {
    const concept: Target = { kind: 'concept', name: 'Chest' }
    expect(titleOrKey(concept, () => null)).toBe('')
  })
})
