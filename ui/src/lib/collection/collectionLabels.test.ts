import { describe, expect, it } from 'vitest'
import {
  CollectionFacet,
  NoPool,
  QualityValue,
  collectionFaceting,
} from './collectionFacets'
import { collectionBar, collectionFacetValueLabel } from './collectionLabels'
import { ItemState } from './itemState'

const t = (key: string): string => `«${key}»`

describe('collectionFacetValueLabel', () => {
  it('words a state, a kind, an origin, and the two values that are ours', () => {
    expect(
      collectionFacetValueLabel(t, CollectionFacet.State, ItemState.Locked),
    ).toBe('«collection.state.locked»')
    expect(collectionFacetValueLabel(t, CollectionFacet.Kind, 'familiar')).toBe(
      '«graph.kinds.familiar»',
    )
    expect(collectionFacetValueLabel(t, CollectionFacet.Origin, 'none')).toBe(
      '«graph.originNone»',
    )
    expect(
      collectionFacetValueLabel(
        t,
        CollectionFacet.Quality,
        QualityValue.Unrated,
      ),
    ).toBe('«collection.qualityUnrated»')
    expect(collectionFacetValueLabel(t, CollectionFacet.Pool, NoPool)).toBe(
      '«collection.poolNone»',
    )
  })

  it('shows a number and a pool as they are', () => {
    expect(collectionFacetValueLabel(t, CollectionFacet.Quality, '4')).toBe('4')
    expect(collectionFacetValueLabel(t, CollectionFacet.Pool, 'treasure')).toBe(
      'treasure',
    )
  })

  it('shows a value outside its set as it came', () => {
    expect(collectionFacetValueLabel(t, CollectionFacet.State, 'lost')).toBe(
      'lost',
    )
    expect(collectionFacetValueLabel(t, CollectionFacet.Kind, 'pill')).toBe(
      'pill',
    )
  })
})

describe('the state row of the Collection', () => {
  it('draws every state it orders, a square and a name each', () => {
    const { order, dot, text } = collectionBar(collectionFaceting([])).state
    expect(order).toEqual([
      ItemState.InCollection,
      ItemState.Available,
      ItemState.Locked,
      ItemState.Unknown,
    ])
    expect(order.map((s) => dot[s])).toEqual([
      'bg-state-done',
      'bg-state-now',
      'bg-state-blocked',
      'hatch-unknown border border-dashed border-state-unknown',
    ])
    expect(order.map((s) => text[s])).toEqual([
      'collection.state.inCollection',
      'collection.state.available',
      'collection.state.locked',
      'collection.state.unknown',
    ])
  })
})
