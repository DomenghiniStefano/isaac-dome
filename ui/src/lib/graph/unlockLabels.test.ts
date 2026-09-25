import { describe, expect, it } from 'vitest'
import { FacetId } from './unlockFacets'
import { NodeState } from './nodeState'
import { unlockBar, unlockFacetValueLabel } from './unlockLabels'

// A key is its own translation here: what matters is which key, or that there was none.
const t = (key: string, args?: Record<string, unknown>): string =>
  args ? `«${key}:${JSON.stringify(args)}»` : `«${key}»`

describe('unlockFacetValueLabel', () => {
  it('words a state, a kind and an origin', () => {
    expect(unlockFacetValueLabel(t, FacetId.State, NodeState.Now)).toBe(
      '«graph.stateName.now»',
    )
    expect(unlockFacetValueLabel(t, FacetId.Unlocks, 'boss')).toBe(
      '«graph.kinds.boss»',
    )
    expect(unlockFacetValueLabel(t, FacetId.Origin, 'none')).toBe(
      '«graph.originNone»',
    )
  })

  it('names a character by its form, which the facet stores as an id', () => {
    const characters = new Map([['14', { name: 'The Lost', tainted: true }]])
    expect(unlockFacetValueLabel(t, FacetId.Character, '14', characters)).toBe(
      '«graph.taintedName:{"name":"The Lost"}»',
    )
  })

  it('shows a value outside its set as it came', () => {
    expect(unlockFacetValueLabel(t, FacetId.State, 'maybe')).toBe('maybe')
    expect(unlockFacetValueLabel(t, FacetId.Unlocks, 'pill')).toBe('pill')
    expect(unlockFacetValueLabel(t, FacetId.Origin, 'flash')).toBe('flash')
    expect(unlockFacetValueLabel(t, FacetId.Character, '99')).toBe('99')
  })
})

describe('the state row of Unlock', () => {
  it('draws every state it orders, a square and a name each', () => {
    const { order, dot, text } = unlockBar.state
    expect(order).toEqual([
      NodeState.Done,
      NodeState.Now,
      NodeState.Blocked,
      NodeState.Partial,
    ])
    expect(order.map((s) => dot[s])).toEqual([
      'bg-state-done',
      'bg-state-now',
      'bg-state-blocked',
      'border border-dashed border-state-blocked',
    ])
    expect(order.map((s) => text[s])).toEqual([
      'graph.stateName.done',
      'graph.stateName.now',
      'graph.stateName.blocked',
      'graph.stateName.partial',
    ])
  })
})
