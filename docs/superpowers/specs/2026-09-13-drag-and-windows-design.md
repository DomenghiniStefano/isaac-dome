# One drag everywhere, and a tab that tears off into its own window (design)

**Date:** 2026-09-13
**Milestone:** design system (`docs/STATUS.md`), cycle 3 of 3 — not a numbered sub-project: it
closes two backlog entries that meet in the same gesture, **B31** (the lifted ghost and one
drag component) and **B15** (tearing a tab off into its own window, and back)
**Branch:** `feature/drag-and-windows`, cut from `develop`, in its own worktree
(`C:/Projects/isaac-dome-drag`) so the wiki infobox work on `feature/wiki-infobox` keeps its
checkout
**Depends on:** 3.1 (tabs own locations, `stores/tabModel.ts`; the live chrome,
`lib/window/appWindow.ts`), 3.3b (the queue you drag, `lib/plan/queueDrop.ts`), 3.5c (the
interface scale, which every window has to agree on); `DESIGN-BRIEF.md` §4.1, §4.2
**Ordered before 3.6 and 3.7**, against B15's own note. B15 said "after 3.7" so that B6's
persisted shape would be decided once, knowing tear-off. Taken the other way round, 3.7 is the
one that gets written knowing the shape, and this spec hands it that shape (§8). Nothing in
3.6 or 3.7 is undone by it.
**Owner's answers (2026-09-13 brainstorm), marked (owner) below:** the full tear-off gesture in
this cycle, not the structure alone; a torn-off window is the whole app, not tabs and content;
a light preview follows the cursor outside the window; the payload-free event rule.
Everything else is the author's, marked **(delegated)**, so the first look can overturn it
cheaply.

## What this is

Two things the owner asked for, which share one piece of machinery.

**B31.** Dragging a queue row does not feel right: the row is only dimmed in place and a line is
drawn in the gap, so the eye has nothing to follow. The owner wants the whole card lifted,
floating above the page, landing exactly where the red marker says. And since two screens
already drag — the queue and the tab strip — the lifting should be **one component**, not a
third copy of the same pointer choreography.

**B15.** Drag a tab out of the window and it opens in a window of its own; drag it back over
another window's strip and the two merge — exactly as a browser does.

They meet because the tear-off *is* the tab strip's drag, continued past the strip's edge. Doing
B15 on top of the two hand-written drags would write the choreography a third time; doing B31
first means the tear-off is a branch inside one composable.

## Applicable constraints

1. **Nothing crosses the IPC but resolved view-models.** No paths, no offsets, no `Debug`
   strings. This spec adds an event channel, and the rule holds there too (§6).
2. **Degrade, never fail.** A drag that loses its pointer, a target window closed mid-gesture, a
   webview that never signals ready: each ends with the tab still somewhere, never lost.
3. **`crates/app` is wiring, and wiring is not tested.** Anything with a return value worth
   checking is a pure function in `ui/src/lib/` or a pure crate — the hit test, the coordinate
   conversion, the tab reducers.
4. **The frontend rules hold.** No `<style>` in SFCs (the ghost's position is a CSS variable
   bound by the template, not inline pixels), no visual constants, no `invoke()` in components,
   no string unions, and the new modules pass `pnpm scan`.
5. **The tab model stays pure.** `stores/tabModel.ts` gains reducers; it keeps knowing nothing
   about windows, Tauri, or the DOM.
6. **The bar is never empty**, one level up: a window always has at least one tab, and the rule
   that protects it is the same one that refuses to tear off the last one (§5).

---

## Decision 1 — the drag becomes three pieces (delegated)

`TabStrip.vue` and `QueueCard.vue` hold the same gesture twice: a press becomes a drag past a
threshold, *only then* is the pointer captured, the rectangles are read once, a drop is
recomputed on every move, and nothing moves until the release. It splits in three.

**`ui/src/lib/drag/dragList.ts` — pure, Vitest.**

```ts
export const Axis = { X: 'x', Y: 'y' } as const
export type Axis = (typeof Axis)[keyof typeof Axis]

export interface Point { x: number; y: number }
export interface Box { left: number; top: number; width: number; height: number }

// A press becomes a drag only past the threshold, on the axis the list runs along.
export const crossedThreshold = (axis: Axis, from: Point, to: Point, threshold: number): boolean

// Which of the snapshotted rectangles holds the point, or null between them.
export const boxAt = (boxes: Box[], p: Point, axis: Axis): number | null

// Where inside the grabbed row the press landed, and the ghost's top-left that keeps it there.
export const grabOffset = (box: Box, press: Point): Point
export const ghostOrigin = (offset: Point, p: Point): Point
```

**`ui/src/composables/useDragList.ts` — the DOM shell.** Owns the capture, the rectangle
snapshot, `pointercancel`, and `Escape` — which today is missing in both screens: a drag, once
started, cannot be called off. It takes the two pure functions the caller brings: where the
items are, and what a drop at index *i* on side *s* means. `dropSide`/`moveIndex` and
`dropEdge`/`dropAnchor` stay where they are — they are the semantics of *that* list, not of the
gesture.

**`ui/src/components/ui/drag/DragGhost.vue`.** `Teleport` to the body, `position: fixed`, the
kit's raised shadow, no transition, content through a slot so the tab and the row draw
themselves. Position and size arrive as CSS variables bound by the template.

What does not change: the original stays dimmed in its slot, the marker keeps naming the
landing, the drop is instant and named by `move_after`, and every keyboard move still works —
the ghost is feedback, not the mechanism.

**Out of scope: the sidebar's resize.** Same pointer choreography, no list and no drop. If B27
generalises it for the tables it gets its own composable.

## Decision 2 — the spike comes first, and it decides one module (owner: full gesture; delegated: the spike)

B15 names two traps, and only one of them is settled by reading. `startDragging()` cannot be
started on a window that did not receive the mousedown, so a torn-off window is moved by
`setPosition` from the origin — that is decided. The other is open: **WebView2's
`setPointerCapture` is unreliable once the cursor leaves the control** (microsoft-ui-xaml #8677,
#8753), and whether `pointermove` and `pointerup` still arrive can only be measured on this
machine.

So task 0 is a spike under `pnpm dev`, a real window, with a mouse, and it answers three
questions:

1. With the pointer captured, does the origin window still receive `pointermove` outside its
   bounds, and does it receive the `pointerup` that ends the drag there?
2. How long does creating a `WebviewWindow` take, from the call to the first paint?
3. Can a window be created and shown **without taking the focus** (`focused: false`), leaving
   the origin's capture intact?

The answer to (1) picks the implementation of **one module**, `ui/src/lib/window/pointerSource.ts`:

```ts
export interface PointerSource {
  onMove: (handler: (p: Point) => void) => void   // desktop physical pixels
  onRelease: (handler: (p: Point) => void) => void
  stop: () => void
}
```

- **If the events arrive**: the DOM source. `cursorPosition()` is still used to convert to
  desktop coordinates, no Rust at all.
- **If they do not**: a Rust source. A thread in `crates/app`, alive only while a drag is,
  polling `GetCursorPos` and `GetAsyncKeyState(VK_LBUTTON)` at ~120 Hz behind
  `#[cfg(windows)]`, emitting `drag-moved` and `drag-released` to the origin window. On any
  other platform it degrades to the DOM source, and the tear-off is what that platform's
  webview allows.
- **`tauri-plugin-drag-as-window` stays prior art, not a dependency.** It would hand the drag
  loop to the OS, but it brings a third-party crate to vet (maintenance, licence against our
  GPL-3.0-only, behaviour with `decorations: false`) and its feedback is an OS drag image, not
  a live window. It is the third choice, taken only if both of the above fail — and then only
  after it has been made to answer on the machine.

Everything downstream of `PointerSource` is written once and does not change with the answer.

## Decision 3 — outside the window, a light preview follows the cursor (owner)

Inside the strip the gesture is today's reorder, with B31's ghost. Past a band of 24 logical
pixels beyond the strip the drag becomes **detached**: the DOM ghost is hidden and a preview
window appears under the cursor.

The preview is a **second, minimal page** — `ui/preview.html`, one more entry in Vite's
`rollupOptions.input` — carrying the tab's label on a card and nothing else: no router, no
store, no i18n, no IPC. Its window is `decorations: false`, `alwaysOnTop`, `skipTaskbar`,
`shadow: false`, `resizable: false` and **`focused: false`**: a preview that took the focus
would cost the origin its pointer capture and kill the gesture in the act.

It is created at the first tear-off of the session and afterwards only hidden and shown again,
so every drag after the first is instant. The label travels in the URL's query — a tab's title
is not a path, an offset or a profile, and nothing else goes with it.

**The DPI trap.** `cursorPosition()` answers in the desktop's physical pixels, the DOM thinks in
logical ones, and two monitors can disagree on the factor. The conversion is a pure function,
tested, not a `devicePixelRatio` sprinkled through the composable:

```ts
export const toClient = (p: Point, outer: Point, scaleFactor: number): Point
export const toDesktop = (p: Point, outer: Point, scaleFactor: number): Point
```

## Decision 4 — windows are born through a handshake, not through a URL (delegated)

Labels are `main` and `win-<base36 timestamp>`. One capability covers them: `"windows"` becomes
`["main", "win-*"]`, and the permissions gain
`core:webview:allow-create-webview-window`, `core:window:allow-cursor-position`,
`core:window:allow-set-position`, `core:window:allow-set-focus`,
`core:window:allow-set-always-on-top`, `core:window:allow-show`, `core:window:allow-hide`.

A new window is created with the app's own `index.html` and **no state in its URL**. It signals
`window-ready`; the origin answers `tabs-seed` with the locations it is to hold and which is
active. `useTabsStore` starts empty when its label is not `main` and waits for the seed; on
`main` it starts as it does today. A seed that never comes (the origin died mid-handshake)
leaves the window with a default tab after a timeout: a window with a bar and no tabs is the one
state the rules forbid.

A torn-off window opens **under the cursor at the release, sized like the origin** — the size
the user already chose, in the place they dropped it.

**A secondary window is the whole app** (owner): title bar, strip, navbar, sidebar, the "+"
button. It can open tabs, search, and navigate like the first. The only asymmetry is §5's.

## Decision 5 — docking, and the two closing rules (delegated)

**Docking.** On release over another window, the origin emits `tab-docked { location, at }` —
`at` in desktop coordinates — to that window's label. The target converts the point with
`toClient`, finds the insertion index with `dropSide`, the same pure function the reorder uses,
inserts the tab, selects it and takes the focus. Only then does the origin drop its own tab.
The order matters: if the target never answers, the tab is still in the origin.

**Which window is under the point.** `getAllWebviewWindows()` gives position, size, visibility
and whether a window is minimized. There is **no z-order API** (tauri#5656), so two overlapping
windows are ambiguous and the rule is ours: a window whose **strip band** contains the point
wins over one whose body does; among equals, the one focused most recently. The focus order is a
`Vec<String>` in `crates/app`, pushed on the window focus event — wiring. The decision is pure
and tested:

```ts
export const windowUnderPoint = (windows: WindowBox[], p: Point): string | null
```

A point over no window at all means a new window.

**Live re-docking.** While the preview passes over another window's strip, that window receives
the point (throttled to one animation frame) and draws the insertion marker between its own
tabs, and the preview hides. You see where the tab will land before you let go. No grace period:
a grace period is what you need when hovering *merges by itself* — here the release merges, and
a release is already an intention.

**A tab in flight belongs to no window (owner, 2026-09-13).** The owner overturned both closing
rules as first written, and the new shape is simpler than either:

- **Any tab can be torn off, the last one included.** "That window already is that tab" was an
  argument, not a requirement, and the owner does not want the gesture refused.
- **The tab leaves the strip the moment it is torn off**, not when it is dropped. What you are
  dragging is no longer in the bar — which is what a browser does, and what makes the rest of
  this section fall out rather than be legislated.

So a tab is in exactly one place at a time: in a strip, or in flight. Three endings dispose of
it, and only these three:

| ending | what happens |
|---|---|
| dropped on a strip (**including its own**) | it lands where that strip's marker says |
| dropped on the bare desktop | a window of its own, there, sized like the one it left |
| cancelled — `Escape`, a lost pointer, a failure mid-gesture | it goes back to the exact index it sat at |

**A window left empty closes**, except the first one, which keeps a fresh landing tab exactly as
closing its last tab already does. The app exits when every window is closed, which is Tauri's
own behaviour and needs nothing from us.

What this removes: `canDetach` as a gate on the gesture, the special case for coming back over
your own strip (it is now a landing like any other), and the duplication that the two-sided
hand-over invited — the origin no longer has to *remember* to remove a tab the target has
already taken, because it gave it up before anyone else could hold it.

## Decision 6 — events from Rust carry no payload (owner)

With two windows the active profile stops being the window's (`DESIGN-BRIEF.md` §4.1, §4.2) and
becomes the app's. So does the interface scale, and so does the plan's queue: two windows on the
Plan, one moves a row, the other has to see it.

The rule:

> **An event emitted by Rust carries no payload. It says "read again".**

`profile-changed`, `settings-changed` and `plan-changed` are emitted by the command that has
just written — `select_profile`, `set_scale`, and every command that touches the plan
(`queue_add`, `queue_remove`, `queue_move`, `queue_import_goals`, `add_goal`, `remove_goal`:
the queue and the goals are one screen, so they are one event) — to every window. Each window
reloads its own store through the commands
it already calls, and `useOnActiveProfile` propagates to every screen that reads the save.

Two things this buys. No second contract: nothing new is serialized across the IPC boundary, so
no path, no id and no `Debug` string can leak through a channel that has no view-models. And no
drift: a payload would be a copy of state that the next command could contradict.

What it costs is a re-read per window per change. With three windows and a 400 KB save that is
nothing; a continuous stream would be a reason to revisit it, and there is none in the app.

**Messages between windows** — `window-ready`, `tabs-seed`, `tab-docked`, `tab-hovering` — carry
frontend types, declared in `ui/src/lib/window/messages.ts`. They never pass through Rust and are
not the IPC contract.

## Decision 7 — one module talks to windows, and it has a fake (delegated)

`lib/window/appWindow.ts` is already the only module that talks to the window, and the scanner
says so. It grows into a small family, and the rule is extended, not broken:

| module | what it owns |
|---|---|
| `lib/window/appWindow.ts` | the current window: controls, focus, position, scale factor |
| `lib/window/windowPort.ts` | the other windows: list, create, emit to, listen |
| `lib/window/pointerSource.ts` | where the pointer is while it is outside (§2) |
| `lib/window/tearOff.ts` | pure: `windowUnderPoint`, `toClient`, `toDesktop`, the strip band |
| `lib/window/messages.ts` | the window-to-window message types |

`windowPort.ts` has a fake behind `?windows=fake`, so the whole gesture can be exercised on the
development server — two invented windows, drawn as boxes, that accept a dock. It is not the
verification (§9); it is what makes the work watchable while it is being written.

## Decision 8 — what this hands to 3.7 (delegated)

3.7 (B6) saves the tabs. This spec does not persist anything, and it fixes the shape so 3.7 does
not have to decide it twice:

> A session is **windows of tabs**: an ordered list of windows, each with its tabs, its active
> one, and its position and size. `main` is the first and is restored first.

## Files

**New.** `ui/src/lib/drag/dragList.ts` (+ test), `ui/src/composables/useDragList.ts`,
`ui/src/components/ui/drag/DragGhost.vue`, `ui/src/composables/useTabDrag.ts`,
`ui/src/lib/window/{windowPort,pointerSource,tearOff,messages}.ts` (+ test for `tearOff`),
`ui/preview.html` + `ui/src/preview/`, and — only if the spike says so — `crates/app/src/cursor.rs`.

**Changed.** `ui/src/components/shell/TabStrip.vue`, `ui/src/screens/plan/QueueCard.vue`,
`ui/src/components/shell/TabItem.vue` and `ui/src/screens/plan/QueueRow.vue` (the ghost's
content), `ui/src/stores/tabModel.ts` (+ test: insert at index, detach, the last tab),
`ui/src/stores/tabs.ts`, `ui/src/kit/sections/`, `ui/vite.config.ts`,
`crates/app/capabilities/default.json`, `crates/app/src/lib.rs` (the focus order and the
window events), `crates/app/src/commands/{profile,queue,plan}.rs` (the emits),
`ui/scripts/scan-conventions.mjs`: its "window API outside `src/lib/window/`" rule matches
`@tauri-apps/api/window` only, and the two namespaces this spec introduces —
`@tauri-apps/api/webviewWindow` and `@tauri-apps/api/event` — would walk straight past it. The
rule is widened to all three, otherwise the document promises a check that never happens.

## Work order

0. **The spike** (§2) — under `pnpm dev`, on the machine. Its answer is written into the report
   whatever it is: a negative result is a result.
1. **B31** — the pure module, the composable, the ghost, the two callers, the Kit section.
   Landable and reviewable on its own: at the end of this step the queue and the strip drag
   through one piece of code and nothing else has changed.
2. **The structure** — labels and capability, `windowPort`, the handshake, docking, the closing
   rules, the payload-free events. Verifiable with a window opened by a command, before any
   drag reaches it.
3. **The gesture** — the tear band, the preview window, `pointerSource`, the hit test, live
   re-docking.
4. **Verification on the machine and the documents** — §9, then `docs/STATUS.md`, B15 and B31
   closed in `docs/BACKLOG.md`, `DESIGN-BRIEF.md` §4 for the secondary window, and
   `docs/frontend-conventions.md` for the one-drag rule.

## Verification

**Vitest**, on what has a value worth checking: `dragList`'s threshold, hit and offset;
`tearOff`'s `windowUnderPoint` with overlapping windows, a minimized one and a point over
nothing; `toClient` / `toDesktop` at scale factors 1 and 1.5; `tabModel`'s new reducers,
including the last tab that does not tear off.

**On the development server**, with `?windows=fake`: the ghost on the Kit, the reorder in the
strip and the queue unchanged, a dock into an invented window.

**On the machine, by hand — this is the verification** (B15's own constraint: fixtures run in
one browser tab). The list goes into the report, each line ticked or explained:

- tear a tab off onto the empty desktop: a window opens under the cursor, sized like the origin
- drag it back over the first window's strip: the marker appears between the tabs, the release
  merges it there
- dock into a *second* secondary window, not only into `main`
- the last tab of a window: the drag is a no-op, nothing opens
- a secondary window whose last tab is docked elsewhere closes; `main` keeps a fresh tab
- two windows on the Plan: a move in one is visible in the other
- change the profile in one window: the other window's indicator and screens follow
- change the scale in one window: the other follows
- two monitors at different scale factors: the drop lands where the cursor is
- the target window closed *while* the tab is in flight: the tab stays where it was, no crash
- `pnpm dev` cold: the first tear-off's preview, and the second one's speed

**`scripts/check` green** before anything is called done, with `--nocapture` so the skips are
counted.

## What this does not do

- **No persistence** (3.7/B6): windows and their tabs are gone at exit. §8 is the shape, not the
  code.
- **No tab dragged into another application**, and no window dragged onto a window to merge:
  only a tab leaves and only a tab returns.
- **No sidebar resize** in the shared composable (§1).
- **No third-party drag plugin** unless §2's spike closes both other doors.
