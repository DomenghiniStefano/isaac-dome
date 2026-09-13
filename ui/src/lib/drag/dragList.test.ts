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
  // It reads travel in **any** direction, not along the list's axis. Measured on the machine
  // on 2026-09-13: with an axis-bound threshold, a tab dragged straight down out of the window
  // never began a drag at all — the strip runs along x, and the tear-off is a vertical gesture,
  // so the one movement the user makes to tear a tab off was the one movement that could not
  // start it. The axis still decides the hit test (`boxAt`); it no longer decides what a drag
  // is.
  it('starts a drag on movement in any direction', () => {
    const from = { x: 100, y: 100 }
    expect(crossedThreshold(from, { x: 100, y: 110 }, 4)).toBe(true)
    expect(crossedThreshold(from, { x: 105, y: 100 }, 4)).toBe(true)
    expect(crossedThreshold(from, { x: 103, y: 103 }, 4)).toBe(true)
  })

  it('does not start one on a hand that did not move', () => {
    const from = { x: 100, y: 100 }
    expect(crossedThreshold(from, { x: 100, y: 100 }, 4)).toBe(false)
    expect(crossedThreshold(from, { x: 102, y: 102 }, 4)).toBe(false)
    expect(crossedThreshold(from, { x: 103, y: 100 }, 4)).toBe(false)
  })

  it('counts travel in both directions, and the threshold itself is a drag', () => {
    const from = { x: 100, y: 100 }
    expect(crossedThreshold(from, { x: 97, y: 100 }, 4)).toBe(false)
    expect(crossedThreshold(from, { x: 96, y: 100 }, 4)).toBe(true)
    expect(crossedThreshold(from, { x: 100, y: 104 }, 4)).toBe(true)
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
