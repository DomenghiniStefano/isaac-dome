# One drag everywhere, and a tab that tears off into a window: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** One composable owns every list drag in the app and lifts what you grab (B31); a tab
dragged past the strip's edge tears off into a window of its own and docks back into another
window's strip (B15).

**Architecture:** The gesture splits in three — pure decisions in `ui/src/lib/drag/`, the DOM
choreography in `useDragList`, the lifted copy in `DragGhost.vue` — and the tab strip and the
queue become its two callers, keeping their own pure drop semantics. Tearing off is that same
composable continued past the strip: past a 24 px band the DOM ghost gives way to a light
preview window that follows the cursor, and the release either docks the tab into the window
under the point or opens a new one there. Everything that talks to windows lives under
`ui/src/lib/window/`, with a fake for the development server; everything that decides is a pure
function with a Vitest test. Rust contributes wiring only: a focus order, and events that carry
no payload and mean "read again".

**Tech Stack:** Vue 3 + TypeScript, Vite 7, Tailwind v4, Vitest, Pinia, Tauri 2
(`@tauri-apps/api`: `window`, `webviewWindow`, `event`), Rust (`crates/app`, wiring only).

**Spec:** `docs/superpowers/specs/2026-09-13-drag-and-windows-design.md`

## Global Constraints

- **Branch:** `feature/drag-and-windows`, cut from `develop`, in the worktree
  `C:/Projects/isaac-dome-drag`. The main checkout stays on `feature/wiki-infobox`: never
  `cd` out of the worktree, and never `git stash` (the stack is shared with the other
  worktrees).
- **Commits:** Conventional Commits, `type(scope): subject`, English, atomic. Scope is the
  package: `ui`, `app`. **Never** a `Co-Authored-By` trailer or any reference to Claude.
- **Stage by explicit path.** `git add -A` is forbidden on this repo: other sessions edit
  `docs/superpowers/` in parallel.
- **No `<style>` in SFCs.** The ghost's position and size are **CSS variables bound by the
  template** (`:style="{ '--drag-left': … }"`), which convention 1 allows explicitly; inline
  pixel properties are not.
- **No hardcoded visual constants**: every value a token in `@theme`. **There is no shadow
  token and there must not be one** — `ui/src/assets/theme/shadow.css` sets `--shadow-*:
  initial` and says the skin is flat on purpose. The ghost reads as lifted through
  `border-2 border-primary bg-sheet` and `z-50`, the layer dialogs and popovers already use.
- **No string unions in TypeScript:** `const X = { … } as const`. The only exception is a
  tagged union's tag.
- **No `invoke()` in components**, and no `@tauri-apps/api/*` outside `ui/src/lib/window/` —
  Task 11 widens the scanner so the two new namespaces are actually covered.
- **Exhaustiveness is mandatory:** no `_ =>` arm on a closed enum; in TypeScript, `assertNever`.
- **`crates/app` is wiring and is not tested.** Anything worth checking is a pure function in
  `ui/src/lib/` or a pure crate.
- **Degrade, never fail:** a lost pointer, a target window closed mid-drag, a seed that never
  arrives — each ends with the tab somewhere, never lost, and never with a window whose bar is
  empty.
- **Before declaring anything done:** `pnpm check` (`scripts/check`) green, and read its skip
  lines — `cargo test` hides them, which is why the script runs `--nocapture`.

## How this plan is ordered

Four phases, each landable on its own:

| phase | tasks | what is true at the end |
|---|---|---|
| 0 — the measurement | 0–1 | we know whether WebView2 speaks outside the window |
| 1 — B31 | 2–5 | the queue and the strip drag through one piece of code, and what you grab lifts |
| 2 — the structure | 6–11 | a second window exists, holds tabs, takes a tab, and every window agrees on the profile |
| 3 — the gesture | 12–16 | you tear a tab off with the mouse, and drop it back |
| 4 — the proof | 17–18 | it was done on the machine, and the documents say so |

Phase 1 does not depend on phase 0's answer. Only Task 13 does.

---

## Phase 0 — the measurement

### Task 0: The baseline

**Files:**
- Create: `docs/superpowers/reports/2026-09-13-drag-and-windows-report.md`

- [ ] **Step 1: Confirm where you are**

Run: `git rev-parse --abbrev-ref HEAD && git status -s`
Expected: `feature/drag-and-windows`, and a clean tree.

- [ ] **Step 2: Confirm the suite is green before touching anything**

Run: `pnpm check`
Expected: green. If it is not, stop — this plan must not start on a red suite. Read the
`skip:` lines it prints: `samples/packed` is a junction to the installed game, and without it
dozens of `unpack`, `catalog` and `ipc` tests skip silently.

- [ ] **Step 3: Seed the report**

```markdown
# One drag everywhere, and a tab that tears off into a window: report

**Plan:** `docs/superpowers/plans/archive/2026-09-13-drag-and-windows.md`
**Spec:** `docs/superpowers/specs/2026-09-13-drag-and-windows-design.md`
**Branch:** `feature/drag-and-windows`

## Task 1 — the spike

(filled in by Task 1, whatever the answer is: a negative result is a result)

## Verification on the machine

(filled in by Task 17)
```

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/reports/2026-09-13-drag-and-windows-report.md
git commit -m "docs: open the report for the drag and windows sub-project"
```

---

### Task 1: The spike — does WebView2 still speak outside the window?

This task **keeps no code**. Its output is three answers written into the report, and a decision
about one module. Anything built is throwaway and is reverted at the end of the task.

**Files:**
- Modify (temporarily, reverted in step 6): `ui/src/components/shell/TabStrip.vue`
- Modify: `docs/superpowers/reports/2026-09-13-drag-and-windows-report.md`

**Interfaces:**
- Produces: the answer that Task 13 implements — `PointerSource` is the DOM one or the Rust one.

- [ ] **Step 1: Put a throwaway probe in the tab strip**

In `TabStrip.vue`, inside `onPointerMove`, right after the capture is taken, add:

```ts
    // THROWAWAY PROBE — reverted at the end of Task 1
    const probe = (tag: string) => (ev: PointerEvent) =>
      console.log(tag, ev.clientX, ev.clientY, ev.buttons, Date.now())
    window.addEventListener('pointermove', probe('move'))
    window.addEventListener('pointerup', probe('up'))
```

- [ ] **Step 2: Run the real app, not the development server**

Run: `pnpm dev`
Expected: the Tauri window opens with its own title bar. **This measurement cannot be made in a
browser tab**: the whole question is what the WebView2 host does with the cursor outside the
window, and the development server has no window.

- [ ] **Step 3: Measure question 1 — do the events keep arriving?**

Open the devtools console. Press on a tab, drag the cursor **outside the window's bounds**, keep
dragging over the desktop for a second, then release **outside**.

Write down, verbatim, into the report:
- do `move` lines keep printing while the cursor is outside? Do their coordinates go negative or
  past the window's width, or do they clamp at the edge?
- does a single `up` line print at the release, with the cursor outside?
- repeat it over **another window** (any application) covering part of the desktop: same answer?

- [ ] **Step 4: Measure question 2 — what does a window cost?**

In the same console:

```js
const t = performance.now()
const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow')
const w = new WebviewWindow('probe-1', { url: 'index.html', width: 600, height: 400 })
await w.once('tauri://created', () => console.log('created', performance.now() - t))
```

Write down the milliseconds from the call to `tauri://created`, and — by eye, or by a
`console.log` at the top of the new window's `main.ts` — roughly how long until it has painted
something. Close it with `await w.close()`.

- [ ] **Step 5: Measure question 3 — can a window appear without taking the focus?**

```js
const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow')
const p = new WebviewWindow('probe-2', {
  url: 'index.html', width: 220, height: 44, decorations: false,
  alwaysOnTop: true, skipTaskbar: true, shadow: false, resizable: false, focus: false,
})
```

While it is open, press a key: does the **origin** window still receive it? Then, with the probe
window showing, start a tab drag in the origin window and check the probe from step 1 still
prints. Close it.

- [ ] **Step 6: Revert the probe**

Run: `git checkout -- ui/src/components/shell/TabStrip.vue`
Expected: `git status -s` shows only the report as modified.

- [ ] **Step 7: Write the decision into the report**

Fill the "Task 1 — the spike" section with the three answers **and the decision they force**:

- Question 1 answered **yes** ⇒ Task 13 implements `PointerSource` with DOM events, plus
  `cursorPosition()` only for the desktop-coordinate conversion. No Rust, no new dependency.
- Question 1 answered **no** (events stop at the edge, or the `up` never arrives) ⇒ Task 13
  implements `PointerSource` with a Rust source: a thread in `crates/app` polling `GetCursorPos`
  and `GetAsyncKeyState(VK_LBUTTON)` behind `#[cfg(windows)]`, emitting `drag-moved` and
  `drag-released`, and the `windows` crate enters `crates/app/Cargo.toml`.
- Question 2 ⇒ if a window costs more than ~150 ms to first paint, the preview page of Task 14
  earns its keep; if it is under that, note it — the owner chose the light preview anyway, and
  the number belongs in the report either way.
- Question 3 answered **no** (the probe steals the focus and the drag dies) ⇒ Task 14's preview
  window cannot be shown during the drag; fall back to showing it only once the pointer is over
  no window, and say so in the report.

Write what you actually saw, including "it did not behave as the documentation says". A negative
result is a result; silence is not.

- [ ] **Step 8: Commit**

```bash
git add docs/superpowers/reports/2026-09-13-drag-and-windows-report.md
git commit -m "docs(ui): measure what the webview does with the cursor outside the window"
```

---

## Phase 1 — B31: one drag, and what you grab lifts

### Task 2: The pure part of a list drag

**Files:**
- Create: `ui/src/lib/drag/dragList.ts`
- Test: `ui/src/lib/drag/dragList.test.ts`

**Interfaces:**
- Produces: `Axis`, `Point`, `Box`, `GhostBox`, `crossedThreshold`, `boxAt`, `grabOffset`,
  `ghostOrigin`, `DragThreshold`. Tasks 3, 12, 15 consume them.

- [ ] **Step 1: Write the failing test**

`ui/src/lib/drag/dragList.test.ts`:

```ts
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

  it('counts travel in both directions, and the threshold itself is not a drag', () => {
    const from = { x: 100, y: 100 }
    expect(crossedThreshold(Axis.X, from, { x: 96, y: 100 }, 4)).toBe(false)
    expect(crossedThreshold(Axis.X, from, { x: 95, y: 100 }, 4)).toBe(true)
  })

  it('is 4 pixels by default, the number both screens used', () => {
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
```

- [ ] **Step 2: Run it and watch it fail**

Run: `pnpm --filter ui test -- src/lib/drag/dragList.test.ts`
Expected: FAIL — `Failed to resolve import "./dragList"`.

- [ ] **Step 3: Write the module**

`ui/src/lib/drag/dragList.ts`:

```ts
import { assertNever } from '@/lib/assertNever'

// Which way the list runs. A strip of tabs runs along x, a queue of rows along y, and the
// gesture reads travel and hit tests on that axis only: dragging a tab downwards is not a
// reorder, it is the tear-off (docs/superpowers/specs/2026-09-13-drag-and-windows-design.md §3).
export const Axis = { X: 'x', Y: 'y' } as const
export type Axis = (typeof Axis)[keyof typeof Axis]

export interface Point {
  x: number
  y: number
}

// A rectangle as the gesture needs it: the DOMRect of an item, snapshotted once when the
// press becomes a drag. Nothing moves until the release, so reading it per pointermove would
// be waste — and a layout read per item per frame is the waste that shows.
export interface Box {
  left: number
  top: number
  width: number
  height: number
}

// Where the lifted copy is drawn, in client pixels.
export type GhostBox = Box

// Pixels the pointer travels before a press becomes a drag: below it, a click. Both screens
// had chosen 4 independently.
export const DragThreshold = 4

const travel = (axis: Axis, from: Point, to: Point): number => {
  switch (axis) {
    case Axis.X:
      return Math.abs(to.x - from.x)
    case Axis.Y:
      return Math.abs(to.y - from.y)
    default:
      return assertNever(axis)
  }
}

export const crossedThreshold = (
  axis: Axis,
  from: Point,
  to: Point,
  threshold: number,
): boolean => travel(axis, from, to) >= threshold

const holds = (box: Box, p: Point, axis: Axis): boolean => {
  switch (axis) {
    case Axis.X:
      return p.x >= box.left && p.x < box.left + box.width
    case Axis.Y:
      return p.y >= box.top && p.y < box.top + box.height
    default:
      return assertNever(axis)
  }
}

// Which snapshotted box the point falls in, along the list's axis only. Null between the
// items or past the ends, which is a real answer: a drop there means something else.
export const boxAt = (boxes: Box[], p: Point, axis: Axis): number | null => {
  for (const [index, box] of boxes.entries())
    if (holds(box, p, axis)) return index
  return null
}

// Where inside the grabbed item the press landed, and the top-left that keeps it there while
// the pointer moves: a lifted row that jumps so its corner meets the cursor reads as a
// different row.
export const grabOffset = (box: Box, press: Point): Point => ({
  x: press.x - box.left,
  y: press.y - box.top,
})

export const ghostOrigin = (offset: Point, p: Point): Point => ({
  x: p.x - offset.x,
  y: p.y - offset.y,
})
```

- [ ] **Step 4: Run the test and watch it pass**

Run: `pnpm --filter ui test -- src/lib/drag/dragList.test.ts`
Expected: PASS, 8 tests.

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/drag/dragList.ts ui/src/lib/drag/dragList.test.ts
git commit -m "feat(ui): the pure part of a list drag, in one module"
```

---

### Task 3: The composable and the lifted copy

**Files:**
- Create: `ui/src/composables/useDragList.ts`
- Create: `ui/src/components/ui/drag/DragGhost.vue`
- Create: `ui/src/components/ui/drag/index.ts`
- Create: `ui/src/kit/sections/app/DragListSection.vue`
- Modify: `ui/src/kit/KitPage.vue`
- Modify: `ui/src/lib/constants/eventKeys.ts`

**Interfaces:**
- Consumes: Task 2's `Axis`, `Box`, `GhostBox`, `Point`, `DragThreshold`, `boxAt`,
  `crossedThreshold`, `grabOffset`, `ghostOrigin`.
- Produces: `useDragList(options): DragList<D>` and `<DragGhost :box="…">`. Tasks 4, 5 and 15
  consume them.

- [ ] **Step 1: Add `Escape` to the event keys**

`ui/src/lib/constants/eventKeys.ts`, inside `EventKey`:

```ts
  Escape: 'Escape',
```

- [ ] **Step 2: Write the composable**

`ui/src/composables/useDragList.ts`:

```ts
import { onBeforeUnmount, ref, shallowRef } from 'vue'
import type { Ref } from 'vue'
import { EventKey } from '@/lib/constants/eventKeys'
import type { Axis, Box, GhostBox, Point } from '@/lib/drag/dragList'
import {
  DragThreshold,
  crossedThreshold,
  ghostOrigin,
  grabOffset,
} from '@/lib/drag/dragList'

// The choreography every list drag in the app shares: a press becomes a drag past the
// threshold and only then is the pointer captured (a capture from the first pixel would send
// the click to the container instead of the item), the rectangles are read once, the drop is
// recomputed on every move, and nothing moves until the release.
//
// What a drop *means* is the caller's: `resolve` turns a point into whatever that list calls a
// landing, `commit` applies it. The queue and the tab strip keep their own pure functions.
export interface DragListOptions<D> {
  axis: Axis
  // The element that takes the capture and hears the moves.
  container: Ref<HTMLElement | null>
  // The draggable items, in order, read at the moment the drag starts.
  items: () => HTMLElement[]
  resolve: (p: Point, boxes: Box[], from: number) => D | null
  commit: (from: number, drop: D | null) => void
  // A busy screen refuses to start a drag; nothing else stops one.
  enabled?: () => boolean
  threshold?: number
}

export interface DragList<D> {
  moving: Ref<boolean>
  from: Ref<number | null>
  drop: Ref<D | null>
  ghost: Ref<GhostBox | null>
  start: (index: number, e: PointerEvent) => void
  move: (e: PointerEvent) => void
  end: (e: PointerEvent) => void
  cancel: () => void
}

const boxOf = (el: HTMLElement): Box => {
  const r = el.getBoundingClientRect()
  return { left: r.left, top: r.top, width: r.width, height: r.height }
}

export const useDragList = <D>(options: DragListOptions<D>): DragList<D> => {
  const moving = ref(false)
  const from = ref<number | null>(null)
  const drop = shallowRef<D | null>(null)
  const ghost = shallowRef<GhostBox | null>(null)

  let press: Point | null = null
  let offset: Point = { x: 0, y: 0 }
  let size: Point = { x: 0, y: 0 }
  let boxes: Box[] = []
  let captured: number | null = null

  const clear = () => {
    moving.value = false
    from.value = null
    drop.value = null
    ghost.value = null
    press = null
    boxes = []
    const el = options.container.value
    if (captured !== null && el?.hasPointerCapture(captured))
      el.releasePointerCapture(captured)
    captured = null
    window.removeEventListener('keydown', onKeydown)
  }

  // A drag, once started, can be called off: Escape puts everything back and commits nothing.
  // Without it the only way out of a drag begun by accident is to drop it somewhere.
  function onKeydown(e: KeyboardEvent) {
    if (e.key !== EventKey.Escape) return
    e.preventDefault()
    clear()
  }

  const begin = (p: Point) => {
    const index = from.value
    const el = index === null ? undefined : options.items()[index]
    if (index === null || !el) return clear()
    boxes = options.items().map(boxOf)
    const box = boxOf(el)
    offset = grabOffset(box, press ?? p)
    size = { x: box.width, y: box.height }
    moving.value = true
    window.addEventListener('keydown', onKeydown)
  }

  const draw = (p: Point) => {
    const origin = ghostOrigin(offset, p)
    ghost.value = {
      left: origin.x,
      top: origin.y,
      width: size.x,
      height: size.y,
    }
  }

  const start = (index: number, e: PointerEvent) => {
    if (e.button !== 0 || options.enabled?.() === false) return
    from.value = index
    press = { x: e.clientX, y: e.clientY }
  }

  const move = (e: PointerEvent) => {
    if (from.value === null || !press) return
    const p = { x: e.clientX, y: e.clientY }
    if (!moving.value) {
      if (
        !crossedThreshold(
          options.axis,
          press,
          p,
          options.threshold ?? DragThreshold,
        )
      )
        return
      begin(p)
      if (!moving.value) return
      options.container.value?.setPointerCapture(e.pointerId)
      captured = e.pointerId
    }
    draw(p)
    drop.value = options.resolve(p, boxes, from.value)
  }

  const end = (e: PointerEvent) => {
    const index = from.value
    const landing = drop.value
    const dragged = moving.value
    clear()
    if (dragged && index !== null) options.commit(index, landing)
  }

  onBeforeUnmount(clear)

  return { moving, from, drop, ghost, start, move, end, cancel: clear }
}
```

- [ ] **Step 3: Write the ghost**

`ui/src/components/ui/drag/DragGhost.vue`:

```vue
<script setup lang="ts">
import type { GhostBox } from '@/lib/drag/dragList'

defineProps<{ box: GhostBox }>()
</script>

<template>
  <!-- The lifted copy, above everything and out of the way of the pointer: the original stays
       dimmed in its slot and the marker keeps naming the landing, so what the eye follows and
       what decides the drop are the same row twice.
       The skin is flat by decision (assets/theme/shadow.css: `--shadow-*: initial`), so the
       lift is a 2px primary border on an opaque sheet, not an invented shadow. Position and
       size are CSS variables bound here — convention 1's one exception to "no inline style". -->
  <Teleport to="body">
    <div
      class="pointer-events-none fixed top-(--drag-top) left-(--drag-left) z-50 h-(--drag-height) w-(--drag-width) overflow-hidden border-2 border-primary bg-sheet"
      :style="{
        '--drag-left': `${box.left}px`,
        '--drag-top': `${box.top}px`,
        '--drag-width': `${box.width}px`,
        '--drag-height': `${box.height}px`,
      }"
    >
      <slot />
    </div>
  </Teleport>
</template>
```

`ui/src/components/ui/drag/index.ts`:

```ts
export { default as DragGhost } from './DragGhost.vue'
```

- [ ] **Step 4: Put it on the Kit, with three rows**

`ui/src/kit/sections/app/DragListSection.vue`:

```vue
<script setup lang="ts">
import { computed, ref } from 'vue'
import { Card } from '@/components/ui/card'
import { DragGhost } from '@/components/ui/drag'
import { useDragList } from '@/composables/useDragList'
import { Axis, boxAt } from '@/lib/drag/dragList'
import type { Box, Point } from '@/lib/drag/dragList'
import KitSection from '../../KitSection.vue'

const rows = ref(['Isaac', 'Magdalene', 'Cain'])
const list = ref<HTMLElement | null>(null)

const items = (): HTMLElement[] =>
  list.value ? [...list.value.querySelectorAll<HTMLElement>('[data-kit-row]')] : []

const drag = useDragList<number>({
  axis: Axis.Y,
  container: list,
  items,
  resolve: (p: Point, boxes: Box[]) => boxAt(boxes, p, Axis.Y),
  commit: (from, to) => {
    if (to === null || to === from) return
    const next = [...rows.value]
    const [row] = next.splice(from, 1)
    if (row) next.splice(to, 0, row)
    rows.value = next
  },
})

const grabbed = computed(() =>
  drag.from.value === null ? null : rows.value[drag.from.value],
)
</script>

<template>
  <KitSection title="DragList">
    <Card class="w-80">
      <div
        ref="list"
        class="flex flex-col"
        @pointermove="drag.move"
        @pointerup="drag.end"
        @pointercancel="drag.end"
      >
        <div
          v-for="(row, index) in rows"
          :key="row"
          data-kit-row
          class="flex h-10 cursor-grab items-center border-b border-hairline px-3 text-label last:border-b-0"
          :class="drag.moving.value && drag.from.value === index && 'opacity-disabled'"
          @pointerdown="drag.start(index, $event)"
        >
          {{ row }}
        </div>
      </div>
    </Card>
    <DragGhost v-if="drag.ghost.value && grabbed" :box="drag.ghost.value">
      <div class="flex h-full items-center px-3 text-label">{{ grabbed }}</div>
    </DragGhost>
  </KitSection>
</template>
```

Add the import and the element to `ui/src/kit/KitPage.vue`, beside the other `app/` sections.

- [ ] **Step 5: Look at it**

Run: `pnpm ui:dev`, open `#kit`, find the DragList section.
Expected: pressing a row and moving 4 px lifts a copy that follows the cursor with the grab kept
where you pressed; the original dims; releasing reorders; **Escape mid-drag puts everything back
and reorders nothing**. Take the screenshot habit from 3.2: look, don't only typecheck.

- [ ] **Step 6: Check the conventions scanner is still clean**

Run: `pnpm scan`
Expected: `0 violations`. If the ghost's `:style` is flagged, it is because it uses a pixel
property and not a variable — fix the ghost, not the scanner.

- [ ] **Step 7: Commit**

```bash
git add ui/src/composables/useDragList.ts ui/src/components/ui/drag ui/src/kit/sections/app/DragListSection.vue ui/src/kit/KitPage.vue ui/src/lib/constants/eventKeys.ts
git commit -m "feat(ui): one composable owns the list drag, and what you grab lifts"
```

---

### Task 4: The tab strip drags through the composable

**Files:**
- Modify: `ui/src/components/shell/TabStrip.vue`
- Test: `ui/src/components/shell/tabs.test.ts` (unchanged — it must still pass untouched)

**Interfaces:**
- Consumes: `useDragList`, `DragGhost`, `Axis`, `boxAt`; the strip's own `dropSide`,
  `moveIndex`, `TabDrag.Threshold`, which do not move.

- [ ] **Step 1: Replace the hand-written choreography**

In `TabStrip.vue`, delete `interface Drag`, `tabRects`, `dropAt`, `onPointerDown`,
`onPointerMove`, `onPointerUp` — everything between `const strip = ref…` and `const neighbour
= …` except `tabElements` — and put in their place:

```ts
const strip = ref<HTMLElement | null>(null)

const tabElements = (): HTMLElement[] =>
  strip.value ? [...strip.value.querySelectorAll<HTMLElement>(tabSelector)] : []

// Where a tab lands: the half of the tab under the pointer decides, and a pointer over the
// dragged tab itself means nothing to do. `dropSide` and `moveIndex` stay the strip's own.
const resolve = (p: Point, boxes: Box[], from: number): Drop | null => {
  const index = boxAt(boxes, p, Axis.X)
  const box = index === null ? undefined : boxes[index]
  if (index === null || index === from || !box) return null
  return { index, side: dropSide(p.x, box.left, box.width) }
}

const drag = useDragList<Drop>({
  axis: Axis.X,
  container: strip,
  items: tabElements,
  threshold: TabDrag.Threshold,
  resolve,
  commit: (from, landing) => {
    if (!landing) return
    const to = moveIndex(from, landing.index, landing.side)
    if (to !== from) emit('move', from, to)
  },
})

const grabbed = computed(() =>
  drag.from.value === null ? null : props.tabs[drag.from.value],
)
```

with the imports:

```ts
import { computed, nextTick, ref } from 'vue'
import { DragGhost } from '@/components/ui/drag'
import { useDragList } from '@/composables/useDragList'
import { Axis, boxAt } from '@/lib/drag/dragList'
import type { Box, Point } from '@/lib/drag/dragList'
```

`interface Drop { index: number; side: DropSide }` stays.

- [ ] **Step 2: Rewire the template**

The `<div ref="strip">` handlers become `@pointermove="drag.move"`, `@pointerup="drag.end"`,
`@pointercancel="drag.end"`; each `TabItem` takes
`:dragging="drag.moving.value && drag.from.value === index"`,
`:drop="drag.drop.value?.index === index ? drag.drop.value.side : null"` and
`@pointerdown="drag.start(index, $event)"`. After the `</div>` of the strip, before the add
button, add the ghost:

```vue
    <DragGhost v-if="drag.ghost.value && grabbed" :box="drag.ghost.value">
      <TabItem :tab="grabbed" :active="true" :dragging="false" :drop="null" />
    </DragGhost>
```

- [ ] **Step 3: Run the suite**

Run: `pnpm ui:test`
Expected: PASS, and `tabs.test.ts` untouched — the pure functions did not move, so their tests
are the proof that the refactor kept the semantics.

- [ ] **Step 4: Look at it**

Run: `pnpm ui:dev` with `?fixture=active`. Open three or four tabs, drag one across the strip.
Expected: the dragged tab lifts and follows the cursor; the original dims in place; the drop
edge still appears on the side the tab will land on; a plain click still selects; middle click
still closes; `Escape` mid-drag cancels.

- [ ] **Step 5: Commit**

```bash
git add ui/src/components/shell/TabStrip.vue
git commit -m "refactor(ui): the tab strip drags through the shared composable"
```

---

### Task 5: The queue drags through the composable

**Files:**
- Modify: `ui/src/screens/plan/QueueCard.vue`
- Modify: `docs/frontend-conventions.md`

**Interfaces:**
- Consumes: `useDragList`, `DragGhost`, `Axis`, `boxAt`; the queue's own `dropEdge`,
  `dropAnchor`, `stepAnchor`, which do not move.

- [ ] **Step 1: Replace the hand-written choreography**

In `QueueCard.vue`, delete `interface Drag`, `dragThreshold`, `rowRects`, `dropAt`, `onGrab`,
`onPointerMove`, `onPointerUp`, and the `drag`/`drop` refs. Keep `anchorOf`'s intent, but read
it from the composable's drop:

```ts
const list = ref<HTMLElement | null>(null)

const ids = computed(() => props.rows.map(rowId))

const rowElements = (): HTMLElement[] =>
  list.value ? [...list.value.querySelectorAll<HTMLElement>(rowSelector)] : []

// The half of the row under the pointer decides the edge; `dropAnchor` then says whether that
// edge means anything — the gaps either side of the dragged row are where it already is.
const resolve = (p: Point, boxes: Box[]): Drop | null => {
  const index = boxAt(boxes, p, Axis.Y)
  const box = index === null ? undefined : boxes[index]
  if (index === null || !box) return null
  return { index, edge: dropEdge(p.y, box.top, box.height) }
}

const drag = useDragList<Drop>({
  axis: Axis.Y,
  container: list,
  items: rowElements,
  enabled: () => !props.busy,
  resolve,
  commit: (from, landing) => {
    const anchor = landing
      ? dropAnchor(ids.value, from, landing.index, landing.edge)
      : null
    const moved = ids.value[from]
    if (anchor && moved !== undefined) emit('move', moved, anchor.after)
  },
})

// The gap the line is drawn in, only for a drop that would move something.
const gap = computed((): number | null => {
  const landing = drag.drop.value
  const from = drag.from.value
  if (!drag.moving.value || !landing || from === null) return null
  if (!dropAnchor(ids.value, from, landing.index, landing.edge)) return null
  return landing.edge === DropEdge.Above ? landing.index : landing.index + 1
})

const grabbed = computed(() =>
  drag.from.value === null ? null : props.rows[drag.from.value],
)
```

with the same imports as Task 4 plus `DragGhost`.

- [ ] **Step 2: Rewire the template**

The list's handlers become `@pointermove="drag.move"`, `@pointerup="drag.end"`,
`@pointercancel="drag.end"`; `QueueRow` takes
`:dragging="drag.moving.value && drag.from.value === index"` and
`@grab="drag.start(index, $event)"`. The `hint` computed reads `drag.moving.value` instead of
`drag.value?.moving`. After the list, inside the `Card`:

```vue
    <DragGhost v-if="drag.ghost.value && grabbed" :box="drag.ghost.value">
      <QueueRow
        :row="grabbed"
        :rows="rows"
        :position="(drag.from.value ?? 0) + 1"
        :dragging="false"
        :busy="true"
      />
    </DragGhost>
```

`:busy="true"` on the copy is deliberate: the ghost's grip and remove button must not answer the
pointer — it is a picture of the row, not a second row.

- [ ] **Step 3: Run the suite**

Run: `pnpm ui:test`
Expected: PASS. `queueDrop.test.ts` is untouched: `dropEdge`, `dropAnchor` and `stepAnchor` did
not move, and Alt+arrow still goes through `onStep`, which this task does not touch.

- [ ] **Step 4: Look at it**

Run: `pnpm ui:dev` with `?fixture=active`, go to the Plan.
Expected: grabbing a row by its grip lifts the whole card above the page; the red marker still
names the landing; releasing puts the row there and the band still says where a move stopped and
why; Alt+↑/↓ still moves a row without the pointer; `Escape` cancels.

- [ ] **Step 5: Write the rule into the conventions**

In `docs/frontend-conventions.md`, where it describes the queue reordering "the way the tab strip
does", replace the description of the duplicated choreography with the rule:

```markdown
**One composable drags every list.** `composables/useDragList.ts` owns the threshold, the
capture, the rectangle snapshot, the lifted copy and `Escape`; `lib/drag/dragList.ts` holds the
decisions it makes, with its own tests. A screen brings two things and nothing else: where its
items are, and what a drop there means — `shell/tabs.ts` for the strip, `lib/plan/queueDrop.ts`
for the queue. A third hand-written pointer drag in a screen is a bug, not a variant. The
lifted copy is `components/ui/drag/DragGhost.vue`: the skin is flat by decision, so it lifts
with a `primary` border on an opaque sheet and never with a shadow token.
```

- [ ] **Step 6: `pnpm check` and commit**

Run: `pnpm check`
Expected: green.

```bash
git add ui/src/screens/plan/QueueCard.vue docs/frontend-conventions.md
git commit -m "refactor(ui): the queue drags through the shared composable, and B31 closes"
```

---

## Phase 2 — the structure: a second window that holds tabs

### Task 6: The window port, and its fake

**Files:**
- Create: `ui/src/lib/window/messages.ts`
- Create: `ui/src/lib/window/windowPort.ts`
- Create: `ui/src/lib/window/fakeWindows.ts`
- Modify: `crates/app/capabilities/default.json`

**Interfaces:**
- Consumes: Task 2's `Point`; `TabLocation` from `@/router/routeTable`.
- Produces: `WindowMessageKind`, `WindowMessage` and its members; `WindowBox`;
  `windowPort` (`label`, `isMain`, `list`, `create`, `send`, `broadcast`, `listen`, `focus`,
  `closeSelf`, `self`); `newWindowLabel()`, `MainLabel`. Tasks 8–16 consume them.

- [ ] **Step 1: Write the messages**

`ui/src/lib/window/messages.ts`:

```ts
import type { Point } from '@/lib/drag/dragList'
import type { TabLocation } from '@/router/routeTable'

// What windows say to each other. These never pass through Rust: they are frontend types on a
// frontend channel, not the IPC contract, and the rules that govern view-models do not apply.
// One Tauri event carries all of them, so a window has one listener and one exhaustive switch.
export const WindowEventName = 'isaac://window'

export const WindowMessageKind = {
  Ready: 'ready',
  Seed: 'seed',
  Docked: 'docked',
  Hovering: 'hovering',
  HoverLeft: 'hoverLeft',
  Focused: 'focused',
} as const
export type WindowMessageKind =
  (typeof WindowMessageKind)[keyof typeof WindowMessageKind]

// A tab as it travels: its location, never its content. That a tab is `{ id, location }` and
// not the view is what makes it movable between windows at all (`docs/BACKLOG.md` B15).
export interface TabSeed {
  location: TabLocation
}

// "I exist and I hold nothing": broadcast by a window that is not `main` when it mounts.
export interface ReadyMessage {
  kind: typeof WindowMessageKind.Ready
  label: string
}

// The answer to Ready, from the window that created it.
export interface SeedMessage {
  kind: typeof WindowMessageKind.Seed
  tabs: TabSeed[]
  activeIndex: number
}

// A tab dropped on this window's strip. `at` is in desktop physical pixels: the receiver
// converts it, because only the receiver knows its own position and scale factor.
export interface DockedMessage {
  kind: typeof WindowMessageKind.Docked
  tab: TabSeed
  at: Point
}

// A tab is being dragged over this window's strip right now: draw the marker. Sent at most
// once per animation frame.
export interface HoveringMessage {
  kind: typeof WindowMessageKind.Hovering
  at: Point
}

export interface HoverLeftMessage {
  kind: typeof WindowMessageKind.HoverLeft
}

// Broadcast by every window when it gains the focus. With no z-order API (tauri#5656) this is
// the only thing that tells two overlapping windows apart, and it costs one string.
export interface FocusedMessage {
  kind: typeof WindowMessageKind.Focused
  label: string
}

export type WindowMessage =
  | ReadyMessage
  | SeedMessage
  | DockedMessage
  | HoveringMessage
  | HoverLeftMessage
  | FocusedMessage
```

- [ ] **Step 2: Write the port**

`ui/src/lib/window/windowPort.ts`:

```ts
import { isTauri } from '@tauri-apps/api/core'
import { emit, emitTo, listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  WebviewWindow,
  getAllWebviewWindows,
} from '@tauri-apps/api/webviewWindow'
import type { Point } from '@/lib/drag/dragList'
import { fakeWindows } from './fakeWindows'
import { WindowEventName } from './messages'
import type { WindowMessage } from './messages'

// The first window's label, fixed by `tauri.conf.json`. Every other window is born here.
export const MainLabel = 'main'

// A window as the hit test needs it: desktop physical pixels, the units `cursorPosition()`
// answers in.
export interface WindowBox {
  label: string
  left: number
  top: number
  width: number
  height: number
  scaleFactor: number
}

export interface WindowPort {
  label: () => string
  isMain: () => boolean
  list: () => Promise<WindowBox[]>
  create: (label: string, at: Point, size: Point) => Promise<void>
  send: (label: string, message: WindowMessage) => Promise<void>
  broadcast: (message: WindowMessage) => Promise<void>
  listen: (handler: (message: WindowMessage) => void) => Promise<() => void>
  focus: (label: string) => Promise<void>
  closeSelf: () => Promise<void>
  self: () => Promise<WindowBox>
}

// A label nothing else holds: two windows are created at the same millisecond only if the user
// has two hands. Base 36 keeps it short enough to read in a log.
export const newWindowLabel = (): string => `win-${Date.now().toString(36)}`

interface Measurable {
  label: string
  outerPosition: () => Promise<{ x: number; y: number }>
  outerSize: () => Promise<{ width: number; height: number }>
  scaleFactor: () => Promise<number>
}

const boxOf = async (w: Measurable): Promise<WindowBox> => {
  const [p, s, f] = await Promise.all([
    w.outerPosition(),
    w.outerSize(),
    w.scaleFactor(),
  ])
  return {
    label: w.label,
    left: p.x,
    top: p.y,
    width: s.width,
    height: s.height,
    scaleFactor: f,
  }
}

const tauriPort: WindowPort = {
  label: () => getCurrentWindow().label,
  isMain: () => getCurrentWindow().label === MainLabel,
  list: async () => {
    const all = await getAllWebviewWindows()
    const shown = await Promise.all(
      all.map(async (w) =>
        (await w.isVisible()) && !(await w.isMinimized()) ? w : null,
      ),
    )
    return Promise.all(
      shown.filter((w): w is (typeof all)[number] => w !== null).map(boxOf),
    )
  },
  create: async (label, at, size) => {
    // The app's own page, with no state in its URL: what the window holds arrives through the
    // handshake (Task 8). Its own title bar, like the first window's.
    const w = new WebviewWindow(label, {
      url: 'index.html',
      x: at.x,
      y: at.y,
      width: size.x,
      height: size.y,
      decorations: false,
      backgroundColor: '#150e0d',
    })
    await new Promise<void>((resolve, reject) => {
      void w.once('tauri://created', () => resolve())
      void w.once('tauri://error', (e) => reject(new Error(String(e.payload))))
    })
  },
  send: async (label, message) => {
    await emitTo(label, WindowEventName, message)
  },
  broadcast: async (message) => {
    await emit(WindowEventName, message)
  },
  listen: (handler) =>
    listen<WindowMessage>(WindowEventName, (e) => handler(e.payload)),
  focus: async (label) => {
    const all = await getAllWebviewWindows()
    await all.find((w) => w.label === label)?.setFocus()
  },
  closeSelf: async () => {
    await getCurrentWindow().close()
  },
  self: () => boxOf(getCurrentWindow()),
}

// Outside Tauri — `pnpm ui:dev` in a browser tab — windows do not exist. The fake invents one
// so the gesture can be exercised and watched; it is not the verification (§9 of the spec is).
export const windowPort: WindowPort = isTauri() ? tauriPort : fakeWindows()
```

- [ ] **Step 3: Write the fake**

`ui/src/lib/window/fakeWindows.ts`:

```ts
import type { Point } from '@/lib/drag/dragList'
import type { WindowMessage } from './messages'
import type { WindowBox, WindowPort } from './windowPort'

// `?windows=fake` invents a second window to the right of the first, so a tear-off and a dock
// can be watched on the development server. Every message is logged rather than sent: there is
// nobody to send it to.
const FakeParam = 'windows'
const FakeValue = 'fake'

const mainBox: WindowBox = {
  label: 'main',
  left: 0,
  top: 0,
  width: 1280,
  height: 800,
  scaleFactor: 1,
}
const otherBox: WindowBox = {
  label: 'win-fake',
  left: 1300,
  top: 60,
  width: 900,
  height: 700,
  scaleFactor: 1,
}

export const fakeWindows = (): WindowPort => {
  const on =
    new URLSearchParams(window.location.search).get(FakeParam) === FakeValue
  const handlers: ((m: WindowMessage) => void)[] = []
  const say = (what: string, detail: unknown) => {
    if (on) console.info(`[windows] ${what}`, detail)
  }
  return {
    label: () => mainBox.label,
    isMain: () => true,
    list: () => Promise.resolve(on ? [mainBox, otherBox] : [mainBox]),
    create: (label: string, at: Point, size: Point) => {
      say('create', { label, at, size })
      return Promise.resolve()
    },
    send: (label: string, message: WindowMessage) => {
      say('send', { label, message })
      return Promise.resolve()
    },
    broadcast: (message: WindowMessage) => {
      say('broadcast', message)
      for (const h of handlers) h(message)
      return Promise.resolve()
    },
    listen: (handler: (m: WindowMessage) => void) => {
      handlers.push(handler)
      return Promise.resolve(() => {
        const at = handlers.indexOf(handler)
        if (at >= 0) handlers.splice(at, 1)
      })
    },
    focus: (label: string) => {
      say('focus', label)
      return Promise.resolve()
    },
    closeSelf: () => {
      say('closeSelf', null)
      return Promise.resolve()
    },
    self: () => Promise.resolve(mainBox),
  }
}
```

- [ ] **Step 4: Widen the capability**

`crates/app/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Every window draws its own title bar: it needs the window controls and dragging. A tab torn off becomes a window of its own, which needs creating windows, the cursor's position, and events between windows.",
  "windows": ["main", "win-*"],
  "permissions": [
    "core:default",
    "core:window:allow-minimize",
    "core:window:allow-toggle-maximize",
    "core:window:allow-close",
    "core:window:allow-start-dragging",
    "core:window:allow-cursor-position",
    "core:window:allow-set-position",
    "core:window:allow-set-size",
    "core:window:allow-set-focus",
    "core:window:allow-set-always-on-top",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-outer-position",
    "core:window:allow-outer-size",
    "core:window:allow-scale-factor",
    "core:window:allow-is-visible",
    "core:window:allow-is-minimized",
    "core:webview:allow-create-webview-window",
    "core:event:allow-listen",
    "core:event:allow-unlisten",
    "core:event:allow-emit",
    "core:event:allow-emit-to"
  ]
}
```

- [ ] **Step 5: Typecheck**

Run: `pnpm typecheck` then `pnpm scan`
Expected: green, `0 violations`. The scanner's window rule matches `@tauri-apps/api/window`
only, so the new imports pass for the wrong reason — Task 11 fixes that, and until then their
passing is not evidence.

- [ ] **Step 6: Commit**

```bash
git add ui/src/lib/window/messages.ts ui/src/lib/window/windowPort.ts ui/src/lib/window/fakeWindows.ts crates/app/capabilities/default.json
git commit -m "feat(ui): one port for the other windows, and a fake for the browser"
```

---

### Task 7: The tab model gives a tab away and takes one in

**Files:**
- Modify: `ui/src/stores/tabModel.ts`
- Test: `ui/src/stores/tabModel.test.ts`

**Interfaces:**
- Produces: `insertTab(state, at, tab)`, `canDetach(state)`, `detachTab(state, id): Detached |
  null`, `seedState(seeds, activeIndex, id)`. Tasks 8 and 9 consume them.

- [ ] **Step 1: Write the failing tests**

Append to `ui/src/stores/tabModel.test.ts` (adding `canDetach, detachTab, insertTab, seedState`
to its import from `./tabModel`, and `defaultLocation` from `@/router/routeTable` if it is not
already there):

```ts
describe('a tab that leaves, and one that arrives', () => {
  const three = (): TabsState => ({
    tabs: [
      { id: 'a', location: defaultLocation },
      { id: 'b', location: defaultLocation },
      { id: 'c', location: defaultLocation },
    ],
    activeId: 'b',
  })

  it('inserts an arriving tab at the index and selects it: a dropped tab is the one you want', () => {
    const after = insertTab(three(), 1, { id: 'd', location: defaultLocation })
    expect(after.tabs.map((t) => t.id)).toEqual(['a', 'd', 'b', 'c'])
    expect(after.activeId).toBe('d')
  })

  it('clamps an index past either end rather than dropping the tab', () => {
    expect(
      insertTab(three(), 99, { id: 'd', location: defaultLocation }).tabs.map(
        (t) => t.id,
      ),
    ).toEqual(['a', 'b', 'c', 'd'])
    expect(
      insertTab(three(), -3, { id: 'd', location: defaultLocation }).tabs.map(
        (t) => t.id,
      ),
    ).toEqual(['d', 'a', 'b', 'c'])
  })

  it('detaching hands back the tab and the state without it', () => {
    const out = detachTab(three(), 'b')
    expect(out?.tab.id).toBe('b')
    expect(out?.state.tabs.map((t) => t.id)).toEqual(['a', 'c'])
    // The active tab left, so its right neighbour takes over, exactly as closing does.
    expect(out?.state.activeId).toBe('c')
  })

  it('detaching a tab that is not there answers null and changes nothing', () => {
    expect(detachTab(three(), 'zzz')).toBeNull()
  })

  it('the last tab does not detach: that window already is that tab', () => {
    const one: TabsState = {
      tabs: [{ id: 'a', location: defaultLocation }],
      activeId: 'a',
    }
    expect(canDetach(one)).toBe(false)
    expect(detachTab(one, 'a')).toBeNull()
    expect(canDetach(three())).toBe(true)
  })

  it('a seeded window holds what it was given, active where it was told', () => {
    const state = seedState(
      [{ location: defaultLocation }, { location: defaultLocation }],
      1,
      (n) => `tab-${n}`,
    )
    expect(state.tabs).toHaveLength(2)
    expect(state.activeId).toBe(state.tabs[1]?.id)
  })

  it('a seed with nothing in it still leaves a bar with one tab', () => {
    const state = seedState([], 0, (n) => `tab-${n}`)
    expect(state.tabs).toHaveLength(1)
    expect(state.activeId).toBe(state.tabs[0]?.id)
  })
})
```

- [ ] **Step 2: Run them and watch them fail**

Run: `pnpm --filter ui test -- src/stores/tabModel.test.ts`
Expected: FAIL — `insertTab is not a function`.

- [ ] **Step 3: Implement them**

Append to `ui/src/stores/tabModel.ts` (its import from `@/router/routeTable` gains
`defaultLocation`):

```ts
// A tab arriving from another window lands at an index and takes the focus: you dropped it
// where you wanted to look at it. The index is clamped, never rejected — a drop a pixel past
// the last tab is a drop on the end, not a lost tab.
export const insertTab = (state: TabsState, at: number, tab: Tab): TabsState => {
  const index = Math.max(0, Math.min(at, state.tabs.length))
  const tabs = [...state.tabs]
  tabs.splice(index, 0, tab)
  return { tabs, activeId: tab.id }
}

// A window holding one tab *is* that tab: taking it out would leave a bar with nothing in it,
// the one state the rules forbid. So the gesture is refused before it starts rather than
// repaired after — "the bar is never empty", one level up.
export const canDetach = (state: TabsState): boolean => state.tabs.length > 1

export interface Detached {
  tab: Tab
  state: TabsState
}

export const detachTab = (state: TabsState, id: string): Detached | null => {
  if (!canDetach(state)) return null
  const tab = state.tabs.find((t) => t.id === id)
  if (!tab) return null
  // As far as what is left behind is concerned, leaving is closing: the same neighbour rule,
  // and `fresh` is never reached because `canDetach` has already refused the last tab.
  return { tab, state: closeTab(state, id, () => tab) }
}

// The state a window born from a tear-off starts in. An empty seed would leave a bar with no
// tabs, so it becomes one default tab: a window showing the landing page beats a window
// showing nothing.
export const seedState = (
  seeds: { location: TabLocation }[],
  activeIndex: number,
  id: (n: number) => string,
): TabsState => {
  const source = seeds.length > 0 ? seeds : [{ location: defaultLocation }]
  const tabs = source.map((seed, n) => ({ id: id(n), location: seed.location }))
  const active = tabs[Math.max(0, Math.min(activeIndex, tabs.length - 1))]
  return { tabs, activeId: active?.id ?? '' }
}
```

- [ ] **Step 4: Run them and watch them pass**

Run: `pnpm --filter ui test -- src/stores/tabModel.test.ts`
Expected: PASS, and the tests that were already in the file untouched.

- [ ] **Step 5: Commit**

```bash
git add ui/src/stores/tabModel.ts ui/src/stores/tabModel.test.ts
git commit -m "feat(ui): the tab model gives a tab away and takes one in"
```

---

### Task 8: A window is born through a handshake

**Files:**
- Create: `ui/src/lib/window/session.ts`
- Modify: `ui/src/stores/tabs.ts`
- Modify: `ui/src/App.vue`

**Interfaces:**
- Consumes: `windowPort`, `newWindowLabel`, `WindowMessageKind`, `TabSeed`, `seedState`.
- Produces: `useWindowSession()`, `oweSeed(label, tabs, activeIndex)`; on the store,
  `pending`, `seed(seeds, activeIndex)`, `openWindowWith(seeds, at, size)`. Tasks 9, 15 and 16
  consume them.

- [ ] **Step 1: Teach the store to wait for its seed**

In `ui/src/stores/tabs.ts`, replace the eager first state:

```ts
// `main` starts with its landing tab, as it always has. A window born from a tear-off starts
// empty and waits for its seed (`lib/window/session.ts`): what it holds is decided by the
// window that created it and never travels in its URL.
const born = windowPort.isMain()
const state = ref<TabsState>(
  born ? firstState(nextId(), defaultLocation) : { tabs: [], activeId: '' },
)
const pending = ref(!born)

const seed = (seeds: TabSeed[], activeIndex: number): void => {
  state.value = seedState(seeds, activeIndex, () => nextId())
  pending.value = false
}
```

and return `pending` and `seed` beside the rest. `active` already answers `undefined` for an
empty bar, and the shell renders nothing for it.

- [ ] **Step 2: Write the session**

`ui/src/lib/window/session.ts`:

```ts
import { onBeforeUnmount, onMounted } from 'vue'
import { assertNever } from '@/lib/assertNever'
import { useTabsStore } from '@/stores/tabs'
import { WindowMessageKind } from './messages'
import type { TabSeed, WindowMessage } from './messages'
import { windowPort } from './windowPort'

// How long a newborn window waits for the seed that says what it holds. If the window that
// created it died in between, nobody will ever answer: it opens on the landing page rather
// than showing a bar with no tabs. Degrade, never fail.
const SeedTimeout = 3000

// What a window that has just created another owes it: the answer to its Ready.
const owed = new Map<string, { tabs: TabSeed[]; activeIndex: number }>()

export const oweSeed = (
  label: string,
  tabs: TabSeed[],
  activeIndex: number,
): void => {
  owed.set(label, { tabs, activeIndex })
}

// A window's whole cross-window life: one listener, one exhaustive switch. Mounted once, by
// App.vue. Docking joins it in Task 9, hovering in Task 16.
export const useWindowSession = (): void => {
  const tabs = useTabsStore()
  let stop: (() => void) | null = null
  let timer: number | null = null

  const onMessage = (m: WindowMessage) => {
    switch (m.kind) {
      case WindowMessageKind.Ready: {
        const seed = owed.get(m.label)
        if (!seed) return
        owed.delete(m.label)
        void windowPort.send(m.label, {
          kind: WindowMessageKind.Seed,
          tabs: seed.tabs,
          activeIndex: seed.activeIndex,
        })
        return
      }
      case WindowMessageKind.Seed:
        if (!tabs.pending) return
        if (timer !== null) window.clearTimeout(timer)
        tabs.seed(m.tabs, m.activeIndex)
        return
      case WindowMessageKind.Docked:
      case WindowMessageKind.Hovering:
      case WindowMessageKind.HoverLeft:
      case WindowMessageKind.Focused:
        // Task 9 and Task 16.
        return
      default:
        return assertNever(m)
    }
  }

  onMounted(async () => {
    stop = await windowPort.listen(onMessage)
    if (!tabs.pending) return
    timer = window.setTimeout(() => tabs.seed([], 0), SeedTimeout)
    await windowPort.broadcast({
      kind: WindowMessageKind.Ready,
      label: windowPort.label(),
    })
  })

  onBeforeUnmount(() => {
    stop?.()
    if (timer !== null) window.clearTimeout(timer)
  })
}
```

- [ ] **Step 3: Mount it**

In `ui/src/App.vue`, call `useWindowSession()` in the setup, beside the other composables.

- [ ] **Step 4: Give the store a way to open a window**

In `ui/src/stores/tabs.ts`:

```ts
  // Opens a window holding these tabs, at this place and size, and owes it its seed before it
  // can ask: the newborn broadcasts Ready the moment it mounts, which may be before or after
  // `create` resolves. The debt is registered first, so neither order loses it.
  const openWindowWith = async (
    seeds: TabSeed[],
    at: Point,
    size: Point,
  ): Promise<void> => {
    const label = newWindowLabel()
    oweSeed(label, seeds, seeds.length - 1)
    await windowPort.create(label, at, size)
  }
```

- [ ] **Step 5: Prove it on the machine**

There is no gesture yet, so drive it from the Kit: add a temporary button that calls
`openWindowWith` with two seeds, and remove it before the commit.

Run: `pnpm dev`
Expected: a second window opens with its own title bar, **two tabs in its strip**, the second
one active, navbar and sidebar in place; the first window unchanged. Write in the report how
long the seed took, wall-clock, from the click to the tabs appearing.

- [ ] **Step 6: Commit**

```bash
git add ui/src/lib/window/session.ts ui/src/stores/tabs.ts ui/src/App.vue
git commit -m "feat(ui): a second window is born with the tabs it was given"
```

---

### Task 9: Docking a tab, and the two closing rules

**Files:**
- Create: `ui/src/lib/window/focusOrder.ts`
- Test: `ui/src/lib/window/focusOrder.test.ts`
- Modify: `ui/src/lib/window/session.ts`
- Modify: `ui/src/stores/tabs.ts`

**Interfaces:**
- Consumes: `insertTab`, `detachTab`, `canDetach`, `windowPort`, `watchWindowFocus`.
- Produces: `rememberFocus(order, label)`, the `focusOrder` ref; on the store, `dock(seed, at)`,
  `giveAway(id, target, at)`, `canTear`. Tasks 12, 15 and 16 consume them.

- [ ] **Step 1: Write the failing test**

`ui/src/lib/window/focusOrder.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { rememberFocus } from './focusOrder'

describe('rememberFocus', () => {
  it('puts the window that just took the focus first', () => {
    expect(rememberFocus(['a', 'b'], 'b')).toEqual(['b', 'a'])
  })

  it('never holds the same window twice', () => {
    expect(rememberFocus(['a', 'b', 'a'], 'a')).toEqual(['a', 'b'])
  })

  it('learns a window it had never seen', () => {
    expect(rememberFocus([], 'win-1')).toEqual(['win-1'])
  })
})
```

- [ ] **Step 2: Run it and watch it fail**

Run: `pnpm --filter ui test -- src/lib/window/focusOrder.test.ts`
Expected: FAIL — the module does not exist.

- [ ] **Step 3: Write it**

`ui/src/lib/window/focusOrder.ts`:

```ts
import { ref } from 'vue'

// Tauri has no z-order API (tauri#5656), so with two overlapping windows nothing tells us which
// one the user would call the top one. Every window broadcasts when it takes the focus and every
// window keeps the same list, most recent first. It is not the z-order — a window can be raised
// without focus — and the hit test only reaches for it when two windows both hold the point.
export const rememberFocus = (order: string[], label: string): string[] => [
  label,
  ...order.filter((l) => l !== label),
]

export const focusOrder = ref<string[]>([])
```

- [ ] **Step 4: Run it and watch it pass**

Run: `pnpm --filter ui test -- src/lib/window/focusOrder.test.ts`
Expected: PASS.

- [ ] **Step 5: Give a tab away, and take one in**

In `ui/src/stores/tabs.ts`:

```ts
  const canTear = computed(() => canDetach(state.value))

  // A tab arriving from another window. `at` is an index in this strip; Task 12 computes it
  // from the drop point, and until then the caller passes the end.
  const dock = (seed: TabSeed, at: number): void => {
    state.value = insertTab(state.value, at, {
      id: nextId(),
      location: seed.location,
    })
  }

  // A tab leaving for another window. The order is load-bearing: the target is told first and
  // only then does the tab leave here, so a target that never answers costs nothing — the tab
  // is still in this window. A window holding one tab refuses: it already is that tab.
  const giveAway = async (
    id: string,
    target: string,
    at: Point,
  ): Promise<boolean> => {
    const out = detachTab(state.value, id)
    if (!out) return false
    await windowPort.send(target, {
      kind: WindowMessageKind.Docked,
      tab: { location: out.tab.location },
      at,
    })
    state.value = out.state
    await windowPort.focus(target)
    // The last tab of a window that is not `main` takes the window with it.
    if (state.value.tabs.length === 0 && !windowPort.isMain())
      await windowPort.closeSelf()
    return true
  }
```

- [ ] **Step 6: Hear the two messages**

In `session.ts`, fill two of the arms left empty in Task 8:

```ts
      case WindowMessageKind.Docked:
        tabs.dock(m.tab, tabs.tabs.length)
        return
      case WindowMessageKind.Focused:
        focusOrder.value = rememberFocus(focusOrder.value, m.label)
        return
```

and broadcast this window's own focus through the module that already owns focus,
`watchWindowFocus` in `lib/window/appWindow.ts`, inside the same `onMounted`:

```ts
    stopFocus = await watchWindowFocus((focused) => {
      if (focused)
        void windowPort.broadcast({
          kind: WindowMessageKind.Focused,
          label: windowPort.label(),
        })
    })
```

with `stopFocus?.()` in `onBeforeUnmount`.

- [ ] **Step 7: Prove it on the machine**

Run `pnpm dev`, open a second window with Task 8's temporary Kit button, then call `giveAway`
from the first window's console with the second window's label.

Expected, each checked separately:
- the tab appears at the end of the second window's strip, selected, and the second window takes
  the focus; the tab is gone from the first
- repeat until the origin has one tab left: the call answers `false` and nothing moves
- from a **secondary** window, give away its last tab: that window closes
- from `main`, give away its last tab: `main` stays, with a fresh landing tab

- [ ] **Step 8: Commit**

```bash
git add ui/src/lib/window/focusOrder.ts ui/src/lib/window/focusOrder.test.ts ui/src/lib/window/session.ts ui/src/stores/tabs.ts
git commit -m "feat(ui): a tab moves between windows, and a window that loses its last one closes"
```

---

### Task 10: What one window writes, every window reads again

**Files:**
- Modify: `crates/app/src/commands/profile.rs`
- Modify: `crates/app/src/commands/queue.rs`
- Modify: `crates/app/src/commands/plan.rs`
- Create: `ui/src/lib/window/appEvents.ts`
- Modify: `ui/src/App.vue`

**Interfaces:**
- Produces: `AppEvent` (three names), `watchAppEvents(handlers): Promise<() => void>`.

- [ ] **Step 1: Emit from the commands that write**

In `crates/app/src/commands/profile.rs`, add `use tauri::Emitter;` and, in `select_profile`
after the successful `settings_file::save`:

```rust
    // The active profile is the app's now that there can be more than one window: the event
    // says "read again" and carries nothing. A payload would be a copy of state the next
    // command could contradict, and a second wire shape to keep in camelCase, for nothing.
    let _ = app.emit("profile-changed", ());
```

and in `set_scale`, after its save:

```rust
    let _ = app.emit("settings-changed", ());
```

The `let _ =` is deliberate: a window closing at that instant makes the emit fail, and a failed
notification must not turn a successful write into an error.

In `queue.rs`, the same line with `"plan-changed"` at the end of `queue_add`, `queue_remove`,
`queue_move` and `queue_import_goals`, after `queue_mutate` has succeeded and before the view is
built; in `plan.rs`, in `add_goal` and `remove_goal` after their writes. The queue and the goals
are one screen, so they are one event.

- [ ] **Step 2: Listen**

`ui/src/lib/window/appEvents.ts`:

```ts
import { listen } from '@tauri-apps/api/event'

// Events emitted by Rust when a command has just written. They carry no payload: they say
// "read again", and each window answers with the commands it already calls. Nothing new
// crosses the IPC boundary, so nothing new can leak across it.
export const AppEvent = {
  ProfileChanged: 'profile-changed',
  SettingsChanged: 'settings-changed',
  PlanChanged: 'plan-changed',
} as const
export type AppEvent = (typeof AppEvent)[keyof typeof AppEvent]

export const watchAppEvents = async (
  handlers: Record<AppEvent, () => void>,
): Promise<() => void> => {
  const stops = await Promise.all(
    Object.values(AppEvent).map((name) => listen(name, () => handlers[name]())),
  )
  return () => {
    for (const stop of stops) stop()
  }
}
```

- [ ] **Step 3: Wire the three re-reads**

In `App.vue`'s setup, beside `useWindowSession()`. Read the three stores' own reader names in
`stores/profile.ts`, `stores/settings.ts` and `stores/queue.ts` before writing this — they are
`load` on the first two, and whatever `queue.ts` exposes on the third:

```ts
// A window never learns of a write it did not make, so it is told. `useOnActiveProfile` carries
// the profile through to every screen that reads the save.
onMounted(async () => {
  stopAppEvents = await watchAppEvents({
    [AppEvent.ProfileChanged]: () => void profile.load(),
    [AppEvent.SettingsChanged]: () => void settings.load(),
    [AppEvent.PlanChanged]: () => void queue.load(),
  })
})
```

with `onBeforeUnmount(() => stopAppEvents?.())`.

- [ ] **Step 4: Prove it on the machine**

Run `pnpm dev`, open a second window, put both on the Plan.
Expected: a move in one appears in the other within a moment; a profile change in one moves the
other's indicator and Progress screens; a scale change in one resizes the other. **Check the
negative too**: with one window open nothing loops — a window reacting to its own event must not
emit it again.

- [ ] **Step 5: `pnpm check` and commit**

Run: `pnpm check`
Expected: green.

```bash
git add crates/app/src/commands/profile.rs crates/app/src/commands/queue.rs crates/app/src/commands/plan.rs ui/src/lib/window/appEvents.ts ui/src/App.vue
git commit -m "feat(app): a write in one window is read again by every window"
```

---

### Task 11: The scanner actually covers the new namespaces

**Files:**
- Modify: `ui/scripts/scan-conventions.mjs`

- [ ] **Step 1: Watch it catch nothing**

Temporarily add `import { emit } from '@tauri-apps/api/event'` to any component under
`ui/src/components/`, then run `pnpm scan`.
Expected: `0 violations` — which is the bug. The rule matches `@tauri-apps/api/window` only, and
both namespaces this sub-project introduces walk straight past it.

- [ ] **Step 2: Widen the rule**

```js
    name: 'window API outside src/lib/window/',
    test: (file, body) =>
      /@tauri-apps\/api\/(window|webviewWindow|event)/.test(body) &&
      !isUnder(file, WINDOW_DIR),
```

- [ ] **Step 3: Watch it catch it, then remove the bait**

Run: `pnpm scan`
Expected: 1 violation, naming that component. Remove the temporary import, run again:
`0 violations`.

- [ ] **Step 4: Commit**

```bash
git add ui/scripts/scan-conventions.mjs
git commit -m "chore(ui): the window rule covers the event and webview namespaces too"
```

---

## Phase 3 — the gesture that leaves the window

### Task 12: Where the point is, in a world of windows

**Files:**
- Create: `ui/src/lib/window/tearOff.ts`
- Test: `ui/src/lib/window/tearOff.test.ts`

**Interfaces:**
- Consumes: `Point`, `Box` (Task 2), `WindowBox` (Task 6).
- Produces: `StripBand`, `toClient`, `toDesktop`, `holdsPoint`, `inStripBand`,
  `windowUnderPoint`, `pastTearBand`. Tasks 15 and 16 consume them.

- [ ] **Step 1: Write the failing tests**

`ui/src/lib/window/tearOff.test.ts`:

```ts
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

  it('a second monitor at another factor does not borrow the first one is', () => {
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
    // 520,310 is inside both: deep in a's body, and on b's strip.
    expect(windowUnderPoint([a, b], { x: 520, y: 310 }, ['a'])).toBe('b')
  })

  it('falls back to the most recently focused when both hold it the same way', () => {
    // 520,400 is in both bodies. There is no z-order API, so the focus order decides.
    expect(windowUnderPoint([a, b], { x: 520, y: 400 }, ['b', 'a'])).toBe('b')
    expect(windowUnderPoint([a, b], { x: 520, y: 400 }, ['a', 'b'])).toBe('a')
  })

  it('with no focus order at all still answers a window, not null', () => {
    expect(windowUnderPoint([a, b], { x: 520, y: 400 }, [])).not.toBeNull()
  })
})

describe('the strip band', () => {
  it('is the top of the window, in its own logical pixels', () => {
    const w = win('a', 0, 0, 2)
    expect(inStripBand(w, { x: 100, y: 10 })).toBe(true)
    // 2x: the band is StripBand logical pixels, so twice as many physical ones.
    expect(inStripBand(w, { x: 100, y: StripBand * 2 - 1 })).toBe(true)
    expect(inStripBand(w, { x: 100, y: StripBand * 2 + 1 })).toBe(false)
  })

  it('is not a band across the whole desktop', () => {
    expect(inStripBand(win('a', 0, 0), { x: 4000, y: 10 })).toBe(false)
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
```

- [ ] **Step 2: Run them and watch them fail**

Run: `pnpm --filter ui test -- src/lib/window/tearOff.test.ts`
Expected: FAIL — the module does not exist.

- [ ] **Step 3: Write the module**

`ui/src/lib/window/tearOff.ts`:

```ts
import type { Box, Point } from '@/lib/drag/dragList'
import type { WindowBox } from './windowPort'

// The top of a window, in its own logical pixels: where its tab strip is drawn. A drop here
// joins that window's strip; a drop lower down is a drop on a window, which means the same
// thing but reads as less deliberate, so the band wins ties (windowUnderPoint).
export const StripBand = 40

// How far from the strip the pointer has to travel before the tab leaves the window. Only
// perpendicular travel counts: sliding far along the strip is how you reach its far end.
export const TearBand = 24

// Desktop physical pixels — what `cursorPosition()` answers — into one window's logical page
// pixels. Two monitors can disagree on the factor, so the window's own is the only one to use:
// `devicePixelRatio` is this window's, and this window may not be the one under the pointer.
export const toClient = (p: Point, w: WindowBox): Point => ({
  x: (p.x - w.left) / w.scaleFactor,
  y: (p.y - w.top) / w.scaleFactor,
})

export const toDesktop = (p: Point, w: WindowBox): Point => ({
  x: p.x * w.scaleFactor + w.left,
  y: p.y * w.scaleFactor + w.top,
})

export const holdsPoint = (w: WindowBox, p: Point): boolean =>
  p.x >= w.left &&
  p.x < w.left + w.width &&
  p.y >= w.top &&
  p.y < w.top + w.height

export const inStripBand = (w: WindowBox, p: Point): boolean =>
  holdsPoint(w, p) && p.y < w.top + StripBand * w.scaleFactor

// Which window a point belongs to. There is no z-order API (tauri#5656), so overlapping windows
// are ambiguous and the rule is ours: a strip beats a body, and among equals the most recently
// focused wins. With nothing to go on it still answers a window rather than null — a drop that
// lands somewhere beats a drop that vanishes.
export const windowUnderPoint = (
  windows: WindowBox[],
  p: Point,
  order: string[],
): string | null => {
  const holding = windows.filter((w) => holdsPoint(w, p))
  if (holding.length === 0) return null
  const strips = holding.filter((w) => inStripBand(w, p))
  const candidates = strips.length > 0 ? strips : holding
  const ranked = [...candidates].sort(
    (a, b) => rank(order, a.label) - rank(order, b.label),
  )
  return ranked[0]?.label ?? null
}

const rank = (order: string[], label: string): number => {
  const at = order.indexOf(label)
  return at < 0 ? order.length : at
}

// Has the tab left the strip? Measured perpendicular to the strip, from its own rectangle, in
// client pixels — the pointer is still inside the window when this first becomes true.
export const pastTearBand = (p: Point, strip: Box): boolean =>
  p.y > strip.top + strip.height + TearBand || p.y < strip.top - TearBand
```

- [ ] **Step 4: Run them and watch them pass**

Run: `pnpm --filter ui test -- src/lib/window/tearOff.test.ts`
Expected: PASS, 12 tests.

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/window/tearOff.ts ui/src/lib/window/tearOff.test.ts
git commit -m "feat(ui): where a point is when there is more than one window"
```

---

### Task 13: Where the pointer is once it has left

**Files:**
- Create: `ui/src/lib/window/pointerSource.ts`
- Create (only if the spike said so): `crates/app/src/cursor.rs`, and `crates/app/src/lib.rs`,
  `crates/app/Cargo.toml`
- Modify: `docs/superpowers/reports/2026-09-13-drag-and-windows-report.md`

**Interfaces:**
- Consumes: Task 1's answer.
- Produces: `watchPointer(onMove, onRelease): () => void`. Task 15 consumes it.

**Read Task 1's section of the report before starting.** If it says the DOM keeps delivering
events outside the window, do step 1 and skip steps 2–4. If it says the events stop, skip step 1
and do steps 2–4. Either way, do step 5.

**Amendment, 2026-09-13.** The spike could not be run — it needs a hand on the mouse (report,
Task 1) — so step 1's implementation was written **as the provisional answer**, with two changes
the plan had not foreseen, both forced by the fact that the answer is unknown:

- the **position** comes from `cursorPosition()` polled once per animation frame in *both*
  variants, not from the events. It is the same source the hit test compares against, and it
  removes one thing the spike's answer could change.
- a third callback, **`onLost`**: if no pointer event arrives for 10 s, or the cursor cannot be
  read, the drag is **cancelled and the tab goes back**. That is exactly the failure the spike
  is about, and a gesture that hangs forever would be worse than one that gives up — landing a
  tab where nobody released it would be worse still. When the measurement is made, either it
  confirms the DOM source and the timeout stays as a safety net, or it forces the Rust source
  and the timeout becomes unreachable.

- [ ] **Step 1: The DOM source (only if the spike said the events arrive)**

`ui/src/lib/window/pointerSource.ts`:

```ts
import { cursorPosition } from '@tauri-apps/api/window'
import type { Point } from '@/lib/drag/dragList'

// Where the pointer is while the tab is outside the window, and when the button comes up.
// Which implementation this file holds was decided by a measurement, not by the documentation:
// see "Task 1 — the spike" in this sub-project's report.
//
// The DOM source: WebView2 keeps delivering pointer events to the captured element with the
// cursor outside the window, so the events are the truth and `cursorPosition()` is used only to
// convert into desktop coordinates, which the hit test needs.
export const watchPointer = (
  onMove: (p: Point) => void,
  onRelease: (p: Point) => void,
): (() => void) => {
  let last: Point = { x: 0, y: 0 }

  const desktop = async (): Promise<Point> => {
    const p = await cursorPosition()
    return { x: p.x, y: p.y }
  }

  const move = () => {
    void desktop().then((p) => {
      last = p
      onMove(p)
    })
  }

  const up = () => {
    stop()
    onRelease(last)
  }

  const stop = () => {
    window.removeEventListener('pointermove', move)
    window.removeEventListener('pointerup', up)
  }

  window.addEventListener('pointermove', move)
  window.addEventListener('pointerup', up)
  return stop
}
```

- [ ] **Step 2: The Rust source — the button (only if the spike said the events stop)**

`crates/app/src/cursor.rs`:

```rust
//! Whether the left mouse button is still down, while the cursor is outside our windows.
//!
//! WebView2 stops delivering pointer events once the cursor leaves the control
//! (microsoft-ui-xaml #8677), measured on this machine in Task 1 of
//! `docs/superpowers/plans/archive/2026-09-13-drag-and-windows.md`: without this the drag would hang
//! with the button already released, which is the worst state the gesture can be in.
//!
//! It reports **nothing but the release**, with no payload: where the cursor is, the window
//! asks `cursorPosition()` for itself. The thread lives only while a drag does.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

/// ~120 Hz: a drag feels late above 16 ms, and polling a key state is cheaper than a frame.
const POLL: Duration = Duration::from_millis(8);

#[derive(Default)]
pub struct DragWatch {
    running: Arc<AtomicBool>,
}

#[cfg(windows)]
fn button_down() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    // The high bit is "down now"; the low bit is "was pressed since last asked", which would
    // report a click that already ended.
    (unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } as u16 & 0x8000) != 0
}

// Every other platform keeps whatever its webview gives us: the DOM source stays in charge
// there, and this never starts.
#[cfg(not(windows))]
fn button_down() -> bool {
    false
}

#[tauri::command]
pub fn drag_watch_start(app: AppHandle, state: tauri::State<'_, DragWatch>) {
    if state.running.swap(true, Ordering::SeqCst) {
        return;
    }
    let running = state.running.clone();
    std::thread::spawn(move || {
        // A drag that somehow never sees a release must not leave a thread behind: four
        // seconds is longer than any drag and shorter than any leak that matters.
        let deadline = std::time::Instant::now() + Duration::from_secs(4);
        while running.load(Ordering::SeqCst) && std::time::Instant::now() < deadline {
            if !button_down() {
                break;
            }
            std::thread::sleep(POLL);
        }
        running.store(false, Ordering::SeqCst);
        let _ = app.emit("drag-released", ());
    });
}

#[tauri::command]
pub fn drag_watch_stop(state: tauri::State<'_, DragWatch>) {
    state.running.store(false, Ordering::SeqCst);
}
```

In `crates/app/src/lib.rs`: `mod cursor;`, `.manage(cursor::DragWatch::default())`, and
`cursor::drag_watch_start, cursor::drag_watch_stop` in `generate_handler!`. In
`crates/app/Cargo.toml`:

```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
  "Win32_Foundation",
  "Win32_UI_Input_KeyboardAndMouse",
] }
```

- [ ] **Step 3: The Rust source — the frontend half**

`ui/src/lib/window/pointerSource.ts`:

```ts
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { cursorPosition } from '@tauri-apps/api/window'
import type { Point } from '@/lib/drag/dragList'

// Where the pointer is while the tab is outside the window, and when the button comes up.
// Which implementation this file holds was decided by a measurement, not by the documentation:
// see "Task 1 — the spike" in this sub-project's report. Here, WebView2 stopped talking at the
// window's edge, so Rust watches the button.
//
// The position still comes from `cursorPosition()`, polled a frame at a time: the event from
// Rust carries no payload, as every event from Rust does — it says the drag is over, and the
// window that cares reads the rest itself.
export const watchPointer = (
  onMove: (p: Point) => void,
  onRelease: (p: Point) => void,
): (() => void) => {
  let last: Point = { x: 0, y: 0 }
  let frame = 0
  let stopped = false
  let unlisten: (() => void) | null = null

  const tick = () => {
    if (stopped) return
    void cursorPosition().then((p) => {
      last = { x: p.x, y: p.y }
      onMove(last)
      frame = requestAnimationFrame(tick)
    })
  }

  const stop = () => {
    if (stopped) return
    stopped = true
    cancelAnimationFrame(frame)
    unlisten?.()
    void invoke('drag_watch_stop')
  }

  void listen('drag-released', () => {
    stop()
    onRelease(last)
  }).then((off) => {
    unlisten = off
    if (stopped) off()
  })
  void invoke('drag_watch_start')
  frame = requestAnimationFrame(tick)
  return stop
}
```

- [ ] **Step 4: Prove the source can speak before trusting its silence**

Run `pnpm dev` and, from the console, call `watchPointer(console.log, () => console.log('up'))`
then press and release the button with the cursor over the desktop.
Expected: a stream of positions, then exactly one `up`. **An instrument that reports nothing
proves nothing until it has been shown able to speak**: if no `up` arrives, the gesture would
read a dead source as "still dragging" and never land.

- [ ] **Step 5: Write which one you built, and why**

In the report, under Task 1's answers, one paragraph: which implementation is in
`pointerSource.ts`, the measurement that chose it, and what would have to change to flip it.

- [ ] **Step 6: `pnpm check` and commit**

```bash
git add ui/src/lib/window/pointerSource.ts docs/superpowers/reports/2026-09-13-drag-and-windows-report.md
git commit -m "feat(ui): follow the pointer once it has left the window"
```

(add `crates/app/src/cursor.rs crates/app/src/lib.rs crates/app/Cargo.toml Cargo.lock` to the
`git add` if steps 2–3 were the ones you did.)

---

### Task 14: The preview that follows the cursor

**Files:**
- Create: `ui/preview.html`
- Create: `ui/src/preview/main.ts`
- Create: `ui/src/preview/PreviewPage.vue`
- Create: `ui/src/lib/window/preview.ts`
- Modify: `ui/vite.config.ts`
- Modify: `crates/app/capabilities/default.json`

**Interfaces:**
- Produces: `showPreview(label, at)`, `movePreview(at)`, `hidePreview()`, `PreviewLabel`.
  Task 15 consumes them.

- [ ] **Step 1: The page**

`ui/preview.html` — the same skeleton as `index.html`, without the splash, with
`<script type="module" src="/src/preview/main.ts">` and the same background colour on `<html>`.

`ui/src/preview/main.ts`:

```ts
import { createApp } from 'vue'
import '../assets/main.css'
import PreviewPage from './PreviewPage.vue'

// The preview is one card with a name on it: no router, no store, no i18n, no IPC. It has to
// be on screen in the time between two frames of a drag, and everything it does not load is
// time it does not spend.
createApp(PreviewPage).mount('#app')
```

`ui/src/preview/PreviewPage.vue`:

```vue
<script setup lang="ts">
// The tab's label, in the query: a title is not a path, an offset or a profile, and nothing
// else travels with it. This page is not part of the app's routing and never becomes one.
const label = new URLSearchParams(window.location.search).get('label') ?? ''
</script>

<template>
  <div
    class="flex h-screen w-screen items-center gap-1.5 overflow-hidden border-2 border-primary bg-sheet px-2 text-label text-foreground select-none"
  >
    <span class="min-w-0 flex-1 truncate">{{ label }}</span>
  </div>
</template>
```

- [ ] **Step 2: Teach Vite about the second page**

`ui/vite.config.ts`:

```ts
import { resolve } from 'node:path'
// …
  build: {
    rollupOptions: {
      input: {
        index: resolve(__dirname, 'index.html'),
        // The tab preview that follows the cursor during a tear-off. A second entry, not a
        // route: it must not carry the app's bundle.
        preview: resolve(__dirname, 'preview.html'),
      },
    },
  },
```

- [ ] **Step 3: The window that shows it**

`ui/src/lib/window/preview.ts`:

```ts
import { isTauri } from '@tauri-apps/api/core'
import { LogicalPosition, LogicalSize } from '@tauri-apps/api/dpi'
import { WebviewWindow, getAllWebviewWindows } from '@tauri-apps/api/webviewWindow'
import type { Point } from '@/lib/drag/dragList'

// The tab that follows the cursor once it has left the strip. It is created on the first
// tear-off of the session and afterwards only hidden and shown again: creating a window costs
// a webview, and the second drag must not pay it.
export const PreviewLabel = 'tab-preview'

const Size = { width: 200, height: 34 }
// The cursor holds the preview near its top-left corner, the way it held the tab.
const Grab = { x: 24, y: 16 }

let preview: WebviewWindow | null = null

const found = async (): Promise<WebviewWindow | null> => {
  if (preview) return preview
  const all = await getAllWebviewWindows()
  preview = all.find((w) => w.label === PreviewLabel) ?? null
  return preview
}

const at = (p: Point): LogicalPosition =>
  new LogicalPosition(p.x - Grab.x, p.y - Grab.y)

export const showPreview = async (label: string, p: Point): Promise<void> => {
  if (!isTauri()) return
  const existing = await found()
  if (existing) {
    await existing.setPosition(at(p))
    await existing.show()
    return
  }
  // `focus: false` is the whole gesture's life: a window that takes the focus takes the
  // origin's pointer capture with it, and the drag dies in the act.
  preview = new WebviewWindow(PreviewLabel, {
    url: `preview.html?label=${encodeURIComponent(label)}`,
    width: Size.width,
    height: Size.height,
    x: p.x - Grab.x,
    y: p.y - Grab.y,
    decorations: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    shadow: false,
    resizable: false,
    focus: false,
  })
  await new Promise<void>((resolve) => {
    void preview?.once('tauri://created', () => resolve())
  })
}

export const movePreview = async (p: Point): Promise<void> => {
  const w = await found()
  await w?.setPosition(at(p))
}

export const hidePreview = async (): Promise<void> => {
  const w = await found()
  await w?.hide()
}

// The label is baked into the page at creation, so a second tear-off of a different tab needs
// the page told again. One message, no rebuild of the window.
export const setPreviewLabel = async (label: string): Promise<void> => {
  const w = await found()
  await w?.emit('preview-label', label)
}
```

and in `PreviewPage.vue`, listen for `preview-label` to replace the text — with the listener in
`lib/window/` the scanner would fail the page for importing the event API, so the page reads it
through a tiny `window.__previewLabel` callback set by `main.ts`, or `preview.ts` recreates the
window when the label changes. **Choose the second**: it is one window creation per distinct
tab dragged, it keeps `@tauri-apps/api` out of `src/preview/`, and it deletes a rule exception.
Replace `setPreviewLabel` with a `showPreview` that closes and recreates when the label differs:

```ts
let shownLabel: string | null = null
// …in showPreview, before reusing:
if (existing && shownLabel !== label) {
  await existing.close()
  preview = null
}
```

- [ ] **Step 4: Let the capability name it**

In `crates/app/capabilities/default.json`, add `"tab-preview"` to `"windows"`.

- [ ] **Step 5: Prove it on the machine**

Run `pnpm dev`; from the console, `showPreview('Unlock', { x: 600, y: 400 })`, then
`movePreview` a few times, then `hidePreview`.
Expected: a small borderless card appears at the cursor and moves; **the main window keeps the
focus** (type something into the search and watch it arrive); hiding it leaves no taskbar entry
behind. Write the creation time of the first show in the report, beside Task 1's number.

- [ ] **Step 6: Commit**

```bash
git add ui/preview.html ui/src/preview ui/src/lib/window/preview.ts ui/vite.config.ts crates/app/capabilities/default.json
git commit -m "feat(ui): a light preview follows the cursor outside the window"
```

---

### Task 15: The tab leaves, and lands

**Files:**
- Create: `ui/src/composables/useTabDrag.ts`
- Modify: `ui/src/components/shell/TabStrip.vue`
- Modify: `ui/src/App.vue` (the strip's new events reach the store)

**Interfaces:**
- Consumes: `useDragList`, `windowPort`, `focusOrder`, `tearOff`'s pure functions, `preview`,
  `watchPointer`, the store's `canTear`, `giveAway`, `openWindowWith`.
- Produces: `useTabDrag(options): { drag, detached, hovering }`.

- [ ] **Step 1: Write the composable**

`ui/src/composables/useTabDrag.ts`:

```ts
import { ref, shallowRef } from 'vue'
import type { Ref } from 'vue'
import { useDragList } from '@/composables/useDragList'
import type { DragList } from '@/composables/useDragList'
import { Axis, boxAt } from '@/lib/drag/dragList'
import type { Box, Point } from '@/lib/drag/dragList'
import { focusOrder } from '@/lib/window/focusOrder'
import {
  PreviewLabel,
  hidePreview,
  movePreview,
  showPreview,
} from '@/lib/window/preview'
import { watchPointer } from '@/lib/window/pointerSource'
import { pastTearBand, toDesktop, windowUnderPoint } from '@/lib/window/tearOff'
import { windowPort } from '@/lib/window/windowPort'
import type { WindowBox } from '@/lib/window/windowPort'
import { WindowMessageKind } from '@/lib/window/messages'
import { dropSide, moveIndex } from '@/components/shell/tabs'
import type { DropSide } from '@/components/shell/tabs'

export interface TabDrop {
  index: number
  side: DropSide
}

export interface TabDragOptions {
  strip: Ref<HTMLElement | null>
  items: () => HTMLElement[]
  labelOf: (index: number) => string
  canTear: () => boolean
  reorder: (from: number, to: number) => void
  // Hands the tab at `index` to that window, at that desktop point.
  giveTo: (index: number, label: string, at: Point) => Promise<void>
  // Opens a window holding the tab at `index`, its top-left at that desktop point.
  openWith: (index: number, at: Point) => Promise<void>
}

export interface TabDrag {
  drag: DragList<TabDrop>
  detached: Ref<boolean>
}

export const useTabDrag = (options: TabDragOptions): TabDrag => {
  const detached = ref(false)
  const windows = shallowRef<WindowBox[]>([])
  const self = shallowRef<WindowBox | null>(null)
  let stopPointer: (() => void) | null = null
  let hovered: string | null = null
  let index: number | null = null

  // The strip's own rectangle, read when the drag starts: the tear band is measured from it.
  const stripBox = (): Box | null => {
    const el = options.strip.value
    if (!el) return null
    const r = el.getBoundingClientRect()
    return { left: r.left, top: r.top, width: r.width, height: r.height }
  }

  const tellHovered = (label: string | null, at: Point) => {
    if (hovered && hovered !== label)
      void windowPort.send(hovered, { kind: WindowMessageKind.HoverLeft })
    hovered = label
    if (label) void windowPort.send(label, { kind: WindowMessageKind.Hovering, at })
  }

  // Outside the window the tab is a preview under the cursor, except over another window's
  // strip, where the marker in *that* strip says where it will land and the preview would only
  // cover it.
  const onOutsideMove = (p: Point) => {
    const target = windowUnderPoint(
      windows.value.filter((w) => w.label !== PreviewLabel),
      p,
      focusOrder.value,
    )
    const mine = self.value
    if (target && mine && target === mine.label && stripBox()) {
      // Back over our own strip: the tab comes home and the gesture is a reorder again.
      tellHovered(null, p)
      void hidePreview()
      detached.value = false
      stopPointer?.()
      stopPointer = null
      return
    }
    tellHovered(target, p)
    if (target) void hidePreview()
    else void movePreview(p)
  }

  const onOutsideRelease = (p: Point) => {
    const at = index
    stopPointer = null
    detached.value = false
    void hidePreview()
    const target = hovered
    tellHovered(null, p)
    if (at === null) return
    if (target) void options.giveTo(at, target, p)
    else void options.openWith(at, p)
    drag.cancel()
  }

  const detach = async (p: Point) => {
    if (!options.canTear() || index === null) return
    detached.value = true
    // The windows do not move during a drag, so one list is enough; the preview is not a
    // target and is filtered out wherever the list is read.
    const [all, mine] = await Promise.all([windowPort.list(), windowPort.self()])
    windows.value = all
    self.value = mine
    await showPreview(options.labelOf(index), toDesktop(p, mine))
    stopPointer = watchPointer(onOutsideMove, onOutsideRelease)
  }

  const resolve = (p: Point, boxes: Box[], from: number): TabDrop | null => {
    index = from
    const strip = stripBox()
    if (!detached.value && strip && pastTearBand(p, strip)) {
      void detach(p)
      return null
    }
    if (detached.value) return null
    const at = boxAt(boxes, p, Axis.X)
    const box = at === null ? undefined : boxes[at]
    if (at === null || at === from || !box) return null
    return { index: at, side: dropSide(p.x, box.left, box.width) }
  }

  const drag = useDragList<TabDrop>({
    axis: Axis.X,
    container: options.strip,
    items: options.items,
    resolve,
    commit: (from, landing) => {
      if (detached.value || !landing) return
      const to = moveIndex(from, landing.index, landing.side)
      if (to !== from) options.reorder(from, to)
    },
  })

  return { drag, detached }
}
```

Note what `toClient` is for here: it is the **receiving** window that converts, in Task 16. This
window only ever converts outwards, with `toDesktop`.

- [ ] **Step 2: Wire the strip to it**

In `TabStrip.vue`, replace the direct `useDragList` of Task 4 with `useTabDrag`, passing:
`strip`, `items: tabElements`, `labelOf: (i) => props.tabs[i]?.label ?? ''`,
`canTear: () => props.canTear`, `reorder: (from, to) => emit('move', from, to)`,
`giveTo: (i, label, at) => emit('giveTo', i, label, at)` and
`openWith: (i, at) => emit('openWith', i, at)` — two new emits, plus a `canTear` prop. The ghost
is rendered only while `!detached`: outside the window the preview has taken over.

In `App.vue`, the two new events call the store's `giveAway` and `openWindowWith`, with the tab's
id and the window's own size read through `windowPort.self()`.

- [ ] **Step 3: Look at it on the development server first**

Run: `pnpm ui:dev` with `?fixture=active&windows=fake`.
Expected: dragging a tab down past the band logs `[windows] create` (the fake's line) and leaves
the strip alone; dragging within the strip still reorders; nothing throws. The fake proves the
wiring, not the gesture.

- [ ] **Step 4: The real one**

Run: `pnpm dev`. Drag a tab down out of the window and release on the empty desktop.
Expected: the preview follows the cursor from the moment the tab clears the band; on release a
window opens there, holding that tab, sized like the origin, and the tab is gone from the first
window. Then: drag a tab out and bring it **back** over its own strip before releasing — the
preview disappears, the ghost returns and the release reorders, exactly as if it had never left.

- [ ] **Step 5: Commit**

```bash
git add ui/src/composables/useTabDrag.ts ui/src/components/shell/TabStrip.vue ui/src/App.vue
git commit -m "feat(ui): a tab dragged out of the window opens one of its own"
```

---

### Task 16: The strip you are about to drop onto says so

**Files:**
- Modify: `ui/src/lib/window/session.ts`
- Modify: `ui/src/components/shell/TabStrip.vue`
- Modify: `ui/src/stores/tabs.ts`

**Interfaces:**
- Consumes: `HoveringMessage`, `HoverLeftMessage`, `toClient`, `boxAt`, `dropSide`,
  `windowPort.self()`.
- Produces: on the store, `incoming: Ref<number | null>` — the index a tab arriving from
  another window would land at, or null.

- [ ] **Step 1: Turn a hover into an index**

In `session.ts`, fill the two remaining arms:

```ts
      case WindowMessageKind.Hovering:
        void tabs.aimIncoming(m.at)
        return
      case WindowMessageKind.HoverLeft:
        tabs.clearIncoming()
        return
```

and in `stores/tabs.ts`:

```ts
  // Where a tab arriving from another window would land, recomputed as it moves over our strip.
  // The point comes in desktop pixels because the sender cannot know our scale factor; this is
  // the only place that converts, and it is the window that owns the factor.
  const incoming = ref<number | null>(null)

  const aimIncoming = async (at: Point): Promise<void> => {
    const mine = await windowPort.self()
    const p = toClient(at, mine)
    const boxes = stripBoxes()
    const index = boxAt(boxes, p, Axis.X)
    const box = index === null ? undefined : boxes[index]
    incoming.value =
      index === null || !box
        ? state.value.tabs.length
        : dropSide(p.x, box.left, box.width) === DropSide.Before
          ? index
          : index + 1
  }

  const clearIncoming = (): void => {
    incoming.value = null
  }
```

`stripBoxes()` reads the same `[role="tab"]` elements the strip reads. To keep the DOM out of the
store, the strip registers a reader with it on mount (`tabs.useStripReader(() => …)`) — a
function property, set once, cleared on unmount.

- [ ] **Step 2: Draw it**

`TabStrip.vue` takes an `incoming: number | null` prop and draws the same 2 px `bg-primary`
marker the queue uses, in the gap at that index — including the gap after the last tab. It is
drawn whether or not a local drag is in flight: the two cannot both be true in one window.

- [ ] **Step 3: Dock where the marker was**

In `session.ts`'s `Docked` arm, replace Task 9's `tabs.tabs.length` with the aimed index:

```ts
      case WindowMessageKind.Docked:
        void tabs.dockAt(m.tab, m.at)
        return
```

where `dockAt` converts the point the same way `aimIncoming` does, inserts there, and clears
`incoming`. One conversion, one rule, one place.

- [ ] **Step 4: Prove it on the machine**

Run `pnpm dev`, tear a tab off to make a second window, then drag a tab from one window over the
other's strip.
Expected: the marker appears **between the target's tabs** and follows the cursor along the
strip; the preview hides while it is there; the release lands the tab exactly where the marker
was, the target takes the focus, and the origin closes if that was its last tab. Move the cursor
off the strip: the marker disappears and the preview comes back.

- [ ] **Step 5: `pnpm check` and commit**

```bash
git add ui/src/lib/window/session.ts ui/src/components/shell/TabStrip.vue ui/src/stores/tabs.ts
git commit -m "feat(ui): the strip under the cursor shows where the tab will land"
```

---

## Phase 4 — the proof

### Task 17: The verification on the machine

Fixtures run in one browser tab and Vitest opens no windows: **this is the verification**, and
nothing in this sub-project is done until it is done. Every line is checked separately and
written into the report — a line that fails is a finding, not a reason to stop.

**Files:**
- Modify: `docs/superpowers/reports/2026-09-13-drag-and-windows-report.md`

- [ ] **Step 1: `pnpm check`, read including the skips**

Run: `pnpm check`
Expected: green. Read the `skip:` lines: an "N passed" does not say how many were skipped.

- [ ] **Step 2: Run the app and work the list**

Run: `pnpm dev`. For each line, write what happened:

1. tear a tab off onto the empty desktop: a window opens under the cursor, sized like the origin
2. drag it back over the first window's strip: the marker appears between the tabs, the release
   merges it there
3. dock into a **second** secondary window, not only into `main`
4. the last tab of a window dropped on the **desktop**: a no-op, nothing opens, nothing closes
4b. the last tab of a **secondary** window dropped on another window's **strip**: it joins, and
    the window it left closes — the two gestures are not the same rule (spec §5)
5. a secondary window whose last tab is docked elsewhere closes; `main` keeps a fresh tab
6. two windows on the Plan: a move in one is visible in the other
7. change the profile in one window: the other's indicator and screens follow
8. change the scale in one window: the other follows
9. two monitors at different scale factors: the drop lands where the cursor is, not offset
10. the target window closed **while** the tab is in flight: the tab stays where it was, no crash
11. cold `pnpm dev`: how long the first tear-off's preview takes, and the second one

- [ ] **Step 3: Write the report**

Fill "Verification on the machine" with the eleven answers, the numbers from Tasks 1, 8 and 14,
and — separately — **what is still not verified**: whether a second monitor was actually
available, what the gesture does on a machine with no Windows (it degrades to the DOM source and
nobody has run it), and anything the eleven lines did not cover.

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/reports/2026-09-13-drag-and-windows-report.md
git commit -m "docs: the drag and the windows, verified on the machine"
```

---

### Task 18: The documents say what happened

**Files:**
- Modify: `docs/STATUS.md`
- Modify: `docs/BACKLOG.md`
- Modify: `DESIGN-BRIEF.md`
- Modify: `CLAUDE.md`

- [ ] **Step 1: STATUS**

A dated section in the session log: what landed, the spike's answer, the eleven checks, and what
is open. Tick the boxes that are **committed work**, never a note.

- [ ] **Step 2: BACKLOG**

Mark **B31** and **B15** closed with their date, in the form the other closed entries use
(`✅ closed on 2026-09-13`), each with one line saying what closed it. If any part was left
open — the plugin never evaluated, a check that could not be run — it stays as its own entry
rather than being buried in a closed one.

- [ ] **Step 3: DESIGN-BRIEF §4**

The contract gained a second kind of window. Say what a secondary window is (the whole app), what
the tear-off gesture is, and that the active profile and the interface scale are the app's, not
the window's — §4.1 and §4.2 currently say the opposite, in as many words, and leaving them is a
documented lie.

- [ ] **Step 4: CLAUDE.md**

Two lines, where the modules and the state are described: `ui/src/lib/window/` is the only place
that talks to windows and events, and one composable drags every list. Keep it to the rule, not
the story.

- [ ] **Step 5: `pnpm check`, then commit**

```bash
git add docs/STATUS.md docs/BACKLOG.md DESIGN-BRIEF.md CLAUDE.md
git commit -m "docs: B31 and B15 land, and a window is no longer the app"
```

- [ ] **Step 6: Hand it on**

The IPC contract is live and someone is on the other end. The events of Task 10 and the second
window of Task 18 are a change to material a design is being built on: **hand them on, do not
merely commit them**. Then invoke `superpowers:finishing-a-development-branch` and decide the
merge into `develop` with the owner — `develop` is checked out in another worktree, so the merge
is run from there, or by whoever holds it.

---

## Self-review of this plan

**Spec coverage.** §1 → Tasks 2–5. §2 → Tasks 1 and 13. §3 → Tasks 12, 14, 15. §4 → Tasks 6, 8.
§5 → Tasks 7, 9, 12, 16. §6 → Task 10. §7 → Tasks 6, 11. §8 → Task 18 (the shape is written in
the spec itself and needs no code). The Files section of the spec matches the files named here,
with one addition the spec did not foresee: `lib/window/focusOrder.ts`, which exists because the
spec's "most recently focused wins" rule needed somewhere to live and Rust turned out not to be
that place — the frontend hears its own focus events already, so no command was added.

**Where this plan knowingly departs from the spec.** The spec put the focus order in
`crates/app` as a `Vec<String>`. Task 9 keeps it in the frontend instead: every window already
watches its own focus through `appWindow.ts`, so a broadcast costs nothing and a Rust command,
its permission and its state are all avoided. The rule it implements is unchanged.

**Type consistency.** `Point` and `Box` come from `lib/drag/dragList.ts` everywhere, including
the window modules. `WindowBox` is `windowPort.ts`'s and carries `scaleFactor`, which `toClient`
needs. `TabSeed` is `{ location }` in the messages, in the store and in the model. `DropSide`
and `moveIndex` keep their names from `components/shell/tabs.ts`; `DropEdge`, `dropAnchor` and
`stepAnchor` keep theirs from `lib/plan/queueDrop.ts`.

**What no task covers, deliberately.** Persistence (3.7/B6), a tab dragged into another
application, dragging a whole window onto another to merge, the sidebar's resize, and
`tauri-plugin-drag-as-window` unless Task 1 closes both other doors.
