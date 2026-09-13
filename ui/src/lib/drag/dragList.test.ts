import { describe, expect, it } from 'vitest'
import {
  Axis,
  DragThreshold,
  boxAt,
  crossedThreshold,
  ghostOrigin,
  grabOffset,
} from './dragList'

const box = (left: number, top: number, width: number, height: number) => ({
  left,
  top,
  width,
  height,
})

describe('crossedThreshold', () => {
  it('reads only the axis the list runs along', () => {
    const from = { x: 100, y: 100 }
    // A tab strip runs along x: ten pixels down is still a click.
    expect(crossedThreshold(Axis.X, from, { x: 100, y: 110 }, 4)).toBe(false)
    expect(crossedThreshold(Axis.X, from, { x: 105, y: 100 }, 4)).toBe(true)
    // A queue runs along y, and the two swap.
    expect(crossedThreshold(Axis.Y, from, { x: 110, y: 100 }, 4)).toBe(false)
    expect(crossedThreshold(Axis.Y, from, { x: 100, y: 105 }, 4)).toBe(true)
  })

  it('counts travel in both directions, and the threshold itself is a drag', () => {
    const from = { x: 100, y: 100 }
    expect(crossedThreshold(Axis.X, from, { x: 97, y: 100 }, 4)).toBe(false)
    expect(crossedThreshold(Axis.X, from, { x: 96, y: 100 }, 4)).toBe(true)
  })

  it('is 4 pixels by default, the number both screens had chosen', () => {
    expect(DragThreshold).toBe(4)
  })
})

describe('boxAt', () => {
  const strip = [box(0, 0, 100, 30), box(100, 0, 100, 30), box(200, 0, 100, 30)]

  it('finds the box the point is inside, on the axis that matters', () => {
    expect(boxAt(strip, { x: 150, y: 15 }, Axis.X)).toBe(1)
    // Off the strip vertically is still the second tab: a strip is a line, not a grid.
    expect(boxAt(strip, { x: 150, y: 900 }, Axis.X)).toBe(1)
  })

  it('gives a box its left edge and not its right: the seam belongs to the next one', () => {
    expect(boxAt(strip, { x: 100, y: 15 }, Axis.X)).toBe(1)
    expect(boxAt(strip, { x: 99, y: 15 }, Axis.X)).toBe(0)
  })

  it('answers null past the ends', () => {
    expect(boxAt(strip, { x: -1, y: 15 }, Axis.X)).toBeNull()
    expect(boxAt(strip, { x: 300, y: 15 }, Axis.X)).toBeNull()
    expect(boxAt([], { x: 0, y: 0 }, Axis.X)).toBeNull()
  })

  it('reads top and height on the other axis', () => {
    const queue = [box(0, 0, 400, 50), box(0, 50, 400, 50)]
    expect(boxAt(queue, { x: 20, y: 70 }, Axis.Y)).toBe(1)
    expect(boxAt(queue, { x: 20, y: 100 }, Axis.Y)).toBeNull()
  })
})

describe('the ghost keeps the grab where the finger put it', () => {
  it('measures the press inside the box', () => {
    expect(grabOffset(box(100, 40, 200, 30), { x: 160, y: 55 })).toEqual({
      x: 60,
      y: 15,
    })
  })

  it('puts the box back under the pointer at that offset', () => {
    const offset = { x: 60, y: 15 }
    expect(ghostOrigin(offset, { x: 500, y: 300 })).toEqual({ x: 440, y: 285 })
  })
})
