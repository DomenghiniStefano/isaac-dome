import { describe, expect, it } from 'vitest'
import { EventKey } from '@/lib/constants/eventKeys'
import {
  DropSide,
  dropSide,
  arrivalGap,
  moveIndex,
  neighbourIndex,
  tabDropAt,
} from './tabs'

describe('dropSide', () => {
  it('lands before a tab when the pointer is on its left half', () => {
    expect(dropSide(110, 100, 40)).toBe(DropSide.Before)
  })

  it('lands after a tab when the pointer is on its right half', () => {
    expect(dropSide(135, 100, 40)).toBe(DropSide.After)
  })

  it('counts the exact middle as after', () => {
    expect(dropSide(120, 100, 40)).toBe(DropSide.After)
  })
})

describe('moveIndex', () => {
  it('moves a tab right, after the target', () => {
    expect(moveIndex(0, 2, DropSide.After)).toBe(2)
  })

  it('moves a tab right, before the target', () => {
    expect(moveIndex(0, 2, DropSide.Before)).toBe(1)
  })

  it('moves a tab left, before the target', () => {
    expect(moveIndex(3, 1, DropSide.Before)).toBe(1)
  })

  it('moves a tab left, after the target', () => {
    expect(moveIndex(3, 1, DropSide.After)).toBe(2)
  })

  it('leaves a tab dropped on itself where it is', () => {
    expect(moveIndex(2, 2, DropSide.Before)).toBe(2)
    expect(moveIndex(2, 2, DropSide.After)).toBe(2)
  })
})

// Three tabs of 100px side by side, from x = 0.
const strip = [0, 100, 200].map((left) => ({
  left,
  top: 0,
  width: 100,
  height: 30,
}))

// A tab arriving from another window: the gap the marker is drawn in is where it lands.
describe('arrivalGap', () => {
  it('lands before or after the tab under the pointer, by its half', () => {
    expect(arrivalGap(strip, { x: 120, y: 10 }, 3)).toBe(1)
    expect(arrivalGap(strip, { x: 180, y: 10 }, 3)).toBe(2)
  })

  it('lands at the end when no tab is under the pointer', () => {
    expect(arrivalGap(strip, { x: 900, y: 10 }, 3)).toBe(3)
    expect(arrivalGap([], { x: 0, y: 0 }, 0)).toBe(0)
  })
})

// The arrows walk the strip one tab at a time and stop at its ends.
describe('neighbourIndex', () => {
  const ids = ['a', 'b', 'c']

  it('moves right and left from the active tab', () => {
    expect(neighbourIndex(ids, 'b', EventKey.ArrowRight)).toBe(2)
    expect(neighbourIndex(ids, 'b', EventKey.ArrowLeft)).toBe(0)
  })

  it('stays on the last tab going right and the first going left', () => {
    expect(neighbourIndex(ids, 'c', EventKey.ArrowRight)).toBe(2)
    expect(neighbourIndex(ids, 'a', EventKey.ArrowLeft)).toBe(0)
  })

  it('answers nothing for another key, or with no active tab in the strip', () => {
    expect(neighbourIndex(ids, 'b', EventKey.Enter)).toBeNull()
    expect(neighbourIndex(ids, null, EventKey.ArrowRight)).toBeNull()
  })
})

// A reorder inside the strip: beside the tab under the pointer, never onto the one being moved.
describe('tabDropAt', () => {
  it('lands on a side of the tab under the pointer', () => {
    expect(tabDropAt(strip, { x: 210, y: 10 }, 0)).toEqual({
      index: 2,
      side: DropSide.Before,
    })
    expect(tabDropAt(strip, { x: 290, y: 10 }, 0)).toEqual({
      index: 2,
      side: DropSide.After,
    })
  })

  it('is no drop over the tab being moved, or over no tab at all', () => {
    expect(tabDropAt(strip, { x: 50, y: 10 }, 0)).toBeNull()
    expect(tabDropAt(strip, { x: 900, y: 10 }, 0)).toBeNull()
  })
})
