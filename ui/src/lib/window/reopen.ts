import { clampToMonitors } from './monitorClamp'
import { oweSeed } from './seeds'
import type { StoredBox, StoredWindow } from './sessionDocument'
import { newWindowLabel, windowPort } from './windowPort'

// How far a restored window is stepped from the one before it when the document has no box for
// it — a version 1 session, or a window whose geometry could not be read. Window geometry, like
// `TearBand`: it is not a visual constant and never reaches a template.
const CascadeStep = 32

// Where a restored window with no box of its own goes: the first window's size, `step` cascade
// steps down and right of it.
export const cascadeBox = (first: StoredBox, step: number): StoredBox => ({
  left: first.left + CascadeStep * step,
  top: first.top + CascadeStep * step,
  width: first.width,
  height: first.height,
})

// One window of the session, opened where it was — or cascaded — and kept on a monitor that is
// there now. The newborn is owed its seed before it is created and the debt is waited for,
// exactly as `openWindowWith` waits.
const reopenOne = async (
  stored: StoredWindow,
  wanted: StoredBox,
  monitors: Awaited<ReturnType<typeof windowPort.monitors>>,
): Promise<void> => {
  const placed = clampToMonitors(wanted, monitors)
  const label = newWindowLabel()
  const paid = oweSeed(label, stored.tabs, stored.activeIndex)
  await windowPort.create(
    label,
    { x: placed.box.left, y: placed.box.top },
    {
      // Physical in, logical out: `create` takes a physical position and a logical size.
      x: placed.box.width / placed.scaleFactor,
      y: placed.box.height / placed.scaleFactor,
    },
  )
  await paid
}

// The windows this session had, minus the one `main` took for itself. Sequential and not
// `Promise.all`: each newborn is owed its seed before the next is created — the debt lives in
// this window's memory and a newborn that asks before it is registered opens with an empty bar.
export const reopenWindows = async (
  rest: readonly StoredWindow[],
): Promise<void> => {
  if (rest.length === 0) return
  // **Only at a launch, and a launch is one window.** `main` re-runs its whole mount whenever its
  // webview reloads — which is every save on the development server — and there it would find
  // the session it wrote a moment ago and open a second copy of every window in it. At a real
  // launch the roster is `main` alone, which is the difference between the two, and reading it
  // costs one call.
  if ((await windowPort.labels()).length > 1) return
  const monitors = await windowPort.monitors()
  const first = await windowPort.self()
  // One after the other, each a cascade step further than the last.
  const reopenFrom = async (
    windows: readonly StoredWindow[],
    step: number,
  ): Promise<void> => {
    const [next, ...others] = windows
    if (!next) return
    await reopenOne(next, next.box ?? cascadeBox(first, step), monitors)
    await reopenFrom(others, step + 1)
  }
  await reopenFrom(rest, 1)
}
