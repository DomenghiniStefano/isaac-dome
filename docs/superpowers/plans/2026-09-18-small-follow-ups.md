# Small follow-ups Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this
> plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the mechanical half of `B12` (design system cycle 1 follow-ups) and `B65` (the
palette swallows `Ctrl+Enter`), each as its own commit, without opening a design question.

**Architecture:** Eight independent changes in `ui/`, ordered cheapest first. Nothing crosses the
IPC boundary and no Rust crate is touched. Where a change lives inside a component — which this
repo cannot mount in a test — the part worth checking is extracted as a pure function beside it
and tested there, the way `progress/progressShares.ts` already is.

**Tech Stack:** Vue 3.5 + TypeScript, Vite 8, Tailwind v4, shadcn-vue on Reka UI 2.10.4,
tailwind-merge 3.6.0, vue-i18n 11.4.10, Vitest.

**Spec:** none of its own — the two backlog entries are the spec: `docs/BACKLOG.md` **B12** (the
numbered list of ten) and **B65** (cause, prediction and *Closes when*).

## Global Constraints

- **No `<style>` in SFCs**, no hardcoded visual constants, no `invoke()` outside `src/lib/ipc/`,
  no raw `<button>`/`<input>` outside `src/components/ui/`, no string unions — the five rules of
  `CLAUDE.md`, enforced by `pnpm scan`.
- **Every new scanner rule is added with its fixtures**, in the `FIXTURES` array: a rule with no
  fixture is a rule nobody can see working.
- **Test-first where a test can exist.** `ui/` has no `@vue/test-utils` and no DOM environment:
  no test mounts a component. A change inside a `.vue` file is therefore verified by `pnpm scan`,
  `pnpm typecheck` and a window — and its decidable half is moved into a `.ts` file that Vitest
  can reach.
- **The test floor moves in the same commit as the tests**: `scripts/test-floor`, `UI_TESTS`.
- **One commit per task**, Conventional Commits, scope `ui`. No `Co-Authored-By` trailer.
- B12's items **3** and **10** (contrast, and a disabled segmented control losing its on-state)
  are **out of scope**: both are "back to design", and neither can be decided without looking at
  a window.

---

### Task 1: `cn()` knows the opacity tokens (B12, item 1)

**Files:**
- Modify: `ui/src/lib/design/themeKeys.ts` — one entry in `ThemeNamespace`
- Modify: `ui/src/lib/cn.ts` — one class group
- Test: `ui/src/lib/cn.test.ts`

**Interfaces:**
- Consumes: `themeKeys(css, namespace)`, already exported.
- Produces: nothing new. `cn('opacity-muted', 'opacity-disabled')` starts answering
  `'opacity-disabled'`.

- [ ] **Step 1: Write the failing test** — in `ui/src/lib/cn.test.ts`, beside the duration one:

```ts
  it('lets a later opacity token replace an earlier one', () => {
    expect(cn('opacity-muted', 'opacity-disabled')).toBe('opacity-disabled')
  })

  it('still lets a numeric opacity replace a token', () => {
    expect(cn('opacity-muted', 'opacity-0')).toBe('opacity-0')
  })
```

- [ ] **Step 2: Run it and watch it fail** — `pnpm --dir ui test -- --run src/lib/cn.test.ts`.
      Expected: the first fails with `'opacity-muted opacity-disabled'`; the second passes
      already, because tailwind-merge knows numbers.

- [ ] **Step 3: Add the namespace** — `ui/src/lib/design/themeKeys.ts`, in `ThemeNamespace`:
      `Opacity: 'opacity',`

- [ ] **Step 4: Extend the class group** — `ui/src/lib/cn.ts`: import
      `opacity from '@/assets/theme/opacity.css?raw'` and, beside `duration`, a group whose
      members are the token names, so `opacity-0` still wins over `opacity-muted`.

- [ ] **Step 5: Green** — `pnpm --dir ui test -- --run src/lib/cn.test.ts`. Both pass.

- [ ] **Step 6: Raise the floor and commit**

```bash
git add ui/src/lib/cn.ts ui/src/lib/cn.test.ts ui/src/lib/design/themeKeys.ts scripts/test-floor
git commit -m "fix(ui): cn() keeps the last opacity token instead of both"
```

---

### Task 2: the vue-i18n build flags are declared (B12, item 2)

**Files:**
- Modify: `ui/vite.config.ts` — a `define` block
- Test: none possible (a build flag has no unit test); verified by `pnpm build` and by grepping
  the bundle.

- [ ] **Step 1: Read the flags from the official documentation** — the names, the values and
      whether dropping the message compiler requires pre-compiled messages. Our messages are
      plain TS objects with `{named}` interpolation, **not** pre-compiled, so the compiler flag
      is only safe if the documentation says so.

- [ ] **Step 2: Add the `define` block** to `ui/vite.config.ts`, each flag with the sentence that
      says what it costs.

- [ ] **Step 3: Verify it took** — `pnpm --dir ui build`, then grep the emitted bundle for the
      legacy API's own symbols. Expected: the flag is inlined and the legacy branch is gone.

- [ ] **Step 4: Commit**

```bash
git add ui/vite.config.ts
git commit -m "build(ui): declare vue-i18n's feature flags so the legacy API leaves the bundle"
```

---

### Task 3: `Button` is a button, not a submit (B12, item 7)

**Files:**
- Create: `ui/src/components/ui/button/buttonType.ts`
- Create: `ui/src/components/ui/button/buttonType.test.ts`
- Modify: `ui/src/components/ui/button/Button.vue`, `ui/src/components/ui/button/index.ts`

**Interfaces:**
- Produces: `buttonType(as: unknown, asChild: boolean | undefined): 'button' | undefined`.

- [ ] **Step 1: Write the failing test** — `ui/src/components/ui/button/buttonType.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { buttonType } from './buttonType'

describe('buttonType', () => {
  it('types a plain button, so a form does not submit on every click', () => {
    expect(buttonType('button', undefined)).toBe('button')
  })

  it('leaves an anchor alone: type is not an anchor attribute', () => {
    expect(buttonType('a', undefined)).toBeUndefined()
  })

  it('leaves the child alone: as-child renders what the caller passed', () => {
    expect(buttonType('button', true)).toBeUndefined()
  })
})
```

- [ ] **Step 2: Run it and watch it fail** —
      `pnpm --dir ui test -- --run src/components/ui/button/buttonType.test.ts`.
      Expected: FAIL, `buttonType` does not exist.

- [ ] **Step 3: Write it** — `ui/src/components/ui/button/buttonType.ts`, with the reason in a
      comment: HTML's default is `type="submit"`, so inside a form every Button that does not say
      otherwise submits it; and the default only applies when we are the ones rendering the
      element, because `as-child` renders the caller's node.

- [ ] **Step 4: Green** — same command. PASS.

- [ ] **Step 5: Use it in the component** — `Button.vue`: a computed `type` bound as `:type`, with
      the note that a caller's own `type="submit"` still wins, a fallthrough attribute being
      merged after the component's own bindings.

- [ ] **Step 6: Export it** — `index.ts`: `export { buttonType } from './buttonType'`.

- [ ] **Step 7: Gate and commit**

```bash
pnpm --dir ui test -- --run && pnpm typecheck && pnpm scan
git add ui/src/components/ui/button scripts/test-floor
git commit -m "fix(ui): a Button is type=button unless the caller says otherwise"
```

---

### Task 4: a field's hover is a field's own colour (B12, item 6)

**Files:**
- Modify: `ui/src/assets/theme/colors.css` — one token
- Modify: `ui/src/components/ui/select/SelectTrigger.vue` — one class

**Interfaces:** none. `--color-field-hover` joins the role tokens.

- [ ] **Step 1: Add the token** beside `--color-row-hover` in `colors.css`, with the sentence: a
      field's hover is a row's *by value* and never *by role* — a Select's trigger is not a row,
      and the day a row's hover moves the fields must not move with it.

- [ ] **Step 2: Use it** — in `SelectTrigger.vue`, `hover:bg-row-hover` becomes
      `hover:bg-field-hover`.

- [ ] **Step 3: Gate** — `pnpm scan && pnpm typecheck`. The Kit page (`pnpm ui:dev`, `#kit`) is
      where it is looked at; nothing red can say whether the colour is right, and it is the same
      value it was.

- [ ] **Step 4: Commit**

```bash
git add ui/src/assets/theme/colors.css ui/src/components/ui/select/SelectTrigger.vue
git commit -m "refactor(ui): a field's hover has a field's own token"
```

---

### Task 5: a diagnostic is a status, not an alert (B12, item 8)

**Files:**
- Modify: `ui/src/components/ui/alert/variants.ts` — an `AlertLive` constant
- Modify: `ui/src/components/ui/alert/Alert.vue`, `ui/src/components/ui/alert/index.ts`
- Modify: the callers that announce a *result* rather than a state, if any

**Interfaces:**
- Produces: `AlertLive = { Polite: 'status', Assertive: 'alert' } as const`, and `Alert`'s
  `live?: AlertLive` prop, default `AlertLive.Polite`.

- [ ] **Step 1: Read every caller** — `grep -rn '<Alert' ui/src --include=*.vue`. An Alert drawn
      **with the page** (a diagnostic the screen was born with) is polite; one that appears **in
      answer to an action** is assertive. Write the answer per caller into the commit body.

- [ ] **Step 2: Add the constant** to `variants.ts`, with the reason: `role="alert"` is an
      assertive live region, which is right for the answer to an action and wrong for a
      diagnostic that was on the page before the reader got there.

- [ ] **Step 3: Use it** in `Alert.vue`: a `live?: AlertLive` prop with
      `withDefaults(..., { live: AlertLive.Polite })`, and `:role="live"` on the `div`.

- [ ] **Step 4: Export** `AlertLive` from `index.ts`.

- [ ] **Step 5: Gate and commit**

```bash
pnpm typecheck && pnpm scan
git add ui/src/components/ui/alert
git commit -m "fix(ui): an Alert drawn with its page is a status, not an interruption"
```

---

### Task 6: the bar says its unreadable share out loud (B12, item 9)

**Files:**
- Modify: `ui/src/components/ui/progress/Progress.vue`
- Modify: `ui/src/components/kpi/KpiTile.vue` — the one caller that has the numbers and i18n
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`

**Interfaces:**
- Produces: `Progress`'s `valueText?: string` prop, handed to Reka's `getValueLabel`.

- [ ] **Step 1: Take the prop** — `Progress.vue`: `valueText?: string` in the props, omitted from
      `delegatedProps`, and a computed `getValueLabel` that is `undefined` when the caller said
      nothing, so Reka keeps its own default. The words are the caller's: a primitive holds no
      string, and only the screen knows what the unreadable part of its own bar is.

- [ ] **Step 2: Give it the words** — a message key per language, e.g.
      `kpi.barValueText: '{done} di {total}, {unknown} non leggibili'` /
      `'{done} of {total}, {unknown} unreadable'`, and `KpiTile.vue` passes
      `:value-text="t('kpi.barValueText', { … })"`.

- [ ] **Step 3: Gate** — `pnpm typecheck && pnpm scan && pnpm --dir ui test -- --run`.

- [ ] **Step 4: Commit**

```bash
git add ui/src/components/ui/progress ui/src/components/kpi/KpiTile.vue ui/src/i18n/messages
git commit -m "feat(ui): the progress bar tells assistive tech what it cannot read"
```

---

### Task 7: the scanner's four gaps (B12, item 5)

**Files:**
- Modify: `ui/scripts/scan-conventions.mjs` — three rules changed or added, and their fixtures
- Modify: `ui/src/screens/goals/WantAnswer.vue` — the two `text-sm` the new rule finds

**Interfaces:** none. Every rule comes with at least one fixture that must be caught and one
that must not.

- [ ] **Step 1: The literal-attribute rule learns more attributes.** `position`, `align` and
      `side` have constants (`ui/src/lib/constants/placement.ts`) and the rule does not look at
      them; `tone` and `live` are props of `Progress` and `Alert`. Measured before writing:
      **no file outside `src/components/ui/` uses any of them literally today**, so the rule
      starts green and is a guard, not a cleanup.

- [ ] **Step 2: The `dark:` rule learns stacked variants.** `hover:dark:bg-x` is invisible to the
      current pattern, because `dark:` is not at the start of the class: allow a chain of
      variants before it.

- [ ] **Step 3: `outline-none` outside a primitive.** In `src/components/ui/` a highlight
      replaces the focus ring and Reka moves DOM focus through the list — four files do it on
      purpose. Outside it, removing the outline removes the only thing that says where the
      keyboard is.

- [ ] **Step 4: Classes of the reset scales.** `@theme` resets `--text-*`, `--font-weight-*`,
      `--radius-*` and `--shadow-*` to `initial`, so `text-sm`, `font-bold`, `rounded-md` and
      every `shadow-*` **generate nothing at all**. `rounded-full` and `rounded-none` are not
      theme values and stay.

- [ ] **Step 5: A fixture per rule**, added to `FIXTURES`: a stacked `dark:` that must be caught,
      a literal `side` on a primitive, `outline-none` in a screen (caught) and in a primitive
      (allowed), `text-sm` (caught) and `rounded-full` (allowed).

- [ ] **Step 6: Run the scanner and fix what it finds** — `pnpm scan`. Expected: two violations
      in `ui/src/screens/goals/WantAnswer.vue`, both `text-sm`, which have been generating
      nothing since the scale was reset. Replace each with the token the surrounding text uses
      (`text-body` or `text-row` — read the block, do not guess).

- [ ] **Step 7: Commit**

```bash
git add ui/scripts/scan-conventions.mjs ui/src/screens/goals/WantAnswer.vue
git commit -m "fix(ui): the scanner sees stacked dark:, lost outlines and dead default scales"
```

---

### Task 8: the palette owns Enter (B65)

**Files:**
- Create: `ui/src/lib/search/highlight.ts`
- Create: `ui/src/lib/search/highlight.test.ts`
- Modify: `ui/src/components/search/SearchPalette.vue`
- Modify: `ui/src/components/ui/command/Command.vue`, `CommandDialog.vue` (expose the highlight)

**Interfaces:**
- Produces: `keyAfterAnswer(rows: SearchRow[], highlighted: string | null): string | null` —
  which row the highlight sits on once an answer lands: the one it was on if it survived, else
  the first row, else nothing.

**What the decision is, written here because B65 asks for it:** *the palette owns Enter.* The
listbox keeps the highlight — it is the one that draws it and moves it with the arrows — and the
palette keeps **what Enter does with it**, for `Enter` and `Ctrl+Enter` together. The library's
own Enter path is left in place for the plain key; what the palette adds is (a) a `keydown`
handler that runs **before** the listbox and answers `Ctrl+Enter` itself, since
`ListboxRoot.onKeydownEnter` returns early on a modifier and never clicks the row, and (b) a
re-highlight when an answer lands, because `ListboxFilter` highlights the first row on every
keystroke and the rows for that keystroke arrive 120 ms later — the highlight it placed is on a
row that is about to unmount, which is what makes plain Enter do nothing.

- [ ] **Step 1: Write the failing test** — `ui/src/lib/search/highlight.test.ts`, four cases: the
      row the user moved to survives the new answer and is kept; the row it was on has gone and
      the first row takes it; an empty answer highlights nothing; nothing highlighted yet takes
      the first row. `SearchRow`'s real shape is read from `ui/src/lib/search/rows.ts` before
      writing the test — the rows it builds must be real ones.

- [ ] **Step 2: Run it and watch it fail** —
      `pnpm --dir ui test -- --run src/lib/search/highlight.test.ts`. FAIL: no such module.

- [ ] **Step 3: Write it** — `ui/src/lib/search/highlight.ts`, with the reason in a comment: the
      rows are replaced wholesale 120 ms after a keystroke and the listbox highlighted the first
      row of the *previous* answer, so keeping the user's own position when it survived is the
      part a library cannot know.

- [ ] **Step 4: Green** — same command. PASS.

- [ ] **Step 5: Expose the highlight through the two wrappers.** `Command.vue` holds a ref on
      `ListboxRoot` and `defineExpose({ highlightItem })`; `CommandDialog.vue` holds a ref on
      `Command` and re-exposes the same one function. Nothing else leaves the primitive.

- [ ] **Step 6: The palette tracks and drives the highlight** — in `SearchPalette.vue`:
      `@highlight` on the dialog updates `highlighted` with the item's value; a `watch` on `rows`
      calls `keyAfterAnswer` and hands the answer to the exposed `highlightItem`.

- [ ] **Step 7: The palette answers `Ctrl+Enter` itself** — a `keydown` handler in the capture
      phase on the dialog's content: on `Enter` **with** `ctrl` or `meta`, `preventDefault`,
      `stopPropagation`, find the row whose key is `highlighted`, and `openAt(row.location)`.
      Plain `Enter` is left to the listbox, which clicks the row and reaches `@select`.

- [ ] **Step 8: Gate and commit**

```bash
pnpm --dir ui test -- --run && pnpm typecheck && pnpm scan
git add ui/src/lib/search ui/src/components/search/SearchPalette.vue ui/src/components/ui/command scripts/test-floor
git commit -m "fix(ui): the palette owns Enter, so Ctrl+Enter opens the row beside the tab"
```

- [ ] **Step 9: Say what is still unverified.** Both halves of B65 end in a window: `Ctrl+Enter`
      opening beside the active tab, and plain `Enter` on a row that arrived after the debounce.
      The card goes to `UAT` with `NEEDS WINDOW`, and the comment names the two gestures to try.

---

## What this plan does not do

- **B12 items 3 and 10** — keyboard-highlight contrast in Select and Command, and a disabled
  segmented control that loses its on-state. Both are "back to design": they need a look at a
  window before anything is decided, and a value chosen without one would be a guess with a
  commit behind it.
- **B12 item 4**, the Command filter's remaining half: `CommandItem` still does not prune its id
  from its group's set on unmount, and `CommandInput` always auto-focuses. The palette's own half
  of that entry — items mounting after the search changed — was closed when `Command.vue` started
  watching `allItems.size`.
- **B66**, the Completion screen's `hard`: it is an analysis entry, not an implementation. Either
  bit 1 is measured outside Greed or the wording steps back to `MarkLevel::Second`'s restraint,
  and that is the owner's call.
