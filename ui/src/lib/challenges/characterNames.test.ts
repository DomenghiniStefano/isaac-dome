import { describe, expect, it } from 'vitest'
import type { ChallengeRow } from '@/lib/ipc/types'
import { challengeCharacterNames } from './characterNames'

const row = (over: Partial<ChallengeRow> = {}): ChallengeRow => ({
  number: 1,
  name: 'Pitch Black',
  state: { kind: 'available' },
  rewards: [],
  character: null,
  characterName: null,
  goal: null,
  blindfolded: null,
  page: null,
  ...over,
})

// The character facet stores the wiki's id; the names are on the rows (B28).
describe("the names the challenges' character facet is labelled with", () => {
  it('names a character by the id the facet stores', () => {
    const names = challengeCharacterNames([
      row({ character: { kind: 'character', id: 4 }, characterName: 'Eve' }),
    ])
    expect([...names]).toEqual([['4', 'Eve']])
  })

  // No name to read is no entry: the label then falls back to the value, never to a guess.
  it('leaves out a row with no character, or one whose name the dataset lacks', () => {
    const names = challengeCharacterNames([
      row(),
      row({ character: { kind: 'character', id: 7 }, characterName: null }),
    ])
    expect(names.size).toBe(0)
  })

  it('reads only a character reference, not another kind of page', () => {
    const names = challengeCharacterNames([
      row({ character: { kind: 'item', id: 4 }, characterName: 'The D6' }),
    ])
    expect(names.size).toBe(0)
  })
})
