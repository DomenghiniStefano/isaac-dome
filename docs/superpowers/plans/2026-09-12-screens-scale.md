# Cycle 3.5c — the interface scales: Implementation Plan

> **For agentic workers:** executed inline with superpowers:executing-plans, on the owner's
> delegation. Steps use checkbox (`- [ ]`) syntax. Every commit is pushed as it lands.
> Branch `feature/screens-scale`.

**Goal:** the whole interface scales over Discord's eleven steps, from a slider in Settings
and from `Ctrl` `+`/`-`/`0`, persisted in `settings.json` and applied before the first paint.

**Architecture:** two custom properties on `<html>` — `--app-scale`, the step as a factor,
and `--sprite-multiple`, an integer — carry everything: the root font size is
`calc(16px * var(--app-scale))`, every px token becomes rem, and the three tokens that *are*
pixel art derive from the multiple. The ladder is one list, written twice (Rust and
TypeScript) and pinned against each other by a test.

**Tech Stack:** Rust (`crates/ipc`, `crates/app`), Vue 3.5, TypeScript, Pinia, Tailwind v4,
Reka UI, vue-i18n, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-12-screens-scale-design.md`

## Global Constraints

- No hardcoded visual constants: the scale moves tokens, never values in a component.
- Degrade, never fail: an unreadable file, a value off the ladder, a failed command — all read as 100, none is an error page.
- A pixel sprite is drawn at a whole multiple of 32; any box that *is* a sprite derives its size from the multiple.
- IPC: `#[serde(rename_all = "camelCase")]`, the wire value is the percentage as a number; exhaustive matches; no `unwrap`/`panic` outside tests.
- Frontend: the five rules, `assertNever`, `as const`, every visible string through `useMessages()`.
- Checks judged by exit code; Cargo with `CARGO_BUILD_JOBS=4` on this machine.
- Commits: Conventional Commits, English, no `Co-Authored-By` or Claude reference, pushed after each task.

---

### Task 1: the ladder in Rust, and the setting

**Files:** Modify `crates/ipc/src/settings.rs`, `crates/ipc/src/lib.rs`; Create `crates/ipc/tests/settings.rs`

**Interfaces:** Produces `ipc::SCALE_PERCENTS: [u16; 11]`, `ipc::snap_percent(u16) -> u16`,
`Settings { active_profile_id, scale: u16 }` with `Default::default().scale == 100`.

- [ ] **Step 1: Failing tests** — `crates/ipc/tests/settings.rs`: the eleven steps in order (50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200); `snap_percent` keeps every step and answers 100 for 0, 137, 201 and `u16::MAX`; the default is 100; `to_value(Settings::default())` is `{"activeProfileId":null,"scale":100}`; `serde_json::from_str::<Settings>("{}")` has scale 100; a file holding `"scale": 137` reads 137 but `snap_percent` on it gives 100, and the accessor the commands use returns 100.
- [ ] **Step 2: Run** `cargo test -p ipc --test settings` — expected: compile error.
- [ ] **Step 3: Implement** — the constant, `snap_percent` (an exact membership test, never a nearest-neighbour search: a value off the ladder is a file we didn't write), the field with `#[serde(default = "default_scale")]`, and `Settings::scale()` returning the snapped value so no caller can forget.
- [ ] **Step 4: Run** the test and `cargo clippy -p ipc --all-targets -- -D warnings`.
- [ ] **Step 5: Commit** — `feat(ipc): the interface scale, eleven steps and a setting that snaps to them`

---

### Task 2: the two commands

**Files:** Modify `crates/app/src/lib.rs`

**Interfaces:** Produces `settings() -> Result<ipc::Settings, IpcError>` and
`set_scale(percent: u16) -> Result<ipc::Settings, IpcError>`; both answer the settings as
they are after the call, with the scale snapped.

- [ ] **Step 1: Implement** both commands beside `select_profile`, which is the pattern: load, change one field with `#[allow(clippy::needless_update)]` and the spread, save, answer. `set_scale` snaps before writing, so the file never holds a value the app would refuse.
- [ ] **Step 2:** Register both in `generate_handler!`.
- [ ] **Step 3: Run** `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`.
- [ ] **Step 4: Commit** — `feat(app): the settings and set_scale commands`

---

### Task 3: the ladder in TypeScript, and the properties

**Files:** Create `ui/src/lib/scale/{steps,apply,shortcut}.ts` and their tests; Modify `ui/src/lib/ipc/{types,settings}.ts`, `ui/src/lib/constants/{commands,keyNames}.ts`

**Interfaces:** Produces `scalePercents: number[]`, `DefaultScale = 100`, `snapPercent`,
`nextPercent`, `previousPercent`, `scaleFactor`, `spriteMultiple`,
`scaleProperties(percent): Record<string, string>`, `shortcutAction(event): 'in' | 'out' |
'reset' | null`; `settings()` and `setScale(percent)` wrappers; `Settings` type.

- [ ] **Step 1: Failing tests** — `steps.test.ts` reads `crates/ipc/src/settings.rs?raw` and asserts the eleven numbers there are exactly `scalePercents`, in order (two ladders that drift are a slider that saves a value the backend refuses); `snapPercent` as in Rust; `nextPercent(200) === 200` and `previousPercent(50) === 50`; `spriteMultiple` over all eleven is `[1,1,2,2,2,2,2,3,3,4,4]`. `apply.test.ts`: `scaleProperties(125)` is `{'--app-scale':'1.25','--sprite-multiple':'3'}`. `shortcut.test.ts`: `Ctrl` with `+`, `=`, `-`, `0` and a bare `+`.
- [ ] **Step 2: Run** `pnpm --filter ui exec vitest run src/lib/scale` — expected: modules missing.
- [ ] **Step 3: Implement** the three modules and the IPC wrappers.
- [ ] **Step 4: Run** the tests, `pnpm typecheck`, `pnpm lint`.
- [ ] **Step 5: Commit** — `feat(ui): the scale ladder, its properties and its shortcut, as pure functions`

---

### Task 4: the token rewrite

**Files:** Modify `ui/src/assets/base.css`, `ui/src/assets/theme/{spacing,typography}.css`; Modify `ui/src/assets/base.test.ts`; Create `ui/src/assets/scale.test.ts`

- [ ] **Step 1: Failing tests** — rewrite `base.test.ts`: `html` now *has* a `font-size` and it is `calc(16px * var(--app-scale))`, with the invariant restated in the comment (at factor 1 the root is 16px, so a spacing step is 4px and `--text-body` is 14px). Add `scale.test.ts`: every `--spacing-*` and `--text-*` declaration in `theme/spacing.css` and `theme/typography.css` is in `rem`, except the three sprite tokens (which are `calc(… var(--sprite-multiple))`) and `--spacing-scrollbar`, which must carry a comment.
- [ ] **Step 2: Run** — expected: both fail.
- [ ] **Step 3: Implement** — `base.css` gets the root font size with the comment of Decision 2; every px token in `spacing.css` becomes rem at 16px (`38px` → `2.375rem`), the three sprite tokens become `calc()` on the multiple, `--spacing-scrollbar` stays with its reason; `typography.css`'s scale becomes rem with the amended comment. `radius.css` and `containers.css` keep px and gain their reason.
- [ ] **Step 4: Run** `pnpm ui:test`, `pnpm typecheck`, `pnpm lint`, `pnpm format:check`, `pnpm scan`.
- [ ] **Step 5: Commit** — `refactor(ui): every size token in rem, so the root font size is the scale`

---

### Task 5: the store, the entry point, the shortcut

**Files:** Create `ui/src/stores/settings.ts`, `ui/src/composables/useShortcut.ts`; Modify `ui/src/main.ts`, `ui/src/App.vue`, `ui/src/lib/constants/stores.ts`, `ui/src/lib/ipc/fixtures/{index,settings}.ts`

**Interfaces:** Produces `useSettingsStore()` with `scale`, `status`, `error`, `load()`,
`setScale(percent)`, `step(direction)`; `applyScale(percent)` writing the properties on
`document.documentElement`; the fixture answering 100 and remembering changes.

- [ ] **Step 1:** `applyScale` in `lib/scale/apply.ts` (the properties on the root element), called by the store on every change and by `main.ts` before mounting.
- [ ] **Step 2:** `main.ts` awaits `settings()` and applies the scale **before** `mount`, inside a `try`: a failed command mounts the app at 100, it never blocks the window. The splash covers the wait (B18).
- [ ] **Step 3:** the store, and `useShortcut` on `window` `keydown` calling it; mounted once in `App.vue`.
- [ ] **Step 4:** the fixture: `settings` and `set_scale` answer from a module-level value, like `chosenId` in `profile.ts`.
- [ ] **Step 5: Run** the frontend checks.
- [ ] **Step 6: Commit** — `feat(ui): the scale is applied before the first paint, and Ctrl +/- moves it`

---

### Task 6: the Slider primitive

**Files:** Create `ui/src/components/ui/slider/{Slider.vue,index.ts}`, `ui/src/kit/sections/SliderSection.vue`; Modify `ui/src/kit/KitPage.vue`

- [ ] **Step 1:** `Slider.vue` wrapping Reka's `SliderRoot`/`SliderTrack`/`SliderRange`/`SliderThumb`, styled by the kit: flat track (`bg-secondary`), the range in `primary`, a square thumb (`size-3.5 bg-foreground`), the focus ring from `:focus-visible`, no radius and no transition.
- [ ] **Step 2:** the Kit row: a slider over five steps with its ticks, and a disabled one.
- [ ] **Step 3: Run** the frontend checks; look at the Kit page through Playwright.
- [ ] **Step 4: Commit** — `feat(ui): a Slider primitive, flat track and a square thumb`

---

### Task 7: the Appearance screen

**Files:** Create `ui/src/screens/AppearanceScreen.vue`, `ui/src/screens/appearance/{ScaleSlider,ScalePreview}.vue`; Modify `ui/src/router/{routeTable,routes}.ts`, `ui/src/components/shell/sectionNav.ts`, `ui/src/i18n/messages/{it,en}.ts`

- [ ] **Step 1:** the route `Appearance` (`/settings/appearance`, `TabOrigin.Settings`, no profile needed), third entry of the Settings sidebar.
- [ ] **Step 2:** `ScalePreview`: a `Card` with a KPI tile, a matrix cell with its sprite, an achievement row and a button — the app's own components, `sticky top-0` so it stays in view while the page scrolls.
- [ ] **Step 3:** `ScaleSlider`: the slider over step indices 0..10, the percentage above the current tick in the accent colour, the eleven values as marks, and the `Ctrl` hint as `Kbd`s. A failed write shows an `Alert` and the size stays applied.
- [ ] **Step 4:** i18n both languages.
- [ ] **Step 5: Run** the frontend checks.
- [ ] **Step 6: Commit** — `feat(ui): the Appearance screen, the slider and its pinned preview`

---

### Task 8: the scanner's rule, the look, the documents

- [ ] **Step 1:** `ui/scripts/scan-conventions.mjs` gains a rule: a `px` value in `theme/` needs a comment on the line above or on the same line (the exceptions of Decision 4 all have one). Add it to `docs/frontend-conventions.md` too, as rule 2's companion.
- [ ] **Step 2:** Through Playwright on `pnpm ui:dev`: the slider at 50, 100 and 200 with the matrix, a table and a wiki page open; the sprite measured at both ends (32px at 50, 128px at 200); the preview staying in view; `Ctrl` `+` from another screen; a reload keeping the value.
- [ ] **Step 3:** `sh scripts/check` green, skips listed.
- [ ] **Step 4:** `docs/STATUS.md` (3.5c ticked, a session log entry), `docs/BACKLOG.md` (B26 closed), `DESIGN-BRIEF.md` (the setting on the wire).
- [ ] **Step 5: Commit** — `docs: cycle 3.5c lands, the interface scales`; then merge into `develop` with `--no-ff`.
