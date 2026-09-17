import { describe, expect, it } from 'vitest'
import type { ChallengeRow } from '@/lib/ipc/types'
import { ChallengeFacet, challengeFaceting } from './challengeFacets'

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

const filter = (
  over: Partial<ReturnType<typeof challengeFaceting.empty>> = {},
) => ({
  ...challengeFaceting.empty(),
  ...over,
})

describe('the challenge facets', () => {
  it('reads a state as its tag, so a blocked row is one value and not one per gate', () => {
    const rows = [
      row({ number: 1, state: { kind: 'done' } }),
      row({ number: 2, state: { kind: 'blocked', missing: [1, 2] } }),
    ]
    expect(challengeFaceting.options(rows, ChallengeFacet.State)).toContain(
      'blocked',
    )
    expect(
      challengeFaceting
        .counts(rows, filter(), ChallengeFacet.State)
        .get('blocked'),
    ).toBe(1)
  })

  // A row the wiki has nothing for contributes **no value**: `blindfolded: null` is "we have no
  // page", and offering it as "not blindfolded" answers a question nobody could ask.
  it('a row with no page answers the blindfolded facet with nothing', () => {
    expect(
      challengeFaceting.options(
        [row({ blindfolded: null })],
        ChallengeFacet.Blindfolded,
      ),
    ).toEqual([])
  })

  it('a row the wiki knows is offered under yes or no', () => {
    const rows = [
      row({ blindfolded: true }),
      row({ number: 2, blindfolded: false }),
    ]
    expect(challengeFaceting.options(rows, ChallengeFacet.Blindfolded)).toEqual(
      ['yes', 'no'],
    )
  })

  // The character is a target, and its value is the id: two challenges played as the same
  // character are one value, not two.
  it('groups by the character the wiki names, and skips the ones it does not', () => {
    const rows = [
      row({ character: { kind: 'character', id: 3 }, characterName: 'Judas' }),
      row({
        number: 2,
        character: { kind: 'character', id: 3 },
        characterName: 'Judas',
      }),
      row({ number: 3, character: null }),
    ]
    expect(challengeFaceting.options(rows, ChallengeFacet.Character)).toEqual([
      '3',
    ])
    expect(
      challengeFaceting
        .counts(rows, filter(), ChallengeFacet.Character)
        .get('3'),
    ).toBe(2)
  })

  it('the search reads the name and the number', () => {
    const r = row({ number: 33, name: 'Onan’s Streak' })
    expect(challengeFaceting.matches(r, filter({ query: '33' }))).toBe(true)
    expect(challengeFaceting.matches(r, filter({ query: 'onan' }))).toBe(true)
    expect(challengeFaceting.matches(r, filter({ query: 'mom' }))).toBe(false)
  })

  it('offers what a challenge unlocks by the achievement it rewards', () => {
    const rows = [
      row({
        rewards: [
          {
            achievement: 122,
            text: 'You unlocked "The Candle"',
            iconUrl: null,
            page: null,
            done: false,
          },
        ],
      }),
      row({ number: 2 }),
    ]
    expect(challengeFaceting.options(rows, ChallengeFacet.Rewards)).toEqual([
      'some',
      'none',
    ])
    expect(
      challengeFaceting
        .counts(rows, filter(), ChallengeFacet.Rewards)
        .get('none'),
    ).toBe(1)
  })
})
