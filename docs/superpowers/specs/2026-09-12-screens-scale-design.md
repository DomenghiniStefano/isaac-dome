# Design system, cycle 3.5c — the interface scales (design)

**Date:** 2026-09-12
**Milestone:** design system (`docs/STATUS.md`), cycle 3 of 3, pulled ahead of 3.6
**Branch:** `feature/screens-scale`, cut from `develop`
**Depends on:** cycle 1's tokens (`ui/src/assets/theme/`), 3.1's shell and settings section;
`docs/BACKLOG.md` B26, which carries the owner's decisions and the reference (Discord's
*Livello di zoom*, four screenshots handed over on 2026-09-12); B18's splash, which is the
moment the scale is applied
**Numbered `c`** because `b` was already the search half of 3.5, which this precedes: a
screen built before the tokens scale is a screen built on values that don't.
**Status:** the mechanism, the ladder and the control were decided by the owner in B26. The
decisions below are the ones the entry left to execution, each marked **(delegated)** so the
first look can overturn it cheaply.

## What this sub-project is

The user picks the size of the whole interface from Settings, on **Discord's eleven steps**
— 50 · 67 · 75 · 80 · 90 · 100 · 110 · 125 · 150 · 175 · 200 — with a slider, with
`Ctrl` `+` / `Ctrl` `-` / `Ctrl` `0`, and with a preview card pinned at the top of the page.
The value persists in `settings.json` and is applied before the first paint.

It goes before every remaining screen because the way it works is **a rewrite of the token
files**: the 38 px tokens become rem, the root font size becomes the scale, and pixel art
gets a rule of its own. Nothing in a component changes, which is exactly why it is cheap
now and expensive after cycle 3.

## Applicable constraints

1. **No hardcoded visual constants** (frontend rule 2). The scale moves tokens, never
   values written in a component; the scanner gains a rule so a px token needs a reason.
2. **Degrade, never fail.** A settings file that can't be read, or holds a value that isn't
   on the ladder, means 100 — never an error page, never a refusal to start.
3. **Only resolved view-models cross the IPC**: the wire value is the percentage as a
   number, validated on both sides by the same ladder.
4. **A pixel sprite is never drawn at a fraction of its size** (§5.6: the game's art is
   pixel art, `image-rendering: pixelated`).
5. **The frontend rules** hold; the new `Slider` is a kit primitive with its Kit page row.

## Decision 1 — a root scale, not the webview's zoom (the owner's, recorded)

`getCurrentWebview().setZoom()` would be one call, but the root scale is the one to take:
it keeps the rules inspectable in CSS, works on the Kit page and on fixtures, survives a
torn-off window (B15) with no re-apply, and lets the sprites stay on whole multiples where
zoom would smear them. The costs are one pass over the token files and a test that no px
token survives without a reason.

## Decision 2 — the ladder and the two variables (delegated)

Two custom properties on `<html>`, and nothing else moves:

```
--app-scale         1.25      the step, as a factor
--sprite-multiple   3         how many times a 32px sprite is drawn, an integer
```

- `html { font-size: calc(16px * var(--app-scale)) }` in `base.css`. Tailwind's spacing is
  `calc(var(--spacing) * n)` with `--spacing: 0.25rem`, so **every spacing step follows the
  root** the moment the tokens are rem — which is what `base.test.ts` was written to
  prevent and now has to be rewritten to require. The invariant it keeps is the same one,
  restated: **at scale 1 a spacing step is 4px and `--text-body` is 14px**.
- **16px is the base, not the browser's own root size.** Reading the browser's setting and
  multiplying by it would make the same step mean two sizes on two machines, and the app's
  whole geometry is drawn from `Schermate.dc.html` at 16px. The user's size is the slider.
- The eleven steps live in `ui/src/lib/scale/steps.ts` as `const ScalePercent = { … } as
  const` plus `scalePercents: number[]` in order; the wire value is the percentage
  (`125`), never the factor, so the file the user can open reads as a percentage.
- `scaleFactor(percent) = percent / 100`, `snapPercent(raw)` returns the nearest step **only
  when `raw` is one of them**, and 100 otherwise: a value that isn't on the ladder is a file
  we didn't write, and we read it as the default rather than inventing a twelfth step.
- `nextPercent` / `previousPercent` walk the ladder and stop at its ends.

## Decision 3 — pixel art keeps whole multiples (delegated)

`spriteMultiple(percent) = max(1, round(2 × factor))` — the owner's rule, so a 32px sprite
is 2x at 100, 3x at 125 and 150, 4x at 175 and 200, 1x at 50 and 67. The multiple is an
integer written on the root, and **every box that holds pixel art derives its size from the
sprite token**, never from its own rem value:

| token | today | after |
|---|---|---|
| `--spacing-sprite` | `4rem` | `calc(32px * var(--sprite-multiple))` |
| `--spacing-mark-symbol` | `32px` | `calc(16px * var(--sprite-multiple))` |
| `--spacing-mark-cell` | `40px` | `calc(var(--spacing-mark-symbol) * 1.25)` |

The reason the derivation is mandatory: at 125 the multiple rounds **up** (3x, 96px, where
the continuous size would be 80px), so a cell sized in rem would be smaller than the symbol
inside it. Where a sprite sits inside a row whose height is rem (the tables' 40px rows), the
sprite keeps a `size-8`-style utility and is not on this rule: it is drawn small enough that
a fractional multiple doesn't read, and the row's height is the constraint. **The rule
applies to the three tokens above**, which are the places a sprite is the box.

`--spacing-achievement` and `--spacing-wiki-figure` stay rem: an achievement drawing is a
painted 263×176 image, not pixel art (§5.6), and the wiki figure holds a portrait.

## Decision 4 — the token rewrite (delegated)

Every px token becomes rem at 16px (`38px` → `2.375rem`), **except** these, which stay px
with a comment saying why:

- **Hairlines and borders**: nothing in the tokens today (borders are Tailwind's `border`,
  1px, and they stay 1px at every scale — a scaled hairline is a blurry hairline).
- **`--radius-cell: 2px` and `--radius-input: 4px`**: a radius of 2.5px is a smudge, and the
  skin is flat on purpose.
- **`--spacing-scrollbar: 12px`**: the scrollbar is the OS's furniture in the app's clothes;
  it keeps one size.
- **The three sprite tokens of Decision 3**, which are `calc()` on the multiple.
- **`--container-tab-narrow: 64px`**: a container query's breakpoint is compared against a
  used width in px, and a rem there would be read against the root at query time — it works,
  but the number means "the width at which a tab drops its icon", which is a fact about the
  drawn tab. It scales with the tab, so it stays px and is measured at scale 1.

`--text-*` all become rem (`14px` → `0.875rem`), which is what makes the type scale follow.
The comment that says "a pixel font is crisp on whole pixels" stays, amended: it is crisp at
the steps that give whole pixels, and the browser rounds per element at the others — exactly
as Discord's text does at 67 or 125.

## Decision 5 — where the value lives, and when it is applied (delegated)

- **`ipc::Settings` grows `scale: u16`**, default 100, `#[serde(default)]` like the rest, and
  every read goes through `snap_percent` so a hand-edited file can't put the app at 137%.
  The Rust ladder and the TypeScript one are the same eleven numbers, each with its own test;
  a third test pins the JSON shape.
- **Two commands**: `settings() -> Settings` and `set_scale(percent: u16) -> Settings`. The
  second writes the file and answers the settings as they now are — the same shape as
  `select_profile`, which already writes and answers.
- **Applied before the first paint**: `main.ts` reads `settings()` and writes the two custom
  properties on `document.documentElement` **before** mounting the app. The splash of B18 is
  what the user sees meanwhile, so the window never shows the shell at the wrong size and
  then jump. A command that fails at startup is not fatal: the app mounts at 100.
- **On the Kit page too**: the Kit mounts through the same entry, so it gets the scale for
  free, which is half the reason for the root-scale mechanism.

## Decision 6 — the control (delegated)

- **A `Slider` primitive** in `components/ui/slider/`, Reka's `SliderRoot` styled by the kit:
  a flat track, a square thumb, no radius, no transition, the ticks as marks under the track
  and the current value in the accent colour above its tick. Props are Reka's (`min`, `max`,
  `step`, `modelValue`); the app drives it over **step indices**, 0 to 10, not percentages,
  so the ladder's uneven spacing is the slider's even spacing — which is what Discord draws.
- **A screen, `Appearance`** (`/settings/appearance`, `routes.appearance`), third entry of the
  Settings sidebar after Profile and Tabs. It carries the preview card, the slider, and the
  shortcut hint; the rest of Settings is 3.6's.
- **The preview card is pinned at the top** (`sticky top-0`), like Discord's: a slice of real
  UI drawn at the chosen size — a KPI tile, a matrix cell with its sprite, a row with an
  achievement drawing, a button, a badge. It is the app's own components, not a picture of
  them, so it scales because everything does.
- **`Ctrl` `+` / `Ctrl` `-` / `Ctrl` `0`** from anywhere in the app, on the same steps and
  the same persisted value: a `useShortcut` composable on `window`, `keydown`,
  `preventDefault` (the webview's own zoom must not also fire). `Ctrl` `0` returns to 100.
  The shortcut is not a second scale: it calls the same store action as the slider.
- The store (`stores/settings.ts`) holds the percentage, applies the properties on every
  change, and persists through the command. A failed write leaves the interface at the new
  size and says so on the page (an `Alert`), because the size is already applied and lying
  about it would be worse.

## Decision 7 — how it degrades

| missing | what happens |
|---|---|
| the settings file | 100, and the first change writes the file |
| a value not on the ladder | 100, silently: it is a file we didn't write |
| the command fails at startup | the app mounts at 100, and the Appearance page shows its error |
| the write fails | the size stays applied, the page says it couldn't be saved |
| no backend (`pnpm ui:dev`) | the fixture answers 100 and remembers changes for the page's life |

## i18n

`routes.appearance`, `appearance.*`: the intro, the slider's label, the preview card's title,
the shortcut hint (`Ctrl` `+`, `Ctrl` `-`, `Ctrl` `0` as `Kbd`s), the step's percentage
(a number with `%`, formatted by `Intl.NumberFormat`), the write error. `KeyName` gains
`Plus`, `Minus` and `Zero`.

## Testing

**Rust** (test-first, values from this spec):

- `crates/ipc/tests/settings.rs`: the eleven steps in order; `snap_percent` keeps a step,
  answers 100 for 0, 137, 201 and `u16::MAX`; `Settings::default().scale == 100`; the JSON
  shape (`{"activeProfileId":null,"scale":100}`) and a file with no `scale` field reading as
  100; a file with `"scale": 137` reading as 100 **through the same function the command
  uses**, so the guarantee isn't only in the test.

**TypeScript** (Vitest):

- `lib/scale/steps.test.ts`: the ladder is the same eleven numbers as Rust's (read from the
  Rust source with `?raw`, the way `collectionLayout.test.ts` reads a token from the CSS —
  two ladders that drift are a slider that saves a value the backend refuses);
  `scaleFactor`, `snapPercent`, `nextPercent`/`previousPercent` at the ends,
  `spriteMultiple` for all eleven steps (1,1,2,2,2,2,2,3,3,4,4 by the rule of Decision 3).
- `lib/scale/apply.test.ts`: `scaleProperties(percent)` gives `{'--app-scale': '1.25',
  '--sprite-multiple': '3'}`.
- `assets/base.test.ts` rewritten: `html` has a `font-size` and it is `calc(16px *
  var(--app-scale))`; `body` keeps the document text size; and **at scale 1 a spacing step is
  4px** stays the invariant, now read as "the root is 16px when the factor is 1".
- A token test (`assets/scale.test.ts`): every `--spacing-*`, `--text-*` declaration in
  `theme/` is in rem, except a list of exceptions each of which must carry a comment — the
  same shape as the scanner's exemptions, and the list is Decision 4's.
- The shortcut: `shortcutAction(event)` is pure (`Ctrl` `+` → `in`, `Ctrl` `-` → `out`,
  `Ctrl` `0` → `reset`, anything else → `null`), tested without a DOM.

**By eye**, through Playwright on `pnpm ui:dev`: the slider at 50, 100 and 200 with the
matrix, a table and the wiki page open; the preview card staying in view while the page
scrolls; a sprite measured at each end (32px at 50, 128px at 200); `Ctrl` `+` from another
screen; the value surviving a reload.

## Files

**Rust:** `crates/ipc/src/settings.rs` (the ladder, `snap_percent`, the field),
`crates/ipc/tests/settings.rs`, `crates/app/src/lib.rs` (`settings`, `set_scale`),
`crates/app/src/settings_file.rs` (unchanged, it already round-trips the struct).

**TypeScript:** `lib/scale/{steps,apply,shortcut}.ts` and their tests, `lib/ipc/settings.ts`,
`lib/ipc/types.ts`, `lib/constants/commands.ts`, `lib/constants/keyNames.ts`,
`stores/settings.ts`, `composables/useShortcut.ts`, `components/ui/slider/*`,
`kit/sections/SliderSection.vue`, `screens/AppearanceScreen.vue`,
`screens/appearance/{ScalePreview,ScaleSlider}.vue`, `router/{routeTable,routes}.ts`,
`components/shell/sectionNav.ts`, `main.ts`, `App.vue` (the shortcut),
`assets/base.css`, `assets/theme/{spacing,typography,radius,containers}.css`,
`lib/ipc/fixtures/{index,settings}.ts`, `i18n/messages/{it,en}.ts`,
`ui/scripts/scan-conventions.mjs` (the px rule).

## Out of scope for this sub-project

- The density control (compact / default / spacious): the tokens exist (`row-compact`,
  `row`, `row-wide`), the control doesn't, and Discord keeps it apart from the zoom.
- The type scale as a user setting: Discord exposes it, we don't — ours is the design's.
- The rest of Settings (3.6) and what a torn-off window does with the scale (B15).
- Persisting anything else in `settings.json`: the sidebar's width and the tables' sizes are
  3.7's session document (B27).
