import { describe, expect, it } from 'vitest'
import { poolLabel } from './pools'

// The word after `from pool` on an item line is `itempools.xml`'s pool name — measured on the
// installed game, 2026-10-04: 31 pools. The ones a player meets most are worded; the rest are
// shown as the log wrote them.
describe('poolLabel', () => {
  it.each([
    'treasure',
    'shop',
    'boss',
    'devil',
    'angel',
    'secret',
    'library',
    'curse',
    'goldenChest',
    'redChest',
    'beggar',
  ])('words %s', (pool) => {
    expect(poolLabel(pool)).toBe(`runs.pool.${pool}`)
  })

  it('leaves a real pool it does not word to be shown as written', () => {
    expect(poolLabel('craneGame')).toBeNull()
  })

  it('leaves a word it has never seen to be shown as written', () => {
    expect(poolLabel('weirdPool')).toBeNull()
  })
})
