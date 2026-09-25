import { describe, expect, it } from 'vitest'
import { ChallengeFacet } from './challengeFacets'
import { challengeBar, challengeFacetValueLabel } from './challengeLabels'

const t = (key: string): string => `«${key}»`
const names = new Map([['3', 'Azazel']])

describe('challengeFacetValueLabel', () => {
  it('words a state, and names a character from the rows', () => {
    expect(
      challengeFacetValueLabel(t, ChallengeFacet.State, 'blocked', names),
    ).toBe('«challenges.state.blocked»')
    expect(
      challengeFacetValueLabel(t, ChallengeFacet.Character, '3', names),
    ).toBe('Azazel')
  })

  it('words the two values of rewards and of blindfolded', () => {
    expect(
      challengeFacetValueLabel(t, ChallengeFacet.Rewards, 'some', names),
    ).toBe('«challenges.rewardsSome»')
    expect(
      challengeFacetValueLabel(t, ChallengeFacet.Rewards, 'none', names),
    ).toBe('«challenges.rewardsNone»')
    expect(
      challengeFacetValueLabel(t, ChallengeFacet.Blindfolded, 'yes', names),
    ).toBe('«challenges.blindfoldedYes»')
    expect(
      challengeFacetValueLabel(t, ChallengeFacet.Blindfolded, 'no', names),
    ).toBe('«challenges.blindfoldedNo»')
  })

  it('shows a state or a character outside its set as it came', () => {
    expect(
      challengeFacetValueLabel(t, ChallengeFacet.State, 'lost', names),
    ).toBe('lost')
    expect(
      challengeFacetValueLabel(t, ChallengeFacet.Character, '99', names),
    ).toBe('99')
  })

  it('shows an unknown rewards or blindfolded value as it came', () => {
    expect(
      challengeFacetValueLabel(t, ChallengeFacet.Rewards, 'many', names),
    ).toBe('many')
    expect(
      challengeFacetValueLabel(t, ChallengeFacet.Blindfolded, 'maybe', names),
    ).toBe('maybe')
  })
})

describe('the state row of the Challenges', () => {
  it('draws every state it orders, a square and a name each', () => {
    const { order, dot, text } = challengeBar.state
    expect(order).toEqual(['done', 'available', 'blocked', 'unknown'])
    expect(order.map((s) => dot[s])).toEqual([
      'bg-state-done',
      'bg-state-now',
      'bg-state-blocked',
      'hatch-unknown border border-dashed border-state-unknown',
    ])
    expect(order.map((s) => text[s])).toEqual([
      'challenges.state.done',
      'challenges.state.available',
      'challenges.state.blocked',
      'challenges.state.unknown',
    ])
  })
})
