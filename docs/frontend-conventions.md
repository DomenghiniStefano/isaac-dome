# Frontend conventions

Rules for the code in `ui/`, written for the target stack: Vue 3 + TypeScript inside
Tauri, Tailwind **v4**, **Lucide** icons, shadcn-vue on **Reka UI**.

They exist for one reason only: in an app where every screen shows the same three states
(done · unlockable · unknown) across ten different grids, consistency isn't achieved by
looking at screenshots. It's achieved by making it uncomfortable to write the special case.

**They're written before `ui/` exists**, on purpose: a convention introduced after the
code is a refactor; introduced before, it's free.

---

## The five rules that get broken first

1. **No `<style>` in SFCs** — everything in Tailwind, except the three exceptions listed below.
2. **No hardcoded visual constants** — every pixel, opacity, and duration is a token in `@theme`.
3. **No `invoke()` in components** — only typed wrappers in `src/lib/ipc/`.
4. **No raw `<button>` / `<input>`** — use the primitives in `src/components/ui/`.
5. **No string unions as discriminators** — `const X = { … } as const`, never
   `type X = 'a' | 'b'`.

The rest of the document explains the reasoning and adds the details.

---

## Structure of `ui/`

```
ui/
  src/
    components/
      ui/          shadcn-vue primitives: they live in the repo and get modified
      <domain>/    app components, named for WHAT THEY ARE
    composables/
    stores/        Pinia, setup syntax
    lib/
      ipc/         typed wrappers around Tauri commands — the only place with invoke()
      constants/   magic strings: command names, storage keys
    i18n/          it, en
    assets/
    App.vue
    main.ts
```

A component is named **for what it is, never for where it's used**: `MarksMatrix`,
not `CompletamentoMatrice`; `ProfileRow`, not `SetupScreenRow`. If a specialized behavior
is needed on top of a generic one, build **a generic base + a thin variant**, don't fold
the feature into the base's name.

---

## `<style>` blocks

**Forbidden in SFCs.** The only allowed exceptions:

| Exception | Why |
|---|---|
| `-webkit-app-region` | Only needed if and when a custom titlebar exists; has no Tailwind equivalent |
| Custom `@keyframes` | Non-trivial animations can't be expressed as utilities |
| Scrollbar overrides | Three lines of CSS read better than `[&::-webkit-scrollbar]:…` |

Outside these three cases, a `<style>` block is a violation, not a shortcut.

**The exception must be declared, not inferred.** An allowed `<style>` block starts with
a comment stating the reason:

```vue
<style>
/* exception allowed: scrollbar override, not expressible as a utility */
::-webkit-scrollbar { … }
</style>
```

Without that comment it's a violation, even if the content would otherwise be allowed. The
reason is practical: automatically recognizing "this block contains *only* exceptions"
would require a CSS parser, and every approximation gets bypassed by accident — just
mention `@keyframes` in a comment and anything gets through. With the marker, the
exception stops being an accident of the text and becomes a deliberate act, visible to
whoever reviews it.

**Why the rule exists.** It's not that "Tailwind is better than CSS": it's that CSS must
live in one place, and a `<style>` block in an SFC is by definition a second place. Three
concrete consequences:

- **The design system doesn't reach it.** When the tokens change — and they will, since
  they come from outside — the utilities update themselves, `<style scoped>` doesn't.
  Nobody notices until someone looks at that screen.
- **It can't be found.** Searching for where a measurement is defined works if
  measurements live in tokens. A local style is invisible to that search.
- **It never dies.** Classes disappear along with the markup that carries them; a rule in
  a `<style>` block outlives the component that used it, and nobody trusts themselves
  enough to delete it.

There's also a reason specific to this app: the same cell appears in Completion, in
Collection, and in Unlock. If the *unknown* state is a token, it's identical across the
three screens by construction; if it's a local style, it's identical by coincidence.

**The exceptions are a readability choice, not a capability limit.** Tailwind v4 could
handle the scrollbar too, with arbitrary variants. The three exceptions exist because in
those cases the utilities make the code worse, not because they can't do it. It's a
distinction that matters: the day a fourth exception seems necessary, the question to ask
is "does it really read better?", not "can it be done?".

### Dynamic values: CSS variables, not inline pixels

Inline `style=` with hand-written pixel values is a violation, and so is `var(--token)`
used directly in the template instead of the class.

**But geometry computed at runtime is a legitimate case**, and shouldn't be confused with
the two things above: a position, a transform, or a height that depend on a measurement
taken from the DOM can't be classes, because they don't exist at build time.

The correct form is neither a `<style>` block nor a hardcoded inline pixel, but a **CSS
variable bound from the template**, consumed by a utility:

```vue
<!-- no: hardcoded visual value, outside the system -->
<div :style="{ transform: `translateX(${panX}px)` }">

<!-- yes: the value is dynamic, how it's used stays inside the system -->
<div :style="{ '--pan-x': `${panX}px` }" class="translate-x-(--pan-x)">
```

The criterion is: **the value can be dynamic, the vocabulary can't.** If the number comes
from a calculation, it goes through a CSS variable; if it comes out of the developer's
head, it's a missing token.

### When to revisit this rule

M2 calls for the **unlock graph**. If it becomes a real visualization — nodes, edges,
SVG — that's the area where this rule could start costing more than it's worth. It won't
change now on a hypothesis, but when we get there it must be reopened explicitly, instead
of being quietly worked around: a rule secretly ignored is worse than a rule that's been
changed.

---

## Tokens and Tailwind v4

**v4 changes the game, for the better.** In Tailwind v3 every visual constant had to be
declared twice — a CSS variable in `main.css` and a token in `tailwind.config.ts`. In v4
the JavaScript config file is gone: tokens are declared **once**, in CSS, inside `@theme`,
and they generate their corresponding utilities on their own.

```css
@import "tailwindcss";

@theme {
  --color-mark-done: …;
  --color-mark-unknown: …;
  --height-row: …;
}
```

### One place for tokens, not one file

`@theme` doesn't have to live entirely in `main.css`. As the tokens grow — and they will,
along with the design system — they get split across multiple files, as long as the split
is **predictable**:

- **one file per family, named after the prefix of the tokens it holds**: `--color-*` in
  `theme/colors.css`, `--spacing-*` in `theme/spacing.css`, typography and text in
  `theme/typography.css`. The file follows from the token's name, and vice versa. It's the
  only thing that makes the split a help instead of a hiding place;
- the files live under `src/assets/theme/` and are imported **at the top** of `main.css`,
  right after `@import 'tailwindcss'`: CSS `@import` rules must precede everything else,
  and Tailwind collects every `@theme` it finds by following the imports;
- **`main.css` stays the single entry point.** That's why `tailwindStylesheet` in
  `.prettierrc.json` keeps pointing there and shouldn't be touched: the plugin follows the
  imports on its own.

What the rule forbids isn't the second file, it's the **second place where the same token
is defined**. Two declarations with the same name don't produce an error: the last one
loaded wins, and the first one stays there lying to whoever reads it.

As long as the tokens fit on half a screen, they stay in `main.css`. The split happens
when it helps find them, not before.

Operational rules:

- **Arbitrary pixels are forbidden.** `w-[48px]`, `h-[22px]`, `rounded-[5px]` are
  violations. If a new measurement is needed: the token in `@theme` first, then the class.
- **Semantic opacity.** Only named tokens (`opacity-disabled`, `opacity-muted`).
  `opacity-50`, `opacity-30` are violations. `opacity-0` and `opacity-100` remain allowed
  as the endpoints of an animation.
- **Semantic durations.** Only `duration-fast`, `duration-slow` and similar; never
  `duration-150`. Watch the token name: the `duration-*` utility reads the
  `--transition-duration-*` family, so it's declared as `--transition-duration-fast`,
  **not** `--duration-fast`. With the wrong name the class simply doesn't exist and
  Tailwind emits nothing, without an error — verified on tailwindcss 4.3.3. (`--opacity-*`,
  on the other hand, is the right family for `opacity-*`.)
- **Spacing:** use Tailwind's standard 4px grid (`p-1`, `gap-2`, …), with no dedicated
  tokens. It's the only family of values for which the default scale is enough.
- **Colors:** never a literal color in a component. The app's three states — *done*,
  *unlockable now*, *locked* — plus *unknown* are semantic tokens, not shades picked case
  by case.

Dark mode in v4 is no longer enabled with `darkMode: 'class'` in a config file: it
requires an explicit variant in the CSS (`@custom-variant dark …`). The dark theme is the
app's true default, not a variant: tokens are defined for dark and adapted for light.

> The **values** of the tokens are set by the design system, not this document. Here we
> only establish that they exist and that nobody writes a visual value anywhere else.

---

## Class ordering

Automated with `prettier-plugin-tailwindcss` **from the first commit**, not left to
discipline. A hand-documented order plus a postponed plugin costs twice: nobody follows
it, and the day the plugin arrives it produces a reordering diff across the whole repo.
Starting with the plugin, the rule enforces itself and that diff never happens.

The resulting order is the plugin's canonical one: position and display, flex/grid,
sizing, overflow, padding, margin, typography, colors, borders, effects, transitions,
interaction, pseudo-states, responsive.

The plugin also orders Vue's dynamic `:class`, not just the static attribute. In Tailwind
v4, since `tailwind.config.js` no longer exists, you need to point to where the CSS
containing `@theme` lives, with the **`tailwindStylesheet`** option:

```json
{
  "plugins": ["prettier-plugin-tailwindcss"],
  "tailwindStylesheet": "./src/assets/main.css"
}
```

Without that option the plugin doesn't know our tokens and treats them as unknown
classes, leaving them at the end.

An error the plugin **doesn't** catch and that needs watching for: **concatenating two
classes without a space** (`h-rowpx-2`). Tailwind silently ignores the resulting class,
without an error.

---

## Icons

Package: **`@lucide/vue`**. Not `lucide-vue-next`, which is the old name and is
**deprecated** as of Lucide v1. Icons are imported one at a time, so only the ones used
end up in the bundle:

```vue
<script setup lang="ts">
import { Check } from '@lucide/vue'
</script>
```

Note: in Lucide v1 **brand icons were removed** from the set. Any logo or brand mark needs
to be drawn separately, not looked for among the icons.

**Sizing is done with the Tailwind class, not with the `size` prop.**

```vue
<!-- no -->
<Check :size="16" />

<!-- yes -->
<Check class="size-4" />
```

Lucide's documentation presents the two approaches as equivalent and doesn't recommend
either. For us, though, they aren't: `:size="16"` is **a visual constant hardcoded inside
a component**, exactly what the token rule two sections above forbids. Going through the
class, the size stays inside the system — it aligns with the text next to it, changes with
the theme, and can be found by searching the way any other measurement is searched.

The same goes for color: `currentColor` is the default, so the icon inherits the text
color. `color` is never passed as a prop; the container's text color is changed instead.

The only prop worth passing is `absolute-stroke-width`, and only if an enlarged icon shows
a disproportionate stroke.

## The IPC layer

**`invoke()` never appears in a component.** Every Tauri command has a typed wrapper in
`src/lib/ipc/`, and components call that.

```
src/lib/ipc/setup.ts       setupState(), selectProfile(id)
src/lib/ipc/save.ts        saveSummary(), completion()
```

Three reasons, in order of importance:

1. Command names are strings: concentrating them in one place means renaming a Rust
   command is a one-file change, not a hunt through the repo.
2. The return type is declared once. Without the wrapper, every caller redeclares its own
   idea of the JSON's shape, and sooner or later two ideas diverge.
3. It's the place to validate what arrives from the boundary. An enum coming off the wire
   is a string: it must be validated there, with a fallback and a warning, not propagated
   downstream.

Command names live in `src/lib/constants/`, not hand-written in the wrapper.

A note on serde, which is the twin trap on the Rust side: every struct that crosses the
IPC has `#[serde(rename_all = "camelCase")]`. Without it, TypeScript reads `undefined` and
nobody notices until it's too late.

---

## UI primitives

shadcn-vue components **live in the repo** — that's the reason shadcn-vue was chosen over
a "complete" library. So they get modified.

- No raw HTML `<button>` or `<input>` in app components.
- No hand-written `<label class="text-xs …">`: a labeled field uses the primitive that
  represents it.
- If a pattern isn't covered, **extend the existing primitive with a prop**, don't
  hand-style it inside a new component. The extension stays reusable; the ad-hoc style
  doesn't.

Legitimate exceptions: non-interactive markup (`<div>`, `<span>`, display-only `<table>`)
and Reka UI's headless primitives where the API requires the native element underneath.

Practical rule: if you're writing `<button class="… hover:bg-…">`, stop and look for the
primitive.

---

## Constants and enums

**A hand-written string union is a magic value wherever it appears** — not only in
`type X = …`, but also inline on a prop, in an `as`, in a generic.

```ts
// no
align?: 'start' | 'center' | 'end'

// yes
const Align = { Start: 'start', Center: 'center', End: 'end' } as const
```

Consumers reference the symbol, not the string. Constants live in the module that owns
them, not in a catch-all `types.ts`.

**This also applies to the types on the IPC wire**, and it's the part that had lagged
behind until 2026-09-06: `CandidateSource`, `SavePrefix`, `MissingReason`, `ItemKindView`,
`OriginView`, `StepsBasis`, and `WikiMissingReason` were hand-written unions in
`src/lib/ipc/types.ts`, with a comment justifying them as "a value, not a discriminator".
But that distinction is the *Rust* rule — it's used to decide whether an enum serializes
tagged or as a bare string — and it says nothing about how TypeScript should declare it.
Now they're `as const` objects like all the others: the same strings stay on the wire, and
the rule has no exceptions to remember.

The **discriminator of a tagged union** stays out of scope: `b.kind === 'paragraph'` isn't
a magic value, it's a comparison that TypeScript narrows on and that `assertNever` forces
you to cover. The tag is defined by the type, not by whoever writes it.

For "field X of an object", use `keyof` on a **named type**, never a hand-pasted list of
keys.

**Exhaustiveness is mandatory** on every `switch` over a closed enum, with `assertNever`
in the final branch. Without it, adding a variant doesn't break the build: it silently
produces an empty output. In our case this applies in particular to `ActiveProfile` and to
`Cell` in the marks matrix — exactly the two types where a forgotten branch would produce
false data instead of an error.

---

## Internationalization

- i18n from day one: **it** and **en**, with keys aligned between the two files.
- No visible string written in the template.
- Item, character, and boss names **stay in English** even in Italian: they're the names
  that appear in the game and that the user searches by. They're not strings to translate,
  they're data.
- `useI18n()` must also be called in the root component, otherwise children emit the
  "Not found parent scope" warning.

---

## TypeScript and Vue

- **TypeScript strict**, always.
- **`<script setup>` always**, never the Options API.
- Pinia in **setup syntax**.
- Always explicit exports, **never `export *`** (already applies to the whole project).
- Naming: `camelCase` for variables and functions, `PascalCase` for types and components,
  `useXxx` for composables, `PascalCase.vue` for component files.
- **lodash-es** preferred for array and object operations, with named imports.
- No `let`/`while` for iterating or accumulating: functional structure, as in the rest of
  the project.

---

## Tests

The project's general rule applies: test-first for code with logic — composables, stores,
pure functions, transformations. **The expected value is decided first by reasoning about
the spec**, then the test is written, then the code. A failing test is a hypothesis of a
bug in the code, not an expectation to fix.

Exempt: pure presentation, configuration, static markup.

This app's heavy logic lives in Rust and already has its own tests. On the frontend side,
most of it is presentation: tests are needed where there's a decision, not where there's a
grid.

---

## What this document doesn't cover

For honesty's sake, and so as not to make this document look more complete than it is:

- **CodeMirror, blob URLs, Reka UI's dismiss gotchas, reconciliation patterns**: rules
  like these get written after paying the price of the problem, and here those problems
  don't exist yet. They get added when we run into them, not in anticipation.
- **Per-workspace persistence**: there's no concept of workspace here. The equivalent is
  the active profile, which lives in the backend.
- **Design and i18n scanning** (tokens declared and never used, mismatched keys between
  `it` and `en`): it makes sense once there's code to scan, so it arrives with the real
  frontend. The conventions scan, on the other hand, already exists — see the following
  section, which explains why these scripts are less optional than they look.

---

## How these rules are checked

| Rule | Who enforces it |
|---|---|
| Class ordering | `prettier-plugin-tailwindcss`, automatic |
| TypeScript strict, types | `vue-tsc --noEmit` |
| Generic Vue and TS rules | ESLint flat config: `eslint-plugin-vue`, `typescript-eslint`, `@vue/eslint-config-typescript` |
| **`<style>` blocks without a marker** | `ui/scripts/scan-conventions.mjs` |
| **Mandatory `<script setup>`** | `ui/scripts/scan-conventions.mjs` |
| **Arbitrary pixels, hardcoded opacity and durations** | `ui/scripts/scan-conventions.mjs` |
| **`invoke()` outside the IPC layer** | `ui/scripts/scan-conventions.mjs` |
| **Numeric `:size` prop on an icon** | `ui/scripts/scan-conventions.mjs` |
| **Raw `<button>` / `<input>` outside `src/components/ui/`** | `ui/scripts/scan-conventions.mjs` |
| **String literal unions (`'a' \| 'b'`)** | `ui/scripts/scan-conventions.mjs` |
| **Visible strings in the template** (skipping `src/kit/`, development-only) | `ui/scripts/scan-conventions.mjs` |
| **`dark:` variant** (one theme) | `ui/scripts/scan-conventions.mjs` |
| **Literal colour in a class** | `ui/scripts/scan-conventions.mjs` |
| **Colour alpha modifier (`bg-x/50`)** | `ui/scripts/scan-conventions.mjs` |
| **`tw-animate-css` class** (not installed) | `ui/scripts/scan-conventions.mjs` |
| **Literal `variant`/`size`/`density`/`orientation` on a primitive** | `ui/scripts/scan-conventions.mjs` |
| **Glyph missing from Determination** | `ui/scripts/scan-conventions.mjs` |

Three rows arrived on 2026-09-06 — before that, the document declared five rules and the
script checked three — and six more on 2026-09-10, with the design system. On the same day
the visible-string heuristic learned to skip quoted attribute values: a class such as
`has-[>svg]:grid-cols-2` used to end the tag early and leave half a class list behind as
"visible text". **This table and the script's `checks` array must have the
same rows**, and that's the only thing keeping the document from promising a check that
doesn't happen.

**The exceptions live in the script, not in the head of whoever runs it.** At the top of
`scan-conventions.mjs` there's an `EXEMPTIONS` array with file, check, and reason: today it
only contains `App.vue` and `WikiInline.vue`, which are declared verification pages and
will be replaced by the real frontend. An empty list is the goal; an exception with no
written reason is an untracked violation.

Two known limits, stated so as not to pretend the script is a compiler: the
visible-strings check is a text heuristic (it strips tags, comments, and `{{ … }}`, then
looks for two letters in a row), and the literal-union check looks at the text, not the
AST — so it also catches a union inside a comment. Both err on the side of the false
positive, which gets noticed; not the false negative, which doesn't.

The bold rows are the point: **ESLint has no official rule to forbid `<style>` blocks in
SFCs or to enforce `<script setup>`** — verified against `eslint-plugin-vue`'s
documentation, this isn't a gap in our configuration.

It follows that this document's most important rules are the ones no linter enforces, and
that without a committed script they'd remain good intentions. That script exists:
`ui/scripts/scan-conventions.mjs`, runnable with `pnpm scan`, exits with a non-zero code
when it finds a violation. **Every new rule added to this document must also be added
there**, otherwise the document promises a check that doesn't happen.
