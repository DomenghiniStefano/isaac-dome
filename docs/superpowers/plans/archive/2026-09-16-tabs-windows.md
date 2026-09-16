# 3.7b — windows of tabs: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:executing-plans` to implement
> this plan task by task. Steps use checkbox (`- [ ]`) syntax for tracking. This repo's plans are
> executed **inline**, not one-subagent-per-task.

**Goal:** the session stops being "main's tabs" and becomes **windows of tabs**. Two windows
closed and the app reopened come back as two windows, in their places, holding what they held —
and the session keeps being written after `main` is closed, which today it silently stops doing.

**Architecture:** every window broadcasts what it holds and where it is; every window keeps the
same ledger; **the one that writes is elected by a pure function over the live labels** — `main`
when it is there, the oldest surviving label otherwise. The document becomes version 2, whose top
level says `windows` instead of `tabs`, and a version 1 document is read as one window with no
box. Restoring is the tear-off's own machinery run against the document: `main` is born, seeds
itself from `windows[0]`, then creates one window per remaining entry and owes each of them its
seed. The stored geometry is clamped onto a monitor that exists, by a pure function.

**Tech Stack:** Vue 3 + TypeScript, Pinia, Vitest, `@tauri-apps/api` 2.11. Frontend only —
3.7b touches no Rust, no `crates/`, no IPC command and no SQL. §12 of the spec says so and this
plan does not move it.

**Spec:** `docs/superpowers/specs/2026-09-15-tabs-session-design.md` (§5, §6, §11 item 2).

## Global Constraints

Copied from `CLAUDE.md` and `docs/frontend-conventions.md`; every task's requirements include
them.

- **Test-first.** The expected value comes from the spec, never from the code's current output.
  A failing test is first of all a hypothesis of a bug in the code.
- **No `<style>` in SFCs**; dynamic values arrive as CSS variables bound by the template.
- **No hardcoded visual constants.** 3.7b **adds no token**: the only numbers it introduces are
  *window geometry* — a cascade step and a debounce — which live as named constants in
  `lib/window/`, beside `StripBand` and `TearBand`, and never in a template.
- **No `invoke()` in components**; nothing outside `ui/src/lib/window/` imports
  `@tauri-apps/api`'s `window`, `webviewWindow` or `event`. **Every module there degrades
  outside Tauri instead of throwing** — including with no DOM at all, which is where 3.7a found
  `fakeWindows` dying before its first line.
- **No raw `<button>` / `<input>`** — the primitives in `ui/src/components/ui/`.
- **No string unions** — `const X = { … } as const`, never `type X = 'a' | 'b'`.
- **No visible string in a template**: every sentence is an i18n key, in `it.ts` **and** `en.ts`.
- **Exhaustiveness**: no catch-all arm on a closed union; `assertNever` instead.
- `pnpm scan` enforces the five rules and has no exemption added by this plan.
- **Commits**: Conventional Commits, `type(scope): subject`, scope `ui`. No `Co-Authored-By`
  trailer, no reference to Claude, ever.
- **Before declaring anything done**: `pnpm check` (`scripts/check`) from the worktree root.

## File Structure

**Created**

| file | responsibility |
|---|---|
| `ui/src/lib/window/sessionWriter.ts` | `mintedAt`, `windowOrder`, `electWriter` — who writes, pure over the live labels |
| `ui/src/lib/window/sessionWriter.test.ts` | its tests |
| `ui/src/lib/window/monitorClamp.ts` | `clampToMonitors` — a stored box onto a monitor that exists, pure |
| `ui/src/lib/window/monitorClamp.test.ts` | its tests |
| `ui/src/lib/window/sessionHealth.ts` | the one write error that is not swallowed, held where a screen can read it |

**Changed**

| file | change |
|---|---|
| `ui/src/lib/window/sessionDocument.ts` | version 2: `windows`, the stored box, and version 1 read as one window |
| `ui/src/lib/window/sessionDocument.test.ts` | the new shape, and the old one still opening |
| `ui/src/lib/window/messages.ts` | `Holding` and `Closing` |
| `ui/src/lib/window/windowPort.ts` | `labels()` (the roster) and `monitors()` |
| `ui/src/lib/window/fakeWindows.ts` | the two new calls, degrading |
| `ui/src/lib/window/appWindow.ts` | `watchWindowBox` — moved and resized |
| `ui/src/lib/window/session.ts` | the ledger, the election, the broadcast, the restore, the one unswallowed error |
| `ui/src/components/shell/AboutDialog.vue` | the session's health, when it has stopped being written |
| `ui/src/i18n/messages/{it,en}.ts` | its two keys |

**Unchanged, and worth saying.** `crates/store` (migration 3 already holds this), `crates/ipc`'s
session settings, `crates/app/src/commands/session.rs`, and every Tauri command. `stores/tabs.ts`
and `stores/tabModel.ts` do not move either: a window's tabs are already a `Session`, and 3.7b
only puts several of them in a list.

## The decisions this plan takes, and why

Five things the spec delegates here.

1. **The roster is not `windowPort.list()`.** `list()` exists for the hit test and *filters out
   windows that are hidden or minimized* — which is right for "where can a tab land" and wrong
   for "which windows are open". A minimized window is still part of the session, and writing a
   document without it would lose it on the next restart. So `labels()` is a second call,
   deliberately not the first one narrowed.
2. **The election is a pure function of the label set, not of who joined first.** `main` when
   present; otherwise the label whose mint time is smallest, since `newWindowLabel()` is
   `win-<base36 of Date.now()>`. Every window then elects the same writer from the same input
   without any window having to be told anything, which is what makes it testable in Vitest and
   not "a thing you find out about in a window".
3. **Correctness comes from reconciling, promptness from a message.** Before writing, the elected
   window drops from its ledger every label that is no longer open. The `Closing` broadcast is
   what makes a survivor write *now* rather than at the next tab change; if it is lost to the
   teardown race, the next write is still correct. The spec's rule that a window does not *write*
   during its own teardown is untouched.
4. **A write is postponed while a live window has not yet said what it holds.** Writing a
   document that is missing a window is worse than writing a beat later, and the wait is bounded:
   every window broadcasts as soon as it is seeded, and seeding has a 700 ms deadline that always
   fires.
5. **The box is stored in desktop physical pixels**, which is what `WindowBox` already reports
   and what `create` takes for its position. The *size* `create` takes is logical, so the clamp
   answers the scale factor of the monitor the window lands on and the caller divides. Getting
   this backwards opens a window twice the size it had on a scaled screen — the same trap
   `windowSize()` already carries a comment about.

---

### Task 1: the roster, and the monitors

**Files:**
- Modify: `ui/src/lib/window/windowPort.ts`, `ui/src/lib/window/fakeWindows.ts`

**Interfaces:**
- Produces: `MonitorArea`; `WindowPort.labels(): Promise<string[]>`;
  `WindowPort.monitors(): Promise<MonitorArea[]>`.

There is no test here: this is the boundary module, whose whole job is to talk to Tauri. What
depends on it is pure and is tested in Tasks 2 and 3.

- [ ] **Step 1: `MonitorArea` and the two calls**

In `windowPort.ts`:

```ts
// A monitor as the clamp needs it: its **work area** — the screen minus the taskbar — in desktop
// physical pixels, and the factor that turns a physical size into the logical one
// `new WebviewWindow` takes. `size` is deliberately not used: a window restored under the
// taskbar is a window whose title bar you cannot grab.
export interface MonitorArea {
  left: number
  top: number
  width: number
  height: number
  scaleFactor: number
}
```

Add to the interface and to `tauriPort`:

```ts
  // **Every window there is, including the ones `list()` hides.** `list()` answers the hit test,
  // so it drops what is minimized or invisible; a minimized window is still part of the session,
  // and a roster that forgets it writes a document that loses it.
  labels: async () => (await getAllWebviewWindows()).map((w) => w.label),
  // Primary first, which is the one a box with nowhere to go lands on.
  monitors: async () => {
    const [all, primary] = await Promise.all([availableMonitors(), primaryMonitor()])
    const areas = all.map(areaOf)
    const at = primary ? areas.findIndex((a) => a.left === primary.workArea.position.x && a.top === primary.workArea.position.y) : -1
    return at <= 0 ? areas : [areas[at]!, ...areas.filter((_, n) => n !== at)]
  },
```

with `areaOf(m: Monitor): MonitorArea` reading `m.workArea.position`, `m.workArea.size` and
`m.scaleFactor`.

- [ ] **Step 2: the fake answers both**

`fakeWindows` gains `labels: () => Promise.resolve(on ? ['main', 'win-fake'] : ['main'])` and one
monitor, `{ left: 0, top: 0, width: 1920, height: 1080, scaleFactor: 1 }`. It must not read
`window` outside the guard that is already there.

- [ ] **Step 3: Verify** — `pnpm typecheck`, `pnpm lint`, `pnpm scan`.

---

### Task 2: who writes is elected

**Files:**
- Create: `ui/src/lib/window/sessionWriter.ts`, `ui/src/lib/window/sessionWriter.test.ts`

**Interfaces:**
- Produces: `mintedAt(label: string): number | null`;
  `windowOrder(labels: readonly string[]): string[]`;
  `electWriter(labels: readonly string[]): string | null`.

- [ ] **Step 1: Write the failing tests**

```ts
describe('the order windows are kept in', () => {
  it('puts main first however the labels arrive', …)
  it('orders the rest by when they were minted, oldest first', …)
  it('puts a label it cannot read the time of last, and keeps them stable by name', …)
})

describe('who writes the session', () => {
  it('is main when main is open', …)
  // The finding this sub-project exists for: main can be closed while other windows live.
  it('is the oldest surviving window when main has gone', …)
  it('hands over when the elected window closes', () => {
    const before = electWriter(['win-1', 'win-2'])   // minted in that order
    expect(electWriter(['win-2'])).not.toBe(before)
    expect(electWriter(['win-2'])).toBe('win-2')
  })
  it('is nobody when there are no windows', () => expect(electWriter([])).toBeNull())
  // Every window must reach the same answer from the same set, whatever order it holds it in.
  it('does not depend on the order the labels are given in', …)
})
```

Mint the labels in the test with real `newWindowLabel()` shapes (`win-<base36>`), written as
literals so the expected value comes from the spec and not from `Date.now()`.

- [ ] **Step 2: Run and confirm red**
- [ ] **Step 3: Implement**

`mintedAt` parses `win-<base36>` and answers `null` for anything else (including `main`).
`windowOrder` is a sort: `main` first, then parsable by ascending mint time, then unparsable by
label. `electWriter` is `windowOrder(labels)[0] ?? null`.

- [ ] **Step 4: Verify** — `pnpm ui:test`, `pnpm typecheck`, `pnpm lint`.

---

### Task 3: a stored box lands on a monitor that exists

**Files:**
- Create: `ui/src/lib/window/monitorClamp.ts`, `ui/src/lib/window/monitorClamp.test.ts`

**Interfaces:**
- Consumes: `MonitorArea` (Task 1), `StoredBox` (Task 4 — declare it here and re-export, or
  declare it in `sessionDocument.ts` and import; **the type lives in `sessionDocument.ts`**,
  because the document is what defines it).
- Produces: `clampToMonitors(box: StoredBox, monitors: readonly MonitorArea[]): Placed`,
  `interface Placed { box: StoredBox; scaleFactor: number }`.

- [ ] **Step 1: Write the failing tests**

```ts
describe('a remembered window lands somewhere it can be reached', () => {
  it('leaves a box that is on a monitor exactly where it was', …)
  it('carries the scale factor of the monitor it is on', …)
  // The case the spec names: the second screen is not plugged in any more.
  it('moves a box that is on no monitor to the primary, at the size it had', …)
  it('chooses the monitor it overlaps most when it straddles two', …)
  // Degrade, never fail: with nothing to clamp against, the box is what it was.
  it('leaves the box alone when no monitor could be read', …)
  it('treats a box touching only the taskbar strip as off the work area', …)
})
```

The last one is the reason `workArea` and not `size`: a box whose only intersection is the band
the taskbar occupies is not reachable, and the primary is where it goes.

- [ ] **Step 2: Run and confirm red**
- [ ] **Step 3: Implement**

Intersection area per monitor, largest wins; ties by monitor order, so the primary wins a tie. No
intersection → the primary's work-area origin, **the size unchanged** (the spec says "at the size
it had", and a window larger than the screen is still reachable by its title bar, while a resized
one is a window the user did not leave). Empty monitor list → `{ box, scaleFactor: 1 }`.

- [ ] **Step 4: Verify** — `pnpm ui:test`, `pnpm typecheck`, `pnpm lint`.

---

### Task 4: the document becomes windows of tabs

**Files:**
- Modify: `ui/src/lib/window/sessionDocument.ts`, `ui/src/lib/window/sessionDocument.test.ts`

**Interfaces:**
- Produces: `interface StoredBox { left; top; width; height }`;
  `interface StoredWindow { tabs: TabSeed[]; activeIndex: number; box?: StoredBox }`;
  `readSession(raw: string | null): StoredWindow[] | null`;
  `writeSession(windows: readonly StoredWindow[]): string`.
- Breaks: both signatures. `session.ts` is the only caller and Task 6 rewrites it.

- [ ] **Step 1: Write the failing tests**

Keep every existing test that is about a *tab* (an unreadable route dropping its tab alone, the
active index following what is left, the pruning of §6) by moving it inside one window. Add:

```ts
describe('the document is windows of tabs', () => {
  it('reads a version 1 document as one window with no box', …)
  it('reads a version 2 document as the windows it holds, in order', …)
  it('keeps each window its own box', …)
  it('drops a window whose every tab was unreadable, and keeps the others', …)
  it('is nothing at all when every window dropped', …)
  it('refuses a version it does not know', () => expect(readSession('{"version":3,"windows":[]}')).toBeNull())
  // §6, now per window: the cap is held by construction and not by a number.
  it('stores a view only on the entry each tab is showing, in every window', …)
  it('writes version 2', …)
})
```

The version 1 test carries the reason in a comment: an app that updates must not lose the tabs
somebody had open, and it has no way to explain that afterwards.

- [ ] **Step 2: Run and confirm red**
- [ ] **Step 3: Implement**

`Version = 2`, `const ReadableVersions = [1, 2]`. A v1 `{tabs, activeIndex}` is read by the tab
reader that already exists and wrapped as one window. A v2 `windows` array is read window by
window; a window with no readable tab is dropped **whole** — turning it into a landing tab would
restore a window the user never had. A box is read field by field and dropped as a whole if any
of the four is not a finite number: half a box is not a position.

Update the module's opening comment — the one that says *"3.7b does"* about the version bump —
to say that it did.

- [ ] **Step 4: Verify** — `pnpm ui:test` (`session.ts` will not typecheck yet; that is Task 6).

---

### Task 5: a window says what it holds, and that it is going

**Files:**
- Modify: `ui/src/lib/window/messages.ts`, `ui/src/lib/window/appWindow.ts`

**Interfaces:**
- Produces: `WindowMessageKind.Holding`, `WindowMessageKind.Closing`; `HoldingMessage`,
  `ClosingMessage`; `watchWindowBox(onChange: () => void): Promise<() => void>`.

- [ ] **Step 1: the two messages**

```ts
// What a window holds, broadcast whenever it changes: its tabs, which one is active, and where
// the window is. **Every window sends it and every window keeps them all**, so whichever window
// turns out to be the elected writer already has the whole session in hand — an election that
// had to ask the others for their tabs first would be a round trip at exactly the moment a
// window is closing.
export interface HoldingMessage {
  kind: typeof WindowMessageKind.Holding
  label: string
  tabs: TabSeed[]
  activeIndex: number
  box?: StoredBox
}

// "I am going." Broadcast as the window unmounts, and **it is not what makes the ledger
// correct** — the writer reconciles against the live roster before every write. It is what makes
// the survivors write *now* instead of at the next tab change, so closing a window is recorded
// even when nothing else happens afterwards.
export interface ClosingMessage {
  kind: typeof WindowMessageKind.Closing
  label: string
}
```

Both join the `WindowMessage` union. The `switch` in `session.ts` stops being exhaustive, which
is the build breaking on purpose.

- [ ] **Step 2: the window's own geometry**

In `appWindow.ts`, beside `watchWindowFocus`:

```ts
// Calls back whenever this window is moved or resized; resolves to the unsubscribe. Outside
// Tauri it never calls back and unsubscribing does nothing, like every other call here.
export const watchWindowBox = async (onChange: () => void): Promise<() => void> => {
  if (!isTauri()) return () => undefined
  const w = getCurrentWindow()
  const stops = await Promise.all([w.onMoved(() => onChange()), w.onResized(() => onChange())])
  return () => stops.forEach((stop) => stop())
}
```

- [ ] **Step 3: Verify** — `pnpm typecheck` fails only in `session.ts`, on the missing arms.

---

### Task 6: the ledger, and the elected writer

**Files:**
- Modify: `ui/src/lib/window/session.ts`
- Create: `ui/src/lib/window/sessionHealth.ts`

**Interfaces:**
- Consumes: Tasks 1–5.
- Produces: `sessionStopped: Ref<boolean>`, `noteSessionError(e: unknown): void`.

- [ ] **Step 1: `sessionHealth.ts`**

```ts
// The one write error that is not swallowed. Every other failure of `set_window_session` leaves
// the tabs on screen and costs nothing to ignore; `SessionTooLarge` means **the session has
// stopped being saved**, and the user finds out at the next restart with nothing having gone
// wrong on screen. A module-level ref, like `focusOrder`: it is a fact about this window, and
// this window is the one that writes.
export const sessionStopped = ref(false)
export const noteSessionError = (e: unknown): void => {
  if (isIpcError(e) && e.kind === 'sessionTooLarge') sessionStopped.value = true
}
```

- [ ] **Step 2: the ledger and the write**

Inside `useWindowSession`:

- `const ledger = new Map<string, StoredWindow>()`, always holding this window's own entry.
- `mine()` builds `{ tabs, activeIndex, box }` from `tabs.session` and the last box read.
- `announce()` writes `mine()` into the ledger and broadcasts `Holding`.
- `remember()` keeps its debounce and becomes:

```ts
const write = async (): Promise<void> => {
  const labels = await windowPort.labels()
  for (const label of [...ledger.keys()]) if (!labels.includes(label)) ledger.delete(label)
  if (electWriter(labels) !== windowPort.label()) return
  // A live window that has not said what it holds yet. Writing now would store a session with
  // one window missing; every window announces as soon as it is seeded, and seeding has a
  // deadline, so this wait is bounded — and it re-arms rather than dropping the write.
  if (labels.some((label) => !ledger.has(label))) return remember()
  const windows = windowOrder(labels).map((label) => ledger.get(label)!).filter((w) => w.tabs.length > 0)
  if (windows.length === 0) return
  try { await setWindowSession(writeSession(windows)) } catch (e) { noteSessionError(e) }
}
```

The `isMain()` guard goes. Its comment is replaced by the reason it went — and the old comment's
second sentence, *"a torn-off window's tabs are not the session"*, is now false and must not be
left standing.

- [ ] **Step 3: the new arms**

`Holding` → `ledger.set(m.label, …)` then `remember()`. `Closing` → `ledger.delete(m.label)` then
`remember()`. Both from every window, including ones that will never be elected: an election
changes with a single close, and a window that kept no ledger would have nothing to write with.

`onMounted` also: `watchWindowBox` → re-read `windowPort.self()`, `announce()`, `remember()`;
and `announce()` once the window is seeded (both branches — restored and torn off).

`onBeforeUnmount` broadcasts `Closing`. It does **not** write.

- [ ] **Step 4: Verify** — `pnpm typecheck`, `pnpm lint`, `pnpm ui:test`.

---

### Task 7: the app comes back as the windows it was

**Files:**
- Modify: `ui/src/lib/window/session.ts`

- [ ] **Step 1: restore, in main's branch of `onMounted`**

```ts
const stored = readSession(await windowSession())   // already inside its try/catch
if (!tabs.pending) return
forget()
tabs.seed(stored?.[0]?.tabs ?? [], stored?.[0]?.activeIndex ?? 0)
void reopen(stored?.slice(1) ?? [])
```

`reopen` is sequential, not `Promise.all`: each window is created and owed its seed, and
`oweSeed` is waited for exactly as `openWindowWith` waits for it — the debt lives in this
window's memory and a newborn that asks before the debt is registered opens empty.

```ts
const reopen = async (rest: readonly StoredWindow[]): Promise<void> => {
  if (rest.length === 0) return
  const monitors = await windowPort.monitors()
  const self = await windowPort.self()
  let n = 0
  for (const w of rest) {
    n += 1
    const placed = clampToMonitors(w.box ?? cascade(self, n), monitors)
    const paid = oweSeed(label, w.tabs, w.activeIndex)
    await windowPort.create(label, { x: placed.box.left, y: placed.box.top },
      { x: placed.box.width / placed.scaleFactor, y: placed.box.height / placed.scaleFactor })
    await paid
  }
}
```

`cascade` is the fallback for a v1 document and for a window whose box could not be read: main's
own box stepped down and right by `CascadeStep` per window, a named constant beside `StripBand`.
**The size is divided by the scale factor** because `create` takes logical pixels for the size
and physical ones for the position — the asymmetry `windowSize()` already carries a comment
about.

- [ ] **Step 2: Verify** — `pnpm typecheck`, `pnpm lint`, `pnpm ui:test`.

---

### Task 8: the session says when it stopped being written

**Files:**
- Modify: `ui/src/components/shell/AboutDialog.vue`, `ui/src/i18n/messages/{it,en}.ts`

- [ ] **Step 1: the alert**

An `Alert` of the warning variant above the promises, `v-if="sessionStopped"`, with a title and a
sentence: the tabs are open and safe, and they will not come back at the next start. Two i18n
keys in both files. **Not a dialog of its own and not a toast**: the spec's word is *diagnostic*,
and nothing the user can do about it justifies interrupting them.

Why About and not a screen: it is the shell's own fact, not any screen's, and it is one click
from every window. When 3.6 builds Settings it moves there, next to the switch that turns the
session off.

- [ ] **Step 2: Verify** — `pnpm ui:test`, `pnpm scan` (a visible string in a template is exactly
  what it catches), `pnpm typecheck`, `pnpm lint`.

---

### Task 9: the whole check, and the half only a window says

- [ ] **Step 1: `pnpm check` from the worktree root.** Green, with the skip count and
  `ISAACDOME_TEST_DECLARATIONS` read rather than the passed count: this worktree has `samples/`
  junctioned, and a run with 0 real files touched is the "green suite, nothing verified" shape.
- [ ] **Step 2: Write the report**, `docs/superpowers/reports/2026-09-16-tabs-windows-report.md`.
  It lists, **unticked, at the top**, what only a window can say, and says plainly that they were
  not seen if they were not:
  - two windows, the app closed and reopened: two windows, in their places, holding what they held
  - `main` closed while a second window lives: the session keeps being written, and the survivor
    is the one writing it
  - a window remembered on a screen that is no longer plugged in opens where it can be reached
  - a window restored on a scaled monitor is the size it was, not twice it
  - a minimized window is still in the document after a restart
- [ ] **Step 3: Update `docs/STATUS.md`** — M1's 3.7b line — and `docs/BACKLOG.md` where B6 and
  B39 name it. A ticked box means committed work, never a note recorded.

## Self-review

- Does any component import `@tauri-apps/api`? No: the two new calls are in `windowPort`, the
  geometry watcher in `appWindow`, and `AboutDialog` reads a ref.
- Is any new number a visual constant? No: `CascadeStep` is window geometry in `lib/window/`,
  where `StripBand` and `TearBand` already live.
- Is the election reachable by a test without a window? Yes — that is why it is a pure function
  of the label set and not of the order windows joined.
- Does anything still assume `main` writes? `grep -n "isMain" ui/src` at the end: the remaining
  uses are the tab store's "the first window keeps its landing tab", which is a different rule.
- Does a version 1 document still open? A test says so, and it is the one test whose failure
  costs a user their tabs on the day they update.
