import { maxBy } from 'lodash-es'
import type { StoredBox } from './sessionDocument'
import type { MonitorArea } from './windowPort'

// Where a remembered window actually opens, and at what density. A window remembered on a second
// screen that is no longer plugged in must not open where nobody can reach it — and "reach" is
// about the **work area**, not the screen: a window under the taskbar has a title bar that cannot
// be grabbed.
//
// Pure, so the whole of it is a Vitest test and none of it is a thing you find out about by
// plugging a monitor in.
export interface Placed {
  box: StoredBox
  // The factor of the monitor the window landed on. The caller divides the size by it, because
  // `windowPort.create` takes a **physical** position and a **logical** size — the same
  // asymmetry `windowSize()` carries a comment about, and getting it backwards opens a window
  // twice the size it had.
  scaleFactor: number
}

const overlap = (box: StoredBox, m: MonitorArea): number => {
  const x =
    Math.min(box.left + box.width, m.left + m.width) -
    Math.max(box.left, m.left)
  const y =
    Math.min(box.top + box.height, m.top + m.height) - Math.max(box.top, m.top)
  return x <= 0 || y <= 0 ? 0 : x * y
}

export const clampToMonitors = (
  box: StoredBox,
  monitors: readonly MonitorArea[],
): Placed => {
  // Nothing to clamp against. Degrade, never fail: the window opens where it was, which is where
  // it worked the last time anybody looked.
  const first = monitors[0]
  if (!first) return { box, scaleFactor: 1 }
  // The one it overlaps most, and the earliest of them on a tie — so a box touching two equally
  // lands on the primary, which is the first the port hands over.
  const best = maxBy(monitors, (m) => overlap(box, m)) ?? first
  if (overlap(box, best) > 0) return { box, scaleFactor: best.scaleFactor }
  // On no monitor at all: the primary's work-area corner, **at the size it had**. A window that
  // comes back resized is not the window that was left, and one larger than the screen is still
  // reachable by the title bar the corner puts on screen.
  return {
    box: { ...box, left: first.left, top: first.top },
    scaleFactor: first.scaleFactor,
  }
}
