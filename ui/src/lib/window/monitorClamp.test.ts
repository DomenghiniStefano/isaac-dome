import { describe, expect, it } from 'vitest'
import { clampToMonitors } from './monitorClamp'
import type { MonitorArea } from './windowPort'

// A laptop screen and a second one to its right, the second one twice as dense. The work areas
// stop short of the bottom, which is where the taskbar is.
const primary: MonitorArea = {
  left: 0,
  top: 0,
  width: 1920,
  height: 1040,
  scaleFactor: 1,
}
const second: MonitorArea = {
  left: 1920,
  top: 0,
  width: 2560,
  height: 1400,
  scaleFactor: 2,
}

const box = { left: 100, top: 50, width: 1280, height: 800 }

describe('a remembered window lands somewhere it can be reached', () => {
  it('leaves a box that is on a monitor exactly where it was', () => {
    expect(clampToMonitors(box, [primary, second]).box).toEqual(box)
  })

  it('carries the scale factor of the monitor it is on', () => {
    const onSecond = { ...box, left: 2000 }
    expect(clampToMonitors(onSecond, [primary, second]).scaleFactor).toBe(2)
  })

  // The case the spec names: the second screen is not plugged in this morning.
  it('moves a box that is on no monitor to the primary, at the size it had', () => {
    const gone = { left: 3000, top: 200, width: 900, height: 700 }
    const placed = clampToMonitors(gone, [primary])
    expect(placed.box.left).toBe(primary.left)
    expect(placed.box.top).toBe(primary.top)
    // "at the size it had": a window that comes back resized is not the window that was left.
    expect(placed.box.width).toBe(900)
    expect(placed.box.height).toBe(700)
    expect(placed.scaleFactor).toBe(1)
  })

  it('chooses the monitor it overlaps most when it straddles two', () => {
    // 100 px of it on the primary, the rest on the second.
    const straddling = { left: 1820, top: 100, width: 1000, height: 700 }
    expect(clampToMonitors(straddling, [primary, second]).scaleFactor).toBe(2)
  })

  // Work area, not screen: a box whose only overlap is the band the taskbar occupies is one
  // whose title bar cannot be grabbed.
  it('treats a box that only reaches the taskbar band as off the monitor', () => {
    const underTheBar = { left: 200, top: 1040, width: 600, height: 400 }
    const placed = clampToMonitors(underTheBar, [primary])
    expect(placed.box.top).toBe(primary.top)
  })

  // Degrade, never fail: with nothing to clamp against, the box is what it was.
  it('leaves the box alone when no monitor could be read', () => {
    expect(clampToMonitors(box, [])).toEqual({ box, scaleFactor: 1 })
  })

  it('lands on the first monitor given when none of them holds the box', () => {
    const gone = { left: -4000, top: -4000, width: 900, height: 700 }
    expect(clampToMonitors(gone, [second, primary]).box.left).toBe(second.left)
  })
})
