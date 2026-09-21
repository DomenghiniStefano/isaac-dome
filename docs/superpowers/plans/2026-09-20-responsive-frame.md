# 3.13a — the responsive frame, implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.
> **This repository's owner executes plans inline** (see the handoff at the end): the two-stage
> subagent loop has been tried here and costs more than it returns.

**Goal:** build the frame that lets a screen fill the window and fold when it can't, and migrate
Unlock onto it as the proof.

**Architecture:** every threshold is a **container query** against one of two named size containers
— `page` (the `<main>` element) and `shell` (the row holding the sidebar and `<main>`) — never a
media query, because the window's width is not the content's and a media query cannot see the
interface's scale. `<main>` stops scrolling and becomes a page box; each screen declares itself
**flowing** (it scrolls) or **filling** (one region takes the height that is left and scrolls
inside). Four rules in `ui/scripts/scan-conventions.mjs` keep it from drifting back.

**Tech Stack:** Vue 3 + TypeScript, Tailwind v4 (container queries in core), Vitest, Tauri 2.

**Spec:** `docs/superpowers/specs/2026-09-20-responsive-layout-design.md` — read it first; every
task below argues from a numbered section of it.

**Branch / worktree:** `feature/responsive-layout` in `C:\Projects\isaac-dome-responsive`.
`samples/` there is a junction to the main worktree's — **do not delete the worktree without
removing the reparse point first** (`CLAUDE.md`, "Don't delete a worktree before unlinking its
junctions").

## Global Constraints

- **Full width, no reading cap anywhere** (§3). `max-w-*` leaves every screen root and nothing
  replaces it, the wiki page included. The long prose line is an accepted cost.
- **The floor is 640 × 480 logical pixels** (§8), declared in `crates/app/tauri.conf.json` **and**
  in the frontend's window creation. At the floor the page container is roughly 428 px at scale
  100 (640 − 168 sidebar − 44 of horizontal padding).
- **Every container variant names its container** (§5): `@max-compact/page:`, `@wide/page:`,
  `@max-compact/shell:`. Never an unnamed `@max-compact:`.
- **Three shared sizes only**: `compact`, `regular`, `wide`. A component may declare a private
  token beside itself if it was measured on the drawn component, with the scale written in the
  comment (`--container-tab-narrow` is the one that exists).
- **No media-query variant** (`sm:` `md:` `lg:` `xl:` `2xl:`) anywhere in `ui/`.
- **`min-h-0` on every link** of a filling screen's chain. A missing one makes `flex-1` grow
  instead of fit, silently.
- **Frontend rules still apply**: no `<style>` in SFCs, no hardcoded visual constants, no
  `invoke()` outside `lib/ipc/`, no raw `<button>`/`<input>`, no string unions.
- **A px token must carry a reason on the line above or beside it** — `scan-conventions.mjs`
  fails otherwise (`pxWithoutReason`).
- **Commits**: Conventional Commits, `type(scope): subject`, English, atomic, **never** a
  `Co-Authored-By` trailer or any reference to Claude.
- **Before declaring anything done**: `pnpm check`.

---

## File map

| File | Responsibility | Task |
|---|---|---|
| `ui/src/lib/window/windowFloor.ts` | *new* — the floor as a constant the frontend and the test both read | 1 |
| `ui/src/lib/window/windowFloor.test.ts` | *new* — the config and the constant say the same number | 1 |
| `crates/app/tauri.conf.json` | the floor for the main window and the one the tray rebuilds | 1 |
| `ui/src/lib/window/windowPort.ts` | the floor for a torn-off window | 1 |
| `ui/src/assets/theme/containers.css` | the three thresholds | 2 |
| `ui/src/lib/design/thresholds.ts` | *new* — the three names and their px, for the test and for anything that must name one | 2 |
| `ui/src/lib/design/thresholds.test.ts` | *new* — the CSS and the constants agree, and the three ascend | 2 |
| `ui/src/App.vue` | the page box, the two containers, the group for the sidebar's second input | 3, 7 |
| the 22 screen roots | flowing shape, `max-w-*` gone | 3 |
| `ui/scripts/scan-conventions.mjs` | the four rules and their fixtures | 4, 5, 8 |
| `ui/src/screens/{Floor,Plan,Live,Runs}Screen.vue` | the eight media-query variants converted | 5 |
| `ui/src/components/ui/virtual/VirtualRows.vue` | the scroll body takes the height instead of a token | 6 |
| `ui/src/assets/theme/spacing.css` | `--spacing-virtual-rows-body` deleted, `--spacing-sidebar-icons` added | 6, 7 |
| `ui/src/screens/UnlockScreen.vue`, `unlock/UnlockTable.vue` | the filling chain | 6 |
| `ui/src/assets/utilities.css` | the collapsed-sidebar utility, `grid-cols-unlock-narrow` | 7, 8 |
| `ui/src/components/shell/SectionSidebar.vue`, `SidebarItem.vue` | the collapsed state | 7 |
| `ui/src/screens/unlock/UnlockTable.vue`, `UnlockRow.vue` | the columns that fall | 8 |
| `ui/src/kit/sections/app/WidthsSection.vue` | *new* — the table at the three widths, side by side | 9 |
| `docs/frontend-conventions.md` | the section that explains all of it | 10 |
| `scripts/test-floor` | the raised counts | 11 |

---

### Task 1: The window floor, in both the places a window is born

**Files:**
- Create: `ui/src/lib/window/windowFloor.ts`
- Create: `ui/src/lib/window/windowFloor.test.ts`
- Modify: `crates/app/tauri.conf.json` (the `app.windows[0]` object)
- Modify: `ui/src/lib/window/windowPort.ts` (the `create` function, the `new WebviewWindow` options)

**Interfaces:**
- Consumes: nothing.
- Produces: `WindowFloor` — `{ readonly Width: 640; readonly Height: 480 }`, imported by
  `windowPort.ts` and by the test.

**Why two places** (spec §1, §8): `tauri.conf.json` is the recipe for the main window and for the
one `crates/app/src/window.rs` rebuilds from the tray; a window born from a torn-off tab is built
by the frontend at `windowPort.ts`. A floor written once holds for every window except the ones the
user makes by hand.

- [ ] **Step 1: Write the failing test**

Create `ui/src/lib/window/windowFloor.test.ts`:

```ts
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { WindowFloor } from './windowFloor'

// Two files declare the floor and neither can see the other (spec §8): the Tauri config is the
// recipe for the main window and the one the tray rebuilds, while a torn-off tab is built in the
// frontend from the constant below. Read with `node:fs` rather than `?raw` because the file is
// outside `ui/`, which Vite would have to be told about.
const conf = JSON.parse(
  readFileSync(
    fileURLToPath(
      new URL('../../../../crates/app/tauri.conf.json', import.meta.url),
    ),
    'utf8',
  ),
) as { app: { windows: Array<{ minWidth?: number; minHeight?: number }> } }

describe('the window floor', () => {
  it('is 640 x 480, the size the compact layouts are designed against', () => {
    expect(WindowFloor.Width).toBe(640)
    expect(WindowFloor.Height).toBe(480)
  })

  it('is the same number in the Tauri config as in the constant', () => {
    const main = conf.app.windows[0]
    expect(main?.minWidth).toBe(WindowFloor.Width)
    expect(main?.minHeight).toBe(WindowFloor.Height)
  })
})
```

- [ ] **Step 2: Run it and watch it fail**

```
pnpm --filter ui test -- windowFloor
```

Expected: FAIL — `Cannot find module './windowFloor'`.

- [ ] **Step 3: Write the constant**

Create `ui/src/lib/window/windowFloor.ts`:

```ts
// The smallest window the layout is designed against (spec 3.13a §8). Logical pixels, and **not**
// a theme token: this is a window's size, which the interface's scale never moves — a floor that
// grew with the scale would let the user shrink the app below its own design by making the text
// bigger.
export const WindowFloor = {
  Width: 640,
  Height: 480,
} as const
```

- [ ] **Step 4: Add the floor to the Tauri config**

In `crates/app/tauri.conf.json`, the window object gains two keys beside `width`/`height`:

```json
        "width": 1280,
        "height": 800,
        "minWidth": 640,
        "minHeight": 480,
```

- [ ] **Step 5: Run the test and watch it pass**

```
pnpm --filter ui test -- windowFloor
```

Expected: PASS, 2 tests.

- [ ] **Step 6: Give a torn-off window the same floor**

In `ui/src/lib/window/windowPort.ts`, inside `create`, add the import and the two options:

```ts
import { WindowFloor } from './windowFloor'
```

```ts
    const w = new WebviewWindow(label, {
      url: 'index.html',
      width: size.x,
      height: size.y,
      // The same floor as the window it was torn out of: a window born here never went through
      // the Tauri config, so a floor written only there would not reach it (spec §8).
      minWidth: WindowFloor.Width,
      minHeight: WindowFloor.Height,
      decorations: false,
      backgroundColor: windowBackground(),
      visible: false,
    })
```

- [ ] **Step 7: Typecheck and commit**

```bash
pnpm typecheck
git add ui/src/lib/window/windowFloor.ts ui/src/lib/window/windowFloor.test.ts crates/app/tauri.conf.json ui/src/lib/window/windowPort.ts
git commit -m "feat(ui): a window cannot be made smaller than the layout it draws"
```

---

### Task 2: The three thresholds, and the test that they exist and ascend

**Files:**
- Modify: `ui/src/assets/theme/containers.css`
- Create: `ui/src/lib/design/thresholds.ts`
- Create: `ui/src/lib/design/thresholds.test.ts`

**Interfaces:**
- Consumes: nothing.
- Produces: `Threshold` — an `as const` object with `Compact: 'compact'`, `Regular: 'regular'`,
  `Wide: 'wide'`; `ThresholdPx` — `Record<Threshold, number>`.

**On the numbers**: the spec (§5, §12) says they are measured on the drawn thing, not chosen on
paper. What this task writes is the starting point, derived and stated: `compact` sits above the
428 px the page container has at the window floor, so the floor is inside compact; `regular` is
near today's `max-w-250` (1000 px); `wide` is where a 1280 window can hold an aside beside the
content. **Task 9 looks at them on the Kit page and moves them if the drawn thing disagrees** —
moving one means changing the CSS and `ThresholdPx` together, which is what the test is for.

- [ ] **Step 1: Write the failing test**

Create `ui/src/lib/design/thresholds.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import containers from '@/assets/theme/containers.css?raw'
import { Threshold, ThresholdPx } from './thresholds'

// The pattern of `lib/scale/rows.test.ts`: a number lives in the CSS, where the layout reads it,
// and in TypeScript, where anything that has to name it reads it. If the two drift, a threshold
// moves in one place and the other keeps describing the old layout.
describe('the page thresholds', () => {
  it('are declared in containers.css at the px the constants say', () => {
    for (const name of Object.values(Threshold))
      expect(containers).toMatch(
        new RegExp(String.raw`--container-${name}:\s*${ThresholdPx[name]}px;`),
      )
  })

  it('ascend: compact is narrower than regular, regular than wide', () => {
    expect(ThresholdPx[Threshold.Compact]).toBeLessThan(
      ThresholdPx[Threshold.Regular],
    )
    expect(ThresholdPx[Threshold.Regular]).toBeLessThan(
      ThresholdPx[Threshold.Wide],
    )
  })

  it('leave the window floor inside compact', () => {
    // 640 window − 168 sidebar at its minimum − 44 of horizontal padding on the page box.
    expect(ThresholdPx[Threshold.Compact]).toBeGreaterThan(640 - 168 - 44)
  })
})
```

- [ ] **Step 2: Run it and watch it fail**

```
pnpm --filter ui test -- thresholds
```

Expected: FAIL — `Cannot find module './thresholds'`.

- [ ] **Step 3: Write the constants**

Create `ui/src/lib/design/thresholds.ts`:

```ts
// The three widths a screen changes shape at (spec 3.13a §5). Three and not seven: a shared scale
// is the only thing that keeps twenty-two screens telling the same story.
export const Threshold = {
  Compact: 'compact',
  Regular: 'regular',
  Wide: 'wide',
} as const
export type Threshold = (typeof Threshold)[keyof typeof Threshold]

// The px each one is declared at in `assets/theme/containers.css`, kept here so the tests and any
// reader can name a threshold without parsing CSS. Moving one means moving both.
export const ThresholdPx: Record<Threshold, number> = {
  [Threshold.Compact]: 560,
  [Threshold.Regular]: 960,
  [Threshold.Wide]: 1280,
}
```

- [ ] **Step 4: Declare the tokens**

Append to `ui/src/assets/theme/containers.css`, inside the existing `@theme` block:

```css
  /* The three widths a screen changes shape at (spec 3.13a §5), used as `@max-compact/page:` and
     `@wide/page:`. In px and measured at scale 100, for the reason the tab's token gives above: a
     container query is compared against a used width.

     compact — below it, one column: tables at their minimum column set, KPI strips stacked. It
     sits above the 428px the page box has at the 640 window floor, so the floor is inside it.
     regular — the shape the screens are drawn at today, near the old `max-w-250`.
     wide — above it, what sat under something may sit beside it. */
  --container-compact: 560px;
  --container-regular: 960px;
  --container-wide: 1280px;
```

- [ ] **Step 5: Run the test and watch it pass**

```
pnpm --filter ui test -- thresholds
```

Expected: PASS, 3 tests.

- [ ] **Step 6: Commit**

```bash
git add ui/src/lib/design/thresholds.ts ui/src/lib/design/thresholds.test.ts ui/src/assets/theme/containers.css
git commit -m "feat(ui): three widths a screen changes shape at"
```

---

### Task 3: The page box, and every screen becomes flowing

**Files:**
- Modify: `ui/src/App.vue` (the shell wrapper `div` and `<main>`)
- Modify: all 22 screen roots (below)

**Interfaces:**
- Consumes: `--container-*` from Task 2 (not used yet in this task, but the containers are declared
  here so the variants have somewhere to resolve).
- Produces: the `page` and `shell` containers; the flowing shape, which Tasks 4 and 7 depend on.

**The order matters**: `<main>` stops scrolling in this task, so every screen has to gain its own
scroll in the same commit, or the app clips its content. That is why the sweep and the page box are
one task and not two.

- [ ] **Step 1: Turn the shell row into the `shell` container and `<main>` into the page box**

In `ui/src/App.vue`, the wrapper:

```html
        <div class="@container/shell flex min-h-0 flex-1">
```

and `<main>`:

```html
          <main
            class="@container/page min-h-0 min-w-0 flex-1 overflow-hidden px-5.5"
          >
```

Two things changed and each has a reason: `overflow-auto` became `overflow-hidden`, because the
screen scrolls now and not the box around it; and `pt-5 pb-15` left, because on a filling screen
(Task 6) that bottom padding would be sixty pixels of nothing under a list that could have used
them. The vertical padding moves into the screen roots in the next step.

- [ ] **Step 2: Sweep the 22 screen roots**

Every root of the form `<div class="flex max-w-250 flex-col gap-4">` becomes:

```html
  <div class="flex h-full flex-col gap-4 overflow-y-auto pt-5 pb-15">
```

The three that differ:
- `ui/src/screens/PlanScreen.vue:48` — `max-w-300` → same replacement.
- `ui/src/screens/GoalsScreen.vue:99` — `max-w-190` → same replacement.
- `ui/src/screens/welcome/WelcomeScreen.vue:45` — `w-full max-w-250` → `w-full`. This one is inside
  the welcome takeover, which is not under `<main>` at all; it keeps its own layout and only loses
  the cap.

The full list of files to edit (find them with `grep -rn "max-w-250\|max-w-300\|max-w-190" ui/src`):
`AppearanceScreen`, `BackgroundScreen`, `ChallengesScreen`, `CollectionScreen`, `CompletionScreen`,
`FloorScreen`, `GoalsScreen`, `LiveScreen`, `PlaceholderScreen`, `PlanScreen`, `ProfileScreen`,
`RollScreen`, `RunsScreen`, `SearchScreen`, `TabsSettingsScreen`, `UnlockScreen`, `UpdatesScreen`,
`welcome/WelcomeScreen`, `wiki/WikiCategoryList`, `wiki/WikiLanding`, `wiki/WikiPage`.

**Leave `max-w-80` on the two `TooltipContent` alone** (`WikiLanding.vue:105`, `WikiPage.vue:137`):
a tooltip is not a screen and a cap on a floating panel is not the cap this spec removed.

- [ ] **Step 3: Look at it**

```
pnpm ui:dev
```

Open a few screens. What must be true: content reaches the right edge of the window; the screen
scrolls, not the window; nothing is clipped at the bottom. What is *expected to still be wrong*:
the virtualized lists still stop at 35rem — that is Task 6.

- [ ] **Step 4: Commit**

```bash
pnpm typecheck && pnpm ui:test
git add ui/src/App.vue ui/src/screens
git commit -m "feat(ui): a screen fills the width and carries its own scroll"
```

---

### Task 4: The scanner learns what a screen root looks like

**Files:**
- Modify: `ui/scripts/scan-conventions.mjs` (a helper, two entries in `checks`, four in `FIXTURES`)

**Interfaces:**
- Consumes: the flowing shape from Task 3 — the rules land green only after it.
- Produces: the two rules. Task 5 and Task 8 add to the same arrays.

**Why the root and not the file**: both rules are facts about one line — the first tag inside
`<template>` — so the helper extracts exactly that and the rules read its class list. Anything
deeper is a component, not a screen.

- [ ] **Step 1: Add the helper, beside `visibleText`**

```js
const SCREENS_DIR = join('src', 'screens')
// A screen's root: the first tag inside its <template>, with its class list. Both rules below are
// facts about that one line and nothing deeper — a `max-w-*` on a card inside a screen is a card's
// business (spec 3.13a §4).
const ROOT_TAG = /<template>\s*(?:<!--[\s\S]*?-->\s*)*<([a-zA-Z][\w-]*)([^>]*)>/
const rootAttrs = (body) => {
  const template = body.match(TEMPLATE_BLOCK)
  if (!template) return null
  const root = `<template>${template[1]}`.match(ROOT_TAG)
  return root ? root[2] : null
}
const rootClasses = (body) => {
  const attrs = rootAttrs(body)
  const cls = attrs?.match(/\bclass="([^"]*)"/)
  return cls ? cls[1] : ''
}
```

- [ ] **Step 2: Add the two rules to `checks`**

```js
  {
    // Spec 3.13a §4. Two shapes and not twenty-two: a screen either flows and scrolls, or fills
    // and hands its height to one region. A root that is neither is a screen whose height nobody
    // decided, which fails nothing and looks like a bug in the list inside it.
    name: 'screen root is neither flowing nor filling',
    test: (file, body) => {
      if (!file.endsWith('.vue') || !isUnder(file, SCREENS_DIR)) return false
      const cls = rootClasses(body)
      if (cls === '') return false
      const flowing = /\boverflow-y-auto\b/.test(cls)
      const filling =
        /\boverflow-hidden\b/.test(cls) && /\bmin-h-0\b/.test(cls)
      return !/\bh-full\b/.test(cls) || !(flowing || filling)
    },
  },
  {
    // Spec 3.13a §3: the cap left by decision. This stops it coming back by habit from the screen
    // next door, which is exactly how all twenty-two came to carry the same one.
    name: 'width cap on a screen root',
    test: (file, body) =>
      file.endsWith('.vue') &&
      isUnder(file, SCREENS_DIR) &&
      /\bmax-w-/.test(rootClasses(body)),
  },
```

- [ ] **Step 3: Add the four fixtures to `FIXTURES`**

```js
  {
    name: 'a flowing screen root is allowed',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="flex h-full flex-col gap-4 overflow-y-auto pt-5 pb-15" />\n</template>\n',
    expect: [],
  },
  {
    name: 'a filling screen root is allowed',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="flex h-full min-h-0 flex-col gap-4 overflow-hidden pt-5 pb-5" />\n</template>\n',
    expect: [],
  },
  {
    name: 'a screen root with no shape at all is caught',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="flex flex-col gap-4" />\n</template>\n',
    expect: ['screen root is neither flowing nor filling'],
  },
  {
    name: 'a width cap on a screen root is caught',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="flex h-full max-w-250 flex-col overflow-y-auto" />\n</template>\n',
    expect: ['width cap on a screen root'],
  },
```

- [ ] **Step 4: Run the scanner**

```
pnpm scan
```

Expected: `0 violations, 0 declared exemptions, 22 fixtures`. If a real screen is flagged, it is
Task 3 that is incomplete — **fix the screen, not the rule**. If a screen legitimately differs
(`welcome/WelcomeScreen.vue` is not under `<main>`), add it to `EXEMPTIONS` with that reason:

```js
const EXEMPTIONS = [
  {
    file: 'src/screens/welcome/WelcomeScreen.vue',
    check: 'screen root is neither flowing nor filling',
    reason:
      'the welcome is a takeover above the router, drawn outside <main>: it has no page box to fill',
  },
]
```

- [ ] **Step 5: Commit**

```bash
git add ui/scripts/scan-conventions.mjs
git commit -m "chore(ui): the scanner reads a screen's root and the shape it declares"
```

---

### Task 5: The eight media-query variants leave, and the rule that keeps them out

**Files:**
- Modify: `ui/src/screens/FloorScreen.vue:92-93`, `PlanScreen.vue:74,86`, `LiveScreen.vue:62`,
  `RunsScreen.vue:112`
- Modify: `ui/src/components/shell/TabItem.vue` (its container gains a name)
- Modify: `ui/scripts/scan-conventions.mjs`

**Interfaces:**
- Consumes: `--container-*` (Task 2), the `page` container (Task 3).
- Produces: nothing later tasks read.

- [ ] **Step 1: Convert the six lines**

`FloorScreen.vue:92-93`:

```html
    <div class="flex flex-col items-start gap-4 @wide/page:flex-row">
      <Card class="w-full @wide/page:w-fit @wide/page:shrink-0">
```

`PlanScreen.vue:74` and `:86`:

```html
      <div v-if="readable" class="flex flex-col items-start gap-4 @wide/page:flex-row">
```

```html
          class="w-full @wide/page:w-plan-aside @wide/page:shrink-0"
```

`LiveScreen.vue:62` and `RunsScreen.vue:112`:

```html
        <div class="grid grid-cols-2 gap-3 @regular/page:grid-cols-4">
```

- [ ] **Step 2: Give the tab's container its name**

`TabItem.vue` currently declares a bare `@container` and uses `@max-tab-narrow:`. Under §5 every
variant names its container, so both halves change: `@container` → `@container/tab`, and each of
the three `@max-tab-narrow:` → `@max-tab-narrow/tab:`. The token itself does not move.

- [ ] **Step 3: Add the rule and its two halves**

In `checks`:

```js
  {
    // Spec 3.13a §2 and §9. The first half is the rejected approach: a media query measures the
    // window, which is not what the content has. The second half is the hole the first leaves —
    // `containers.css` does not reset Tailwind's own `--container-*` scale, so `@max-md/page:` is
    // writable today and would be a fourth threshold nobody declared.
    //
    // **A Vue event binding is not a variant.** `@update:open="x"` has the same shape, so the two
    // are told apart the only way that is honest here: a container variant names its container
    // with a `/`, an event binding never does. The list of bare names below exists so that the
    // rule can say so out loud rather than by accident.
    name: 'media query variant, or a container size that is not ours',
    test: (_f, body) => {
      const media = /(^|[\s"'`])(?:[a-z0-9-]+:)*(?:sm|md|lg|xl|2xl):/m.test(body)
      const ours = new Set(['compact', 'regular', 'wide', 'tab-narrow'])
      const named = [...body.matchAll(/@(?:max-)?([a-z0-9-]+)\/[a-z-]+:/g)]
      return media || named.some(([, size]) => !ours.has(size))
    },
  },
```

- [ ] **Step 4: Add the four fixtures**

```js
  {
    name: 'a media query variant is caught',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="grid grid-cols-2 sm:grid-cols-4" />\n</template>\n',
    expect: ['media query variant, or a container size that is not ours'],
  },
  {
    name: 'one of our container variants is allowed',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="flex flex-col @wide/page:flex-row @max-compact/page:hidden" />\n</template>\n',
    expect: [],
  },
  {
    name: "a container size that is not ours is caught",
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="@max-md/page:hidden" />\n</template>\n',
    expect: ['media query variant, or a container size that is not ours'],
  },
  {
    name: 'a Vue event binding is not a container variant',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <Thing @update:open="go" @update:model-value="go" />\n</template>\n',
    expect: [],
  },
```

- [ ] **Step 5: Run the scanner and the suite**

```
pnpm scan && pnpm ui:test
```

Expected: `0 violations, … 26 fixtures`. A violation here means a conversion in Step 1 or 2 was
missed.

- [ ] **Step 6: Commit**

```bash
git add ui/src/screens ui/src/components/shell/TabItem.vue ui/scripts/scan-conventions.mjs
git commit -m "refactor(ui): every fold is measured against its own box, never the window"
```

---

### Task 6: The filling shape — the list takes the height that is left

**Files:**
- Modify: `ui/src/components/ui/virtual/VirtualRows.vue`
- Modify: `ui/src/assets/theme/spacing.css` (delete `--spacing-virtual-rows-body`)
- Modify: `ui/src/screens/UnlockScreen.vue`, `ui/src/screens/unlock/UnlockTable.vue`

**Interfaces:**
- Consumes: the page box (Task 3).
- Produces: the filling chain, which Tasks 8 and 9 draw inside, and which 3.13b repeats for the
  other five lists.

**This is the task that breaks silently.** Every link from the screen root to the scroll box needs
`min-h-0`; one missing and `flex-1` grows instead of fitting, the list pushes the page, and it looks
like the virtualizer's fault. Check the chain link by link in Step 4.

- [ ] **Step 1: The scroll box takes the height instead of capping it**

In `VirtualRows.vue`, the scroller element:

```html
  <div
    ref="scroller"
    class="min-h-0 flex-1 overflow-auto"
    @scroll="onScroll"
  >
```

- [ ] **Step 2: Delete the token**

In `ui/src/assets/theme/spacing.css`, remove the `--spacing-virtual-rows-body` declaration and the
comment above it that describes it. Nothing else reads it — confirm with:

```
grep -rn "virtual-rows-body" ui/src
```

Expected: no matches.

- [ ] **Step 3: Make Unlock a filling screen**

`UnlockScreen.vue` root:

```html
  <div class="flex h-full min-h-0 flex-col gap-4 overflow-hidden pt-5 pb-5">
```

The `Card` that holds the filter bar and the table:

```html
      <Card class="flex min-h-0 flex-1 flex-col">
```

`UnlockTable.vue` root:

```html
  <div class="flex min-h-0 flex-1 flex-col">
```

- [ ] **Step 4: Walk the chain and look at it**

```
pnpm ui:dev
```

Open Unlock. What must be true: the header, the diagnostics and the filter bar stay put; only the
rows scroll; the last row ends at the bottom of the window with no gap and no page scrollbar; making
the window taller gives the rows the extra height.

If the rows push the page instead, walk the chain from the root down —
`div` → `Card` → `div.flex.flex-col` → the scroller — and find the link without `min-h-0`.

- [ ] **Step 5: Run the suite and commit**

```bash
pnpm typecheck && pnpm ui:test && pnpm scan
git add ui/src/components/ui/virtual/VirtualRows.vue ui/src/assets/theme/spacing.css ui/src/screens/UnlockScreen.vue ui/src/screens/unlock/UnlockTable.vue
git commit -m "feat(ui): a list is as tall as what is left, and scrolls inside it"
```

---

### Task 7: The sidebar collapses to icons

**Files:**
- Modify: `ui/src/assets/theme/spacing.css` (`--spacing-sidebar-icons`)
- Modify: `ui/src/assets/utilities.css` (the collapsed utility)
- Modify: `ui/src/components/shell/SectionSidebar.vue`, `SidebarItem.vue`
- Modify: `ui/src/App.vue` (the group, and `:label` on the items)
- Modify: `ui/src/kit/sections/app/SectionSidebarSection.vue`

**Interfaces:**
- Consumes: the `shell` container (Task 3).
- Produces: `SidebarItem`'s new required prop `label: string` — the default slot for the item's text
  goes away, because the tooltip needs the text as a value. The icon slot stays.

**The state has two inputs and both are CSS** (§6): too narrow, **or** the shell root carries
`data-sidebar="collapsed"`. **Nothing writes the attribute in this cycle** — the button is its own
card. It is wired here so that card writes one attribute and reopens nothing.

- [ ] **Step 1: The width of a collapsed sidebar**

In `ui/src/assets/theme/spacing.css`, beside the other shell sizes:

```css
  /* A sidebar with nothing but its icons: the row button's own height, so the column reads as
     square (spec 3.13a §6). */
  --spacing-sidebar-icons: 3rem;
```

- [ ] **Step 2: The collapsed condition, written once**

In `ui/src/assets/utilities.css`:

```css
/* Collapsed by either input (spec 3.13a §6): the shell is too narrow, or somebody asked for it.
   Written as one utility because the pair *is* the state — repeating two variants on six elements
   is six chances to write only half of it, and the half that is missing shows up only at one of
   the two. */
@utility sidebar-collapsed-hidden {
  @container shell (width < 560px) {
    display: none;
  }
  [data-sidebar='collapsed'] & {
    display: none;
  }
}
```

**If Tailwind refuses the nested `@container` inside `@utility`**, drop the utility and write the
pair inline on each of the six elements instead — same meaning, more places to get it wrong:

```
@max-compact/shell:hidden group-data-[sidebar=collapsed]/shell:hidden
```

The `560px` above is `--container-compact`; if Task 9 moves the threshold, move it here too, and
say so in the commit.

- [ ] **Step 3: The shell root becomes a group**

In `App.vue`:

```html
        <div class="group/shell @container/shell flex min-h-0 flex-1">
```

The `data-sidebar` attribute is deliberately absent: the button's card adds it.

- [ ] **Step 4: The aside, its header and its handle**

In `SectionSidebar.vue`, the `<aside>`:

```html
  <aside
    :style="widthVariable"
    class="flex w-(--sidebar-width) shrink-0 border border-secondary bg-data @max-compact/shell:w-sidebar-icons group-data-[sidebar=collapsed]/shell:w-sidebar-icons"
  >
```

The header row's title and its `HelpTip` take `sidebar-collapsed-hidden`, and the header row itself
centres when there is no text:

```html
      <div class="flex items-center gap-2 px-2.75 pt-2.5 pb-2.25 @max-compact/shell:justify-center">
        <span class="text-highlight [&_svg]:size-3.5"><slot name="icon" /></span>
        <span class="sidebar-collapsed-hidden text-caption tracking-caps text-foreground uppercase">{{ title }}</span>
        <HelpTip v-if="hint" class="sidebar-collapsed-hidden">{{ hint }}</HelpTip>
      </div>
```

The resize handle takes the same utility — dragging the edge of a collapsed sidebar means nothing:

```html
      class="sidebar-collapsed-hidden w-1.25 shrink-0 cursor-col-resize hover:bg-input data-[resizing=true]:bg-input"
```

- [ ] **Step 5: `SidebarItem` takes its label as a value, and a tooltip with it**

```vue
<script setup lang="ts">
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { AriaCurrent } from '@/lib/constants/aria'

// The label is a **value** and not a slot, because the tooltip needs the same text the row shows.
// The tooltip is always mounted, not only when collapsed: whether the sidebar is collapsed is a
// CSS fact here (spec §6) and JavaScript never learns it — and a label truncated by a narrow
// sidebar wants the tooltip just as much as a hidden one does.
defineProps<{ active: boolean; label: string }>()
</script>

<template>
  <Tooltip>
    <TooltipTrigger as-child>
      <!-- The active item has the red bar on the left (ButtonVariant.Nav). -->
      <Button
        :variant="ButtonVariant.Nav"
        :size="ButtonSize.Row"
        :aria-current="active ? AriaCurrent.Page : undefined"
        class="gap-2.25 [&_svg]:size-4 [&_svg]:text-subtle-foreground aria-[current=page]:[&_svg]:text-foreground @max-compact/shell:justify-center"
      >
        <slot name="icon" />
        <span class="sidebar-collapsed-hidden truncate">{{ label }}</span>
      </Button>
    </TooltipTrigger>
    <TooltipContent>{{ label }}</TooltipContent>
  </Tooltip>
</template>
```

The three names are verified: `ui/src/components/ui/tooltip/index.ts` exports `Tooltip`,
`TooltipTrigger` and `TooltipContent` beside `HelpTip`, and `TooltipTrigger` forwards Reka's props
whole (`v-bind="props"` over `TooltipTriggerProps`), so `as-child` reaches the primitive and the
`Button` stays the trigger. `TooltipProvider` is already mounted at the root of both `App.vue` and
the Kit page, so nothing else has to be added.

**Why a tooltip and not a `title` attribute**, since `HelpTip.vue` argues it in its own comment and
the same argument holds here: `title` is invisible to the keyboard, arrives a second late with none
of our styling, and never reaches a touch at all.

- [ ] **Step 6: Both callers pass the label**

In `App.vue`:

```html
            <SidebarItem
              v-for="entry in entries"
              :key="entry.key"
              :active="isEntryActive(entry, tabs.location)"
              :label="t(entry.label)"
              @click="openEntry(entry, $event)"
            >
              <template #icon><component :is="entry.icon" /></template>
            </SidebarItem>
```

In `SectionSidebarSection.vue` (Kit), the same: `:label="item.label"` for the loop, and
`label="Profilo di gioco"` / `label="Tab"` for the two literal ones, with the text out of the
element body.

- [ ] **Step 7: See both states on the Kit page**

In `SectionSidebarSection.vue`, wrap the second sidebar in a shell-sized box so the collapsed state
is visible without touching the window:

```html
    <div class="@container/shell group/shell flex w-96 items-start gap-4">
```

A 384 px box is below `--container-compact`, so that sidebar draws collapsed while the first one,
outside the box, stays wide.

```
pnpm ui:dev
```
then `#kit`. What must be true: the narrow one is a column of icons, each with a tooltip on hover;
no resize handle on it; the wide one is unchanged.

- [ ] **Step 8: Commit**

```bash
pnpm typecheck && pnpm ui:test && pnpm scan
git add ui/src/assets ui/src/components/shell ui/src/App.vue ui/src/kit
git commit -m "feat(ui): the section sidebar falls back to its icons when there is no room"
```

---

### Task 8: Unlock's columns fall by priority

**Files:**
- Modify: `ui/src/assets/utilities.css` (`grid-cols-unlock-narrow`, beside `grid-cols-unlock`)
- Modify: `ui/src/screens/unlock/UnlockTable.vue` (the header's seven cells, both grid classes)
- Modify: `ui/src/screens/unlock/UnlockRow.vue` (the row's seven cells)
- Modify: `ui/scripts/scan-conventions.mjs`

**Interfaces:**
- Consumes: `--container-compact`, the `page` container, the filling chain (Tasks 2, 3, 6).
- Produces: the pattern 3.13b repeats on five more tables.

**Which three fall, and why those**: `unlocks` and `condition` are the two widest and the two whose
absence costs least — the row is still identified by its name — and `fanOut` is a number that means
nothing without the columns beside it. What stays is the drawing, the name, the state badge and the
button that queues the row: at the 428 px floor that is 64 + 136 + 40 of fixed columns and about
188 px of name.

- [ ] **Step 1: The narrow template, beside the wide one**

In `ui/src/assets/utilities.css`, immediately after `@utility grid-cols-unlock`:

```css
/* Unlock at compact (spec 3.13a §7): the drawing, the name, the state, the queue button. What
   falls — what the row unlocks, its condition, and the fan-out — carries
   `@max-compact/page:hidden` on its cell in `UnlockTable.vue` (header) and `UnlockRow.vue` (row).
   The two edits are one change: a template that drops a track while the cell stays shifts every
   cell after it into the wrong column. */
@utility grid-cols-unlock-narrow {
  grid-template-columns:
    var(--spacing-unlock-art)
    minmax(0, 1fr)
    var(--spacing-unlock-state)
    var(--spacing-unlock-queue);
}
```

- [ ] **Step 2: The header — the template and the three cells**

In `UnlockTable.vue`, the header row:

```html
    <div
      class="grid grid-cols-unlock @max-compact/page:grid-cols-unlock-narrow items-center border-b border-hairline bg-muted text-label text-subtle-foreground"
    >
      <span />
      <span class="px-2 py-1.5">{{ t('unlock.columns.achievement') }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{ t('unlock.columns.unlocks') }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{ t('unlock.columns.condition') }}</span>
      <span class="px-2 py-1.5">{{ t('unlock.columns.state') }}</span>
      <span class="px-2 py-1.5 text-right @max-compact/page:hidden">{{
        t('unlock.columns.fanOut')
      }}</span>
      <span />
    </div>
```

and the row's class inside `cn(…)`:

```js
            'absolute inset-x-0 top-0 grid h-row-wide translate-y-(--row-start) grid-cols-unlock @max-compact/page:grid-cols-unlock-narrow items-center border-b border-hairline hover:bg-row-hover',
```

- [ ] **Step 3: The row's three cells**

In `UnlockRow.vue`, add `@max-compact/page:hidden` to exactly three cells — the third (`unlocks`),
the fourth (`condition`) and the sixth (`fanOut`):

```html
  <span class="flex min-w-0 items-center gap-2 px-2 @max-compact/page:hidden">
```

```html
  <span class="min-w-0 truncate px-2 text-caption text-foreground-soft @max-compact/page:hidden">
```

```html
  <span class="px-2 text-right text-row text-foreground tabular-nums @max-compact/page:hidden">{{
    node.done ? '—' : node.graph.fanOut
  }}</span>
```

- [ ] **Step 4: The rule that catches half the work**

In `checks`:

```js
  {
    // Spec 3.13a §7 and §9. It does not prove the right columns fell — nothing in a script can.
    // It proves both edits were made: a narrow template with no hidden cell is a grid that
    // dropped a track while every cell stayed, which slides the rest into the wrong columns.
    name: 'narrow grid template with no column hidden',
    test: (_f, body) =>
      /\bgrid-cols-[a-z-]+-narrow\b/.test(body) &&
      !/@max-compact\/page:hidden/.test(body),
  },
```

and two fixtures:

```js
  {
    name: 'a narrow template with its hidden cells is allowed',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="grid grid-cols-unlock @max-compact/page:grid-cols-unlock-narrow">\n    <span class="@max-compact/page:hidden" />\n  </div>\n</template>\n',
    expect: [],
  },
  {
    name: 'a narrow template with no hidden cell is half the work',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="grid grid-cols-unlock @max-compact/page:grid-cols-unlock-narrow" />\n</template>\n',
    expect: ['narrow grid template with no column hidden'],
  },
```

**Note the consequence**: the header and the row live in two files, so the rule holds for each of
them separately — `UnlockTable.vue` and `UnlockRow.vue` both carry a narrow template class and a
hidden cell, and both pass on their own.

- [ ] **Step 5: Run the scanner**

```
pnpm scan
```

Expected: `0 violations, … 28 fixtures`.

- [ ] **Step 6: Commit**

```bash
git add ui/src/assets/utilities.css ui/src/screens/unlock ui/scripts/scan-conventions.mjs
git commit -m "feat(ui): Unlock keeps its name and its state when the room runs out"
```

---

### Task 9: The Kit page shows the table at all three widths

**Files:**
- Create: `ui/src/kit/sections/app/WidthsSection.vue`
- Modify: `ui/src/kit/KitPage.vue` (import and place it)

**Interfaces:**
- Consumes: `UnlockTable` (Tasks 6, 8), `Threshold`/`ThresholdPx` (Task 2).
- Produces: nothing code reads. It produces the thing a person looks at.

**Why it matters here**: this is the only place a compact layout can be judged without dragging a
window, and it is where Task 2's provisional numbers get checked against the drawn thing.

- [ ] **Step 1: Write the section**

Create `ui/src/kit/sections/app/WidthsSection.vue`:

```vue
<script setup lang="ts">
import UnlockTable from '@/screens/unlock/UnlockTable.vue'
import type { UnlockNode } from '@/lib/ipc/types'
import KitSection from '../../KitSection.vue'

// Three boxes, each its own `page` container, so a compact layout can be judged without resizing
// the window — the thing container queries buy that media queries could not (spec 3.13a §11).
// The widths straddle the two thresholds: 400 is below compact, 800 between compact and regular,
// 1200 above regular.
const widths = [400, 800, 1200]

const node = (id: number, text: string, done: boolean): UnlockNode => ({
  achievement: {
    kind: 'known',
    id,
    text,
    condition: 'Sconfiggi Mom con Isaac',
    iconUrl: null,
  },
  done,
  unlocks: [],
  origin: null,
  missing: [],
  graph: {
    kind: 'computed',
    availableNow: !done,
    blockedBy: 0,
    fanOut: 3,
    stepsMissing: 0,
  },
})

const nodes = [
  node(1, 'Il Cubo di Ghiaccio', false),
  node(2, 'La Sacca del Diavolo', true),
  node(3, 'Un nome lungo che deve troncarsi quando la colonna si stringe', false),
]
</script>

<template>
  <KitSection title="Larghezze" class="col-span-3">
    <div class="flex items-start gap-4 overflow-x-auto">
      <div
        v-for="width in widths"
        :key="width"
        :style="{ '--kit-width': `${width}px` }"
        class="@container/page flex h-96 w-(--kit-width) shrink-0 flex-col border border-border"
      >
        <UnlockTable
          :nodes="nodes"
          :queued="new Set<number>()"
          :can-write="false"
          :busy="false"
          :offset="null"
        />
      </div>
    </div>
  </KitSection>
</template>
```

The width travels as a CSS variable bound by the template, never an inline pixel — frontend rule 1,
and the same shape `SectionSidebar` already uses for `--sidebar-width`.

- [ ] **Step 2: Place it on the page**

In `ui/src/kit/KitPage.vue`, add the import beside the other `sections/app/` ones and the element
near the top of the template, where the wide sections are.

- [ ] **Step 3: Look at the three boxes**

```
pnpm ui:dev
```
then `#kit`. What must be true: the 400 box shows four columns — drawing, name, state, queue — with
the name truncating; the 800 and 1200 boxes show all seven; no box has a horizontal scrollbar
inside it.

- [ ] **Step 4: Check the thresholds against the drawn thing**

This is the step Task 2 deferred. If the 400 box is cramped or the seven columns still fit
comfortably at some width below `--container-compact`, move the number — **in
`containers.css`, in `ThresholdPx`, and in `utilities.css`'s `sidebar-collapsed-hidden`**, which
carries the same 560 px literally. `pnpm ui:test` fails if the first two disagree; nothing checks
the third, so it is the one to check by hand.

- [ ] **Step 5: Commit**

```bash
pnpm typecheck && pnpm ui:test && pnpm scan
git add ui/src/kit
git commit -m "chore(ui): the Kit page draws a table at three widths at once"
```

---

### Task 10: The document

**Files:**
- Modify: `docs/frontend-conventions.md`

**Interfaces:** none.

- [ ] **Step 1: Write the section**

Add a section titled **"Responsive layout"** after the `<style>` blocks section, covering, each in
a paragraph or a table, with no more words than the thing needs:

- The two screen shapes, with the class list of each, and the `min-h-0` chain and why it fails
  silently.
- The two containers, `page` and `shell`, what each is measured against, and **why the sidebar's
  threshold hangs on `shell`** — on `page` it would oscillate.
- The three sizes, the rule that every variant names its container, and when a private token is
  allowed (measured on the component, with the scale in the comment).
- The column-drop pattern: the `@utility` pair, the hidden cells, and the rejected `0px` track with
  its accessibility reason.
- The window floor and the two places it is declared.
- A pointer to the spec for the reasoning, and to `scan-conventions.mjs` for the four rules.

Update the file's own structure listing if it enumerates `src/lib/` — `lib/design/thresholds.ts`
and `lib/window/windowFloor.ts` are new.

- [ ] **Step 2: Check the references**

```
node scripts/check-doc-refs.mjs
```

Expected: `0 new`. A `NEW` line is a path this section named that git does not track.

- [ ] **Step 3: Commit**

```bash
git add docs/frontend-conventions.md
git commit -m "docs: the responsive layout, its two shapes and its two containers"
```

---

### Task 11: The whole check, the floor, the board

**Files:**
- Modify: `scripts/test-floor`

- [ ] **Step 1: Run everything**

```
pnpm check
```

Expected: green, with one exception — the test-count floor prints that the UI suite grew and gives
the line to paste. It never fails on a rise.

- [ ] **Step 2: Raise the floor, with the deliberate line**

In `scripts/test-floor`, paste the new `UI_TESTS=` value and add the paragraph above it in the
file's own voice: 3.13a added five UI tests — two on the window floor (the constant, and the Tauri
config agreeing with it) and three on the thresholds (declared in the CSS at the px the constants
say, ascending, and leaving the window floor inside compact). The Rust count does not move.

- [ ] **Step 3: Run the check again**

```
pnpm check
```

Expected: fully green.

- [ ] **Step 4: Look at it in a window**

```
pnpm dev
```

Four things, none of which any test above can see:

1. Drag the window narrow to the floor. The sidebar becomes icons; Unlock keeps its name, state and
   queue button; nothing overlaps; the OS refuses to go below 640 × 480.
2. Drag it wide. The content reaches both edges; no cap; Plan's and Floor's asides move beside
   their content past `wide`.
3. `Ctrl` `+` twice. At 150 % the layout folds **earlier** than it did at 100 % — this is the whole
   argument for container queries, and it is the one thing a browser at one scale cannot show.
4. Tear a tab into a new window and try to shrink it. The floor holds there too.

- [ ] **Step 5: Commit and push**

```bash
git add scripts/test-floor
git commit -m "chore: the test floor rises by 3.13a's five UI tests"
git push -u origin feature/responsive-layout
```

- [ ] **Step 6: Move the card**

Card [#51](https://trello.com/c/lARifuay) → **UAT**, not Done. Tick the checklist items that are
really done. Leave open anything Step 4 could not settle, and comment what is still unverified and
what would settle it. Keep the `NEEDS WINDOW` label if any of the four in Step 4 went unlooked-at.

**Do not merge into `master`.** It is frozen (`CLAUDE.md`), and finishing a sub-project ends at
`develop`: merge there, push, stop.

---

## Self-review

**Spec coverage.** §1 is context, no task. §2 → Tasks 2, 5. §3 → Task 3. §4 → Tasks 3, 4, 6.
§5 → Tasks 2, 5. §6 → Task 7. §7 → Task 8. §8 → Task 1. §9 → Tasks 4, 5, 8. §10 → all.
§11 → Tasks 1, 2, 9, 11. §12's four open questions: the three thresholds are measured in Task 9
Step 4; Unlock's falling columns are fixed in Task 8 with the reasoning; `preview.ts`'s window is
**not** covered by any task — see below; the default `--container-*` scale is policed by Task 5's
rule rather than overwritten, which §12 allows and Task 5's comment records.

**One gap, left open on purpose.** §12 asks whether `ui/src/lib/window/preview.ts`'s window takes
the floor. No task decides it, because the answer needs somebody to open that window and see what it
is for. It is the first thing to settle in 3.13b, and it is written on the card rather than guessed
here.

**Type consistency.** `WindowFloor.Width`/`.Height` are used in Task 1 only. `Threshold` and
`ThresholdPx` are defined in Task 2 and read in Task 9. `SidebarItem`'s prop is `label: string` in
Task 7 Step 5 and is passed as `:label` in Step 6 at both call sites. `--container-compact` is
written as `560px` in three files — `containers.css`, `thresholds.ts`, and `utilities.css`'s
`sidebar-collapsed-hidden` — and Task 9 Step 4 names all three as the places to change together,
with the note that only the first two are checked by a test.
