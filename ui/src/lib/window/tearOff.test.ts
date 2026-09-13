import { describe, expect, it } from 'vitest'
import {
  StripBand,
  holdsPoint,
  inStripBand,
  pastTearBand,
  toClient,
  toDesktop,
  windowUnderPoint,
} from './tearOff'
import type { WindowBox } from './windowPort'

const win = (
  label: string,
  left: number,
  top: number,
  scaleFactor = 1,
): WindowBox => ({ label, left, top, width: 1000, height: 800, scaleFactor })

describe('desktop pixels and client pixels are not the same pixels', () => {
  it('subtracts the window and divides by its scale factor', () => {
    // A window at 100,50 on a 150% screen: a cursor 300 physical px to its right is 200
    // logical px into the page, not 300.
    expect(toClient({ x: 400, y: 50 }, win('a', 100, 50, 1.5))).toEqual({
      x: 200,
      y: 0,
    })
  })

  it('goes back the way it came', () => {
    const w = win('a', 100, 50, 1.5)
    const desktop = { x: 460, y: 200 }
    expect(toDesktop(toClient(desktop, w), w)).toEqual(desktop)
  })

  it('a second monitor at another factor does not borrow the first one', () => {
    const second = win('b', 1920, 0, 2)
    expect(toClient({ x: 2020, y: 100 }, second)).toEqual({ x: 50, y: 50 })
  })
})

describe('which window holds the point', () => {
  const a = win('a', 0, 0)
  const b = win('b', 500, 300)

  it('answers null over the bare desktop', () => {
    expect(windowUnderPoint([a, b], { x: 5000, y: 5000 }, [])).toBeNull()
    expect(windowUnderPoint([], { x: 10, y: 10 }, [])).toBeNull()
  })

  it('answers the only window holding the point', () => {
    expect(windowUnderPoint([a, b], { x: 100, y: 100 }, [])).toBe('a')
    expect(windowUnderPoint([a, b], { x: 1400, y: 1000 }, [])).toBe('b')
  })

  it('prefers the window whose strip band holds the point, over one holding it in its body', () => {
    // 520,310 is inside both: deep in a's body, and on b's strip. The focus order says `a`,
    // and the strip still wins — a drop on a strip is a more deliberate thing than a drop
    // on a window.
    expect(windowUnderPoint([a, b], { x: 520, y: 310 }, ['a'])).toBe('b')
  })

  it('falls back to the most recently focused when both hold it the same way', () => {
    // 520,400 is in both bodies. There is no z-order API (tauri#5656), so the focus order
    // is the only thing that can decide.
    expect(windowUnderPoint([a, b], { x: 520, y: 400 }, ['b', 'a'])).toBe('b')
    expect(windowUnderPoint([a, b], { x: 520, y: 400 }, ['a', 'b'])).toBe('a')
  })

  it('with no focus order at all still answers a window, not null', () => {
    // A drop that lands somewhere beats a drop that vanishes.
    expect(windowUnderPoint([a, b], { x: 520, y: 400 }, [])).not.toBeNull()
  })

  it('ignores a window the order knows but the list does not', () => {
    expect(windowUnderPoint([a], { x: 100, y: 100 }, ['ghost', 'a'])).toBe('a')
  })
})

describe('the strip band', () => {
  it('is the top of the window, in its own logical pixels', () => {
    const w = win('a', 0, 0, 2)
    expect(inStripBand(w, { x: 100, y: 10 })).toBe(true)
    // At 2x the band is StripBand logical pixels, so twice as many physical ones.
    expect(inStripBand(w, { x: 100, y: StripBand * 2 - 1 })).toBe(true)
    expect(inStripBand(w, { x: 100, y: StripBand * 2 + 1 })).toBe(false)
  })

  it('is not a band across the whole desktop', () => {
    expect(inStripBand(win('a', 0, 0), { x: 4000, y: 10 })).toBe(false)
  })

  it('starts at the window, not at the screen', () => {
    const w = win('a', 500, 300)
    expect(inStripBand(w, { x: 600, y: 310 })).toBe(true)
    expect(inStripBand(w, { x: 600, y: 10 })).toBe(false)
  })
})

describe('holdsPoint', () => {
  it('holds its left and top edges, and not its right and bottom', () => {
    const w = win('a', 0, 0)
    expect(holdsPoint(w, { x: 0, y: 0 })).toBe(true)
    expect(holdsPoint(w, { x: 999, y: 799 })).toBe(true)
    expect(holdsPoint(w, { x: 1000, y: 400 })).toBe(false)
    expect(holdsPoint(w, { x: 400, y: 800 })).toBe(false)
  })
})

describe('pastTearBand', () => {
  const strip = { left: 0, top: 0, width: 800, height: 36 }

  it('holds while the pointer stays near the strip', () => {
    expect(pastTearBand({ x: 400, y: 20 }, strip)).toBe(false)
    expect(pastTearBand({ x: 400, y: 50 }, strip)).toBe(false)
  })

  it('lets go below the band, and above it', () => {
    expect(pastTearBand({ x: 400, y: 200 }, strip)).toBe(true)
    expect(pastTearBand({ x: 400, y: -40 }, strip)).toBe(true)
  })

  it('does not tear off sideways: a strip is a line you slide along', () => {
    expect(pastTearBand({ x: -600, y: 20 }, strip)).toBe(false)
    expect(pastTearBand({ x: 4000, y: 20 }, strip)).toBe(false)
  })
})
