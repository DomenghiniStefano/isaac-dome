import { describe, expect, it } from 'vitest'
import { EventKey } from '@/lib/constants/eventKeys'
import {
  DropEdge,
  StepDirection,
  dropAnchor,
  dropEdge,
  stepAnchor,
  stepDirection,
} from './queueDrop'

// The payload's queue (contracts/payload/queue.with_rows.json): 480, 55, 69.
const ids = [480, 55, 69]

describe('dropEdge', () => {
  it('is the half of the row under the pointer, the middle counting as below', () => {
    expect(dropEdge(109, 100, 20)).toBe(DropEdge.Above)
    expect(dropEdge(110, 100, 20)).toBe(DropEdge.Below)
  })
})

describe('dropAnchor', () => {
  it('names the row a drop lands under, or the top', () => {
    expect(dropAnchor(ids, 2, 0, DropEdge.Above)).toEqual({ after: null })
    expect(dropAnchor(ids, 2, 0, DropEdge.Below)).toEqual({ after: 480 })
    expect(dropAnchor(ids, 0, 1, DropEdge.Below)).toEqual({ after: 55 })
    expect(dropAnchor(ids, 0, 2, DropEdge.Below)).toEqual({ after: 69 })
  })

  it('sends nothing for a drop on the gap the row already fills', () => {
    expect(dropAnchor(ids, 1, 1, DropEdge.Above)).toBeNull()
    expect(dropAnchor(ids, 1, 1, DropEdge.Below)).toBeNull()
    expect(dropAnchor(ids, 0, 1, DropEdge.Above)).toBeNull()
    expect(dropAnchor(ids, 1, 0, DropEdge.Below)).toBeNull()
  })

  it('sends nothing for a row or a target outside the queue', () => {
    expect(dropAnchor(ids, 3, 0, DropEdge.Above)).toBeNull()
    expect(dropAnchor(ids, 0, 3, DropEdge.Above)).toBeNull()
    expect(dropAnchor(ids, 0, -1, DropEdge.Below)).toBeNull()
  })
})

describe('stepAnchor', () => {
  it('moves a row one place up or down', () => {
    expect(stepAnchor(ids, 2, StepDirection.Up)).toEqual({ after: 480 })
    expect(stepAnchor(ids, 1, StepDirection.Up)).toEqual({ after: null })
    expect(stepAnchor(ids, 0, StepDirection.Down)).toEqual({ after: 55 })
    expect(stepAnchor(ids, 1, StepDirection.Down)).toEqual({ after: 69 })
  })

  it('does nothing past either end', () => {
    expect(stepAnchor(ids, 0, StepDirection.Up)).toBeNull()
    expect(stepAnchor(ids, 2, StepDirection.Down)).toBeNull()
  })
})

// The keyboard's drag: Alt with an arrow moves the row one step, and nothing else is a move —
// a bare arrow scrolls the page, as it would anywhere else.
describe('stepDirection', () => {
  it('reads Alt with an arrow as a step up or down', () => {
    expect(stepDirection(EventKey.ArrowUp, true)).toBe(StepDirection.Up)
    expect(stepDirection(EventKey.ArrowDown, true)).toBe(StepDirection.Down)
  })

  it('reads a bare arrow, or Alt with another key, as no step', () => {
    expect(stepDirection(EventKey.ArrowUp, false)).toBeNull()
    expect(stepDirection(EventKey.ArrowLeft, true)).toBeNull()
    expect(stepDirection(EventKey.Enter, true)).toBeNull()
  })
})
