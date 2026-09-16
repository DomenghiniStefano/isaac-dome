# 3.7c — the measured sizes: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:executing-plans` to implement
> this plan task by task. Steps use checkbox (`- [ ]`) syntax for tracking. This repo's plans are
> executed **inline**, not one-subagent-per-task.

**Goal:** the sidebar you sized stays the sidebar you sized, across a restart. It is the last
piece of 3.7 and the smallest, and the reason it was deferred here rather than done in 3.1 is
that it needed a document to live in.

**Architecture:** the session document gains a **named key beside `windows`**, which costs no
migration and no version bump — that is the whole point of migration 3 having been an object with
a version rather than a bare array of tabs, and of §6 of the spec bumping the version only when
the *top level* changed meaning. The width is one value for the app, shared by every window at
runtime, broadcast when it changes and written by the elected writer.

**Tech Stack:** Vue 3 + TypeScript, Pinia, Vitest. Frontend only — 3.7c touches no Rust, no
`crates/`, no IPC command and no SQL.

**Spec:** `docs/superpowers/specs/2026-09-15-tabs-session-design.md` (§8, §11 item 3), and
`docs/BACKLOG.md` B27 for what the sizes are.

## What this is *not*, measured rather than assumed

B27 has three parts and §8 of the spec names two keys, `sidebarWidth` and `tables`. **Only the
first has a value to store today.** B27's own ordering puts "fill the page" first and "resizable
by hand" second, and neither is built: `grep -rn "useResize\|resizable"` over `ui/src` finds
nothing, and the one resize gesture in the app is the sidebar's, exactly as B27 described it on
2026-09-12. A `tables` key would be a named place for a value nothing produces, which is the kind
of thing this repo does not build in advance — so it is **not** added, and B27 stays open for the
two parts that are actually missing.

## Global Constraints

Copied from `CLAUDE.md` and `docs/frontend-conventions.md`; every task's requirements include
them.

- **Test-first.** The expected value comes from the spec, never from the code's current output.
- **No `<style>` in SFCs**; dynamic values arrive as CSS variables bound by the template — which
  is already how the sidebar's width travels, and this plan does not change it.
- **No hardcoded visual constants.** 3.7c adds no number at all: `SidebarWidth` and
  `clampSidebarWidth` already exist and already carry the bounds.
- **No `invoke()` in components**; nothing outside `ui/src/lib/window/` imports
  `@tauri-apps/api`'s `window`, `webviewWindow` or `event`.
- **Nothing under `ui/src/lib/` imports from `ui/src/components/`** — checked, 0 occurrences
  today, and this plan keeps it at 0. It is why the shared ref holds a bare `number | null` and
  the clamp stays where it is: `App.vue` is a component and may know both.
- **No string unions** — `const X = { … } as const`.
- **Exhaustiveness**: no catch-all arm on a closed union; `assertNever` instead.
- `pnpm scan` enforces the five rules and has no exemption added by this plan.
- **Commits**: Conventional Commits, `type(scope): subject`, scope `ui`. No `Co-Authored-By`
  trailer, no reference to Claude, ever.
- **Before declaring anything done**: `pnpm check` (`scripts/check`) from the worktree root.

## File Structure

**Created**

| file | responsibility |
|---|---|
| `ui/src/lib/window/layout.ts` | the one width the whole app shares, as a ref and a setter. No clamp, no default: it holds `null` until somebody sets one. |

**Changed**

| file | change |
|---|---|
| `ui/src/lib/window/sessionDocument.ts` | the document is a `StoredSession`, not a bare list of windows: `sidebarWidth` beside them |
| `ui/src/lib/window/sessionDocument.test.ts` | its tests, and the one that pins that the version did **not** move |
| `ui/src/lib/window/messages.ts` | `Layout` |
| `ui/src/lib/window/session.ts` | read it, broadcast it, write it |
| `ui/src/App.vue` | the sidebar's width stops being this window's `ref` and becomes the shared one |

**Unchanged, and worth saying.** `components/shell/sidebarWidth.ts` and `SectionSidebar.vue` do
not move: the gesture, the bounds, the keyboard step and the double-click reset are right and
already tested. All 3.7c does is decide where the number it produces lives.

## The decisions this plan takes

1. **One value, shared live.** §8 puts `sidebarWidth` beside `windows` and not inside one, so the
   document holds a single number. Two windows holding different widths would mean the document
   silently keeps one of them, and the user learns which at the next launch — so the value is
   shared at runtime too, and dragging one window's edge moves the others'. **The cost is real
   and is stated rather than discovered**: it is the one thing in 3.7c a window will judge, and
   the alternative was a second, invisible notion of "the width that will be stored".
2. **No version bump.** An older app reading a document with `sidebarWidth` in it ignores a key
   it does not know and is wrong about nothing — which is the rule the module's own comment
   states, and it predicted this key by name.
3. **The document refuses nonsense, the component enforces bounds.** `readSession` keeps a finite
   number and drops anything else; `clampSidebarWidth` puts it between 168 and 420 where it is
   used. Splitting it the other way would put the design's pixels in a parser.
4. **A newborn window is told the layout with its seed's answer.** A torn-off window must open
   with the sidebar its creator has, not with the default, and the moment its creator answers
   `Ready` is the moment somebody new is known to exist.

---

### Task 1: the document carries more than windows

**Files:**
- Modify: `ui/src/lib/window/sessionDocument.ts`, `ui/src/lib/window/sessionDocument.test.ts`

**Interfaces:**
- Produces: `interface StoredSession { windows: StoredWindow[]; sidebarWidth?: number }`;
  `readSession(raw: string | null): StoredSession | null`;
  `writeSession(session: StoredSession): string`.
- Breaks: both signatures. `session.ts` is the only caller and Task 3 follows it.

- [ ] **Step 1: Write the failing tests**

Every existing test keeps its subject and gains `.windows`. New:

```ts
describe('the sizes the document remembers', () => {
  it('round-trips the sidebar width', …)
  it('has none when none was ever set', …)
  it('drops a width that is not a finite number, and keeps the windows', …)
  // The key is beside `windows`, so an app that does not know it is wrong about nothing.
  it('does not move the version', …)
  it('reads a version 1 document, which never had one', …)
})
```

- [ ] **Step 2: Run and confirm red**
- [ ] **Step 3: Implement** — `readSidebarWidth` reuses the finite-number guard `readBox` already
  has; `writeSession` omits the key when there is nothing to write rather than storing `null`.
- [ ] **Step 4: Verify** — `pnpm ui:test` for this file (`session.ts` will not typecheck yet).

---

### Task 2: the one width the app shares

**Files:**
- Create: `ui/src/lib/window/layout.ts`
- Modify: `ui/src/lib/window/messages.ts`

**Interfaces:**
- Produces: `sidebarWidth: Ref<number | null>`, `setSidebarWidth(px: number | null): void`;
  `WindowMessageKind.Layout`, `LayoutMessage`.

No test of its own: a ref and an assignment. What is worth checking is the document (Task 1) and
what the session does with it, and the session is not reachable from Vitest — the same wall 3.7b
hit, and the same answer: what is worth checking is pure and lives in `sessionDocument.ts`.

- [ ] **Step 1: `layout.ts`**, holding `null` until somebody sets a width, with the comment that
  says why it is not a default: the default belongs to the sidebar, which is a component, and
  nothing under `lib/` imports from `components/`.
- [ ] **Step 2: `LayoutMessage { kind, sidebarWidth }`** into the union. The `switch` in
  `session.ts` stops being exhaustive, which is the build breaking on purpose.
- [ ] **Step 3: Verify** — `pnpm typecheck` fails only on the missing arm.

---

### Task 3: the session reads it, says it, and writes it

**Files:**
- Modify: `ui/src/lib/window/session.ts`

- [ ] **Step 1: the arm and the watch**

`Layout` → set the shared ref, **with an echo guard**: the setter would otherwise wake this
window's own watcher, which broadcasts, which wakes the others', and a value that never changes
would still cost a round of messages every time.

`watch(sidebarWidth, …)` → broadcast `Layout` and `remember()`.

- [ ] **Step 2: the document**

Main's restore: `setSidebarWidth(stored?.sidebarWidth ?? null)` beside the seeding it already
does. The write: `writeSession({ windows: decision.windows, sidebarWidth })`, where a `null` ref
is written as nothing at all.

- [ ] **Step 3: the newborn is told**

The answer to `Ready` broadcasts `Layout` as well as sending the `Seed`: a torn-off window must
open with the sidebar its creator has.

- [ ] **Step 4: Verify** — `pnpm typecheck`, `pnpm lint`, `pnpm ui:test`.

---

### Task 4: the sidebar stops being this window's number

**Files:**
- Modify: `ui/src/App.vue`

- [ ] **Step 1**: the local `ref<number>(SidebarWidth.Default)` becomes a writable `computed` over
  the shared ref — reading it through `clampSidebarWidth` so a stored width outside today's
  bounds is brought inside them rather than trusted, writing it through `setSidebarWidth`.
- [ ] **Step 2: Verify** — `pnpm typecheck`, `pnpm lint`, `pnpm ui:test`, `pnpm scan`.

---

### Task 5: the whole check, and the half only a window says

- [ ] **Step 1: `pnpm check`** from the worktree root, reading the skip count and
  `ISAACDOME_TEST_DECLARATIONS` rather than the passed count.
- [ ] **Step 2: the report**, `docs/superpowers/reports/2026-09-16-tabs-sizes-report.md`, listing
  unticked what only a window can say:
  - the sidebar sized, the app closed and reopened: it comes back at that width
  - a width stored by an older build outside today's bounds opens inside them
  - two windows: dragging one edge moves the other, which is decision 1 and is the thing to
    judge rather than assume
  - a torn-off window opens with its creator's sidebar, not the default
- [ ] **Step 3: `docs/STATUS.md`, `docs/BACKLOG.md`** — 3.7 closes as a spec; **B27 does not**,
  and the entry says which two of its three parts are left and why.

## Self-review

- Does anything under `lib/` now import from `components/`? `grep -rn "from '@/components" ui/src/lib/` must still answer 0.
- Is a number hardcoded anywhere? No: 3.7c adds none.
- Is the version bumped? No, and a test says so — a key beside `windows` is one an older app can ignore.
- Is `tables` added? No, and the plan says why: nothing produces a table size yet.
