import { describe, expect, it } from 'vitest'
import {
  stripBandPx,
  holdsPoint,
  inStripBand,
  pastTearBand,
  toClient,
  toDesktop,
  stripUnderPoint,
  grabOffset,
  windowOrigin,
} from './tearOff'
import type { WindowBox } from './windowPort'
import spacing from '@/assets/theme/spacing.css?raw'
import { remToPx } from '@/lib/scale/rows'
import { scalePercents } from '@/lib/scale/steps'

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

// **A tab docks onto a strip, never onto a window.** This used to answer for a window's whole
// surface, and that was wrong twice over: the card that follows the cursor hides itself when a
// strip is going to show the marker instead, so it vanished whenever the cursor crossed any
// window (the owner saw it: "ogni tanto la card scompare"); and a drop on a window's content
// has no obvious place to land, so it had to invent one. Over a body, the answer is now the
// same as over the desktop — nothing — and a release there opens a window of its own.
describe('which strip a point is over', () => {
  const a = win('a', 0, 0)
  const b = win('b', 500, 300)

  it('answers null over the bare desktop', () => {
    expect(stripUnderPoint([a, b], { x: 5000, y: 5000 }, [], 100)).toBeNull()
    expect(stripUnderPoint([], { x: 10, y: 10 }, [], 100)).toBeNull()
  })

  it('answers the window whose strip holds the point', () => {
    expect(stripUnderPoint([a, b], { x: 100, y: 10 }, [], 100)).toBe('a')
    expect(stripUnderPoint([a, b], { x: 700, y: 310 }, [], 100)).toBe('b')
  })

  it('answers null over a window that is not its strip', () => {
    // Deep in a's content, and in b's too: neither is a landing.
    expect(
      stripUnderPoint([a, b], { x: 520, y: 400 }, ['a', 'b'], 100),
    ).toBeNull()
    expect(stripUnderPoint([a, b], { x: 100, y: 400 }, [], 100)).toBeNull()
  })

  it('prefers the most recently focused when two strips overlap', () => {
    // b's strip sits over a's content; only b is a strip here, so the order does not even come
    // up. Two strips on the same point is the case it does: there is no z-order API
    // (tauri#5656), so the focus order is the only thing that can decide.
    const c = win('c', 500, 300)
    expect(stripUnderPoint([b, c], { x: 700, y: 310 }, ['c', 'b'], 100)).toBe(
      'c',
    )
    expect(stripUnderPoint([b, c], { x: 700, y: 310 }, ['b', 'c'], 100)).toBe(
      'b',
    )
  })

  it('ignores a window the order knows but the list does not', () => {
    expect(stripUnderPoint([a], { x: 100, y: 10 }, ['ghost', 'a'], 100)).toBe(
      'a',
    )
  })
})

describe('the strip band', () => {
  it('is the top of the window, in its own logical pixels', () => {
    const w = win('a', 0, 0, 2)
    expect(inStripBand(w, { x: 100, y: 10 }, 100)).toBe(true)
    // At 2x the band is StripBand logical pixels, so twice as many physical ones.
    expect(inStripBand(w, { x: 100, y: stripBandPx(100) * 2 - 1 }, 100)).toBe(
      true,
    )
    expect(inStripBand(w, { x: 100, y: stripBandPx(100) * 2 + 1 }, 100)).toBe(
      false,
    )
  })

  it('is not a band across the whole desktop', () => {
    expect(inStripBand(win('a', 0, 0), { x: 4000, y: 10 }, 100)).toBe(false)
  })

  it('starts at the window, not at the screen', () => {
    const w = win('a', 500, 300)
    expect(inStripBand(w, { x: 600, y: 310 }, 100)).toBe(true)
    expect(inStripBand(w, { x: 600, y: 10 }, 100)).toBe(false)
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

// Card #80, item 08: the band follows the interface's scale. It was 40 logical pixels at any
// scale while the title bar holding the strip is a rem token, so at 150% the strip was 45px
// and at 200% 60px against a 40px band: a drop on the lower part of the strip opened a window.
describe('the strip band at every scale', () => {
  const titlebarRem = Number(
    spacing.match(/--spacing-titlebar:\s*([\d.]+)rem;/)?.[1],
  )

  it('reads the title bar token it has to cover', () => {
    expect(titlebarRem).toBeGreaterThan(0)
  })

  it('covers the whole title bar at every step of the scale', () => {
    for (const percent of scalePercents) {
      expect(stripBandPx(percent)).toBeGreaterThanOrEqual(
        remToPx(titlebarRem, percent),
      )
    }
  })

  it('is the 40px it always was at 100%', () => {
    expect(stripBandPx(100)).toBe(40)
  })

  it('catches a drop on the lower half of the strip at 200%', () => {
    const w = win('a', 0, 0)
    // 55px down: inside a 60px title bar, outside the old 40px band.
    expect(inStripBand(w, { x: 100, y: 55 }, 200)).toBe(true)
  })
})

describe('where a torn-off window opens', () => {
  it('is offset by the grab inside the tab and the strip inside the window', () => {
    const ghost = { left: 140, top: 6, width: 160, height: 30 }
    const strip = { left: 72, top: 4, width: 800, height: 36 }
    expect(grabOffset({ x: 150, y: 20 }, ghost, strip)).toEqual({
      x: 82,
      y: 18,
    })
  })

  it('puts the tab under the cursor, the offset scaled to desktop pixels', () => {
    expect(windowOrigin({ x: 1000, y: 500 }, { x: 82, y: 18 }, 1.5)).toEqual({
      x: 877,
      y: 473,
    })
  })

  it('lands on whole pixels', () => {
    expect(windowOrigin({ x: 10, y: 10 }, { x: 1, y: 1 }, 1.25)).toEqual({
      x: 9,
      y: 9,
    })
  })
})
