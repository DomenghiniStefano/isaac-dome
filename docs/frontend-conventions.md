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
  components.json  shadcn-vue registry settings, written by hand (never `init`)
  src/
    components/
      ui/          shadcn-vue primitives (reka-vega), dressed: they live in the repo
      shell/       title bar, tabs, window controls, navbar, section sidebar
      marks/       the completion-matrix cell, its bit reading, and the grid of cells
      graph/       a node's state badge and its why, the achievement drawing, kind labels
      plan/        QueueError: a write to the plan queue that was refused, and why
      sprite/      PixelSprite: a game sprite that falls back, never a broken image
      kpi/         the KPI tile
      wiki/        the wiki's inline tokens and blocks
      data-state/  read-but-empty, unreadable, empty category
      <domain>/    further app components, named for WHAT THEY ARE
    composables/   useOnActiveProfile: a screen reloads when the active profile changes;
                   useIpcErrorText: one sentence per IpcError, for every screen
    stores/        Pinia, setup syntax: tabs (and tabModel, its pure rules), profile,
                   completion, graph, queue
    router/        routeTable (names, paths, titles, icons: no components), routes, index
    screens/       one screen per route, and the parts only it uses (`screens/profile/`,
                   `screens/completion/`, `screens/nextSteps/`, `screens/unlock/`,
                   `screens/plan/`, `screens/collection/`)
    kit/           development-only Kit page: every primitive in every state (`#kit`)
    verify/        development-only verification page: every command, raw (`#verify`)
    lib/
      ipc/         typed wrappers around Tauri commands — the only place with invoke()
        transport.ts  call(): invoke() in Tauri, the fixtures under `pnpm ui:dev`
        errors.ts     isIpcError, shared by the stores
        fixtures/     development answers, one scenario per `?fixture=`; `art.ts` and
                      `graphArt.ts` glob the pack's sprites, which since 2026-09-15 are not
                      there, so every image answers null; `graph.ts`
                      answers the pack's real unlock payloads, `?catalog=none` without the
                      game, and `graphArt.ts` held their 1,500 images, loaded only then and
                      indexed once; `queue.ts` keeps a plan queue in memory, repaired by a port
                      of the Rust rule (`queueRepair.ts`), `?queue=empty|unavailable|unreadable`;
                      `collection.ts` answers the pack's `collection.json` when it has one, and
                      until then real names and locks with **declared synthetic** quality, pools
                      and flags (`collectionSource()`), `?collection=unread`
      window/      appWindow: the only module that talks to the window; windowFloor: the
                   smallest window the layout is designed against, 640x480
      profile/     what the profile screen and the indicator show, as pure functions
      completion/  what the Completion screen counts, as pure functions
      graph/       a node's state and why, Unlock's facets, search and sort, as pure functions
      plan/        the queue's drops and anchors, the row a move stopped under, as pure functions
      collection/  a collectible's state and the Collection's facets, as pure functions
      constants/   magic strings: command names, dev routes, key names, placement
      design/      themeKeys: the token names cn() reads from the theme CSS;
                   thresholds: the three widths a screen changes shape at
      cn.ts        class merging that knows our tokens
    i18n/          it (the schema), en, locale, useMessages()
    assets/
      main.css     imports only
      theme/       one file per token family
      base.css     document defaults, focus ring, scrollbar, reduced motion
      utilities.css
      fonts/
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
  --color-state-done: …;
  --color-state-unknown: …;
  --spacing-row: …;
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

The split happened with the design system (2026-09-10): `theme/colors.css`,
`typography.css`, `spacing.css`, `radius.css`, `shadow.css`, `opacity.css`, `motion.css`.
The values and the reasoning behind each are in
`docs/superpowers/specs/2026-09-10-design-system-foundations-design.md`.

Operational rules:

- **Arbitrary pixels are forbidden.** `w-[48px]`, `h-[22px]`, `rounded-[5px]` are
  violations. If a new measurement is needed: the token in `@theme` first, then the class.
- **Sizes are in rem, and the root font size is the interface's scale.** Since cycle 3.5c
  the user picks the size of the whole app (`docs/BACKLOG.md` B26): `html` is
  `calc(16px * var(--app-scale))`, so every rem token follows. A token left in px stays its
  own size while the rest moves — which fails nothing and reads as a bug in one component.
  Some values keep px on purpose: a hairline, the two radii, the scrollbar, and the sprite
  tokens, which are whole multiples of a pixel size because the game's art is pixel art. The
  rule is that **each one says why on the line above**, and the scanner checks it.
- **Semantic opacity.** Only named tokens (`opacity-disabled`, `opacity-muted`).
  `opacity-50`, `opacity-30` are violations. `opacity-0` and `opacity-100` remain allowed
  as the endpoints of an animation.
- **Semantic durations.** Only `duration-tap`, `duration-panel`, `duration-sheet`,
  `duration-loop` and similar; never `duration-150`. Watch the token name: the
  `duration-*` utility reads the `--transition-duration-*` family, so it's declared as
  `--transition-duration-fast`, **not** `--duration-fast`. With the wrong name the class
  simply doesn't exist and Tailwind emits nothing, without an error — verified on
  tailwindcss 4.3.3. (`--opacity-*`, on the other hand, is the right family for
  `opacity-*`.)
- **Spacing:** padding and gaps use Tailwind's standard 4px grid (`p-1`, `gap-2`, half
  steps such as `p-2.5` allowed). Named spacing tokens exist only for dimensions that mean
  something — row heights, control height, scrollbar, sprite sizes — in
  `theme/spacing.css`. The grid is only 4px while `rem` is the browser's 16px: **no font
  size on `html`**. The document's text size sits on `body`; on the root it made every step
  3.5px for a whole cycle, and `src/assets/base.test.ts` now fails if it comes back.
- **Letter spacing:** `tracking-*` is reset and holds two tokens, `tracking-nav` and
  `tracking-caps`; `cn()` knows them through `ThemeNamespace.Tracking`.
- **Colors:** never a literal color in a component. The data states — *done*, *unlockable
  now*, *blocked*, *unknown*, *unexpected* — and the *challenge* tag are semantic tokens
  (`state-*`, `challenge`), not shades picked case by case.

**One theme, the dark one.** Values live directly in `@theme`; there is no `.dark` class and
no `@custom-variant dark`, and a `dark:` class is a scanner violation — without the custom
variant, Tailwind's built-in `dark:` follows `prefers-color-scheme`, so a leftover would
switch on with the Windows setting. A light theme later means moving the values of
`theme/colors.css` to selectors and mapping them with `@theme inline`; no component class
changes.

**The default scales are off.** Every namespace we define is reset first
(`--color-*: initial`, `--text-*`, `--font-*`, `--font-weight-*`, `--radius-*`, `--shadow-*`,
`--ease-*`, `--animate-*`), so `bg-red-500`, `text-sm`, `rounded-md`, `font-bold`,
`ease-in-out` and `animate-pulse` generate nothing. The static utilities survive
(`bg-transparent`, `text-current`, `rounded-full`). Spacing keeps the default grid. The trap
this creates is the old one in a new place: a class that doesn't exist emits nothing, so
check the Kit page, not only the typecheck.

**Motion runs on `steps()`.** `duration-tap|panel|sheet|loop` with `ease-tap|panel|sheet|frame`,
and `animate-*` tokens for entrances; the default transition is `0ms` on `steps(1)`, so hover
and active never lag. No exit animations. `prefers-reduced-motion` collapses everything to
0ms in `base.css`.

> The **values** of the tokens are set by the design system, not this document. Here we
> only establish that they exist and that nobody writes a visual value anywhere else.

---

## Responsive layout

The reasoning, and the two approaches that were rejected, are in
[`docs/superpowers/specs/2026-09-20-responsive-layout-design.md`](superpowers/specs/2026-09-20-responsive-layout-design.md).
What follows is the contract.

### A screen is one of three shapes

`<main>` is the **page box**: it scrolls nothing, carries the horizontal padding only, and is the
size container named `page`. Under it every screen declares itself:

| shape | root classes | who |
|---|---|---|
| **flowing** | `flex h-full flex-col gap-4 overflow-y-auto pt-5 pb-15` | content flows and the screen scrolls: Profile, Appearance, Background, Tabs settings, Floor, Live, Roll, the wiki landing and page |
| **filling** | `flex h-full min-h-0 flex-col gap-4 overflow-hidden pt-5 pb-5`, with **exactly one** descendant carrying `min-h-0 flex-1` | a screen with a list: Unlock, Collection, Challenges, Runs, Search, the wiki's category lists |
| **banded** | `-mx-5.5 flex h-full min-h-0 flex-col overflow-hidden`, a `hero-wash` header, then a body carrying `min-h-0 flex-1` and its own `px-5.5` | a screen that opens on a band: Completion, Goals |

**The third row is a correction, not an addition.** Completion stopped being *flowing* with card
#58 and this table was not updated: it took the gutter back with `-mx-5.5` so its band could be
the page's full width, and its matrix has scrolled inside itself ever since. A contract that
describes two of three shapes is read as forbidding the third, so it is written down now that
Goals is the second screen of the kind. The band takes the gutter from the screen rather than
taking it itself — `WikiLanding.vue` records what doing that the other way round cost.

**`min-h-0` on every link of a filling chain is not decoration.** A flex item's default
`min-height:auto` refuses to shrink below its content, so one missing `min-h-0` between the page
box and the scroll body makes `flex-1` grow instead of fit: the list pushes the screen, and it
reads as a bug in the virtualizer. Unlock's chain is
root → `Card class="min-h-0 flex-1"` → the table's root → `VirtualRows`.

**No `max-w-*` on a screen root.** The cap was removed by decision and prose is not excepted: a
long line on the wiki page is an accepted cost, not an oversight.

### A fold is measured against a box, never against the window

Every threshold is a **container query**. Media query variants (`sm:` `md:` `lg:` `xl:` `2xl:`)
are forbidden in `ui/`, for two reasons either of which decides it alone: the section sidebar is
252px of width the window cannot account for, and a media query's `rem` resolves against the
initial 16px rather than the root, so it cannot see the interface's scale — at 150% a media-query
layout never folds, which is the one situation that needed it most.

Two named containers, and **every variant names the one it means**:

| container | is | measures |
|---|---|---|
| `page` | `<main>` | everything a screen draws |
| `shell` | the row holding the sidebar and `<main>` | the sidebar's own collapse |

The sidebar's threshold hangs on `shell` and not on `page` **on purpose**: on `page` a collapse
would widen the content, re-cross the threshold and oscillate.

Three shared sizes in `assets/theme/containers.css` — `compact` (800px), `regular` (960px),
`wide` (1280px) — written `@max-compact/page:hidden`, `@wide/page:flex-row`. They are px and
measured at scale 100, because a container query is compared against a used width.

A component whose break is a fact about **itself** rather than about the page declares its own
token beside itself, with the scale it was measured at in the comment. Two do:
`--container-tab-narrow` and `--container-sidebar-room`.

### A table drops columns by priority

Per table, two edits that are one change:

1. a **pair of `@utility` declarations adjacent in `utilities.css`** — `grid-cols-unlock` and
   `grid-cols-unlock-narrow` — with a comment naming which columns fall and why those;
2. `@max-compact/page:hidden` on the cells that fall, in the header **and** in the row.

A template that drops a track while its cell stays slides every cell after it into the wrong
column: it reads as a styling bug and is a counting one. Collapsing the track to `0px` instead —
one edit, nothing to keep in step — is **rejected**: a zero-width cell stays in the accessibility
tree, and a screen reader would read the columns the eye was told it could do without.

### The window has a floor

640 × 480 logical pixels, declared in **two** places because windows are born in two:
`crates/app/tauri.conf.json` for the main window and the one the tray rebuilds, and
`ui/src/lib/window/windowFloor.ts` for a window torn off a tab, which never goes through that
config. A test holds the two to the same number. At the floor the page box has about 428px.

### What checks it

`scan-conventions.mjs`, four rules: no media-query variant and no container size that is not ours;
a screen root is one of the two shapes; no width cap on a screen root; a narrow grid template with
no hidden cell. A screen, for the middle two, is **what the router mounts** — read from
`routes.ts`, because `src/screens/` also holds the parts only one screen uses.

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

**Determination has no `→ ← ↑ ↓ ⏎ ⌘ ✓`** (measured on the font file). Written in source they
fall back to whatever system font the machine has, so they are Lucide icons
(`ArrowRightIcon`, `ArrowUpIcon`, `ArrowDownIcon`, `CornerDownLeftIcon`, `CheckIcon`), inside a
`Kbd` for keys; the scanner rejects the glyphs. The font has one weight: emphasis is colour
(`text-foreground` against `text-foreground-soft`), never `font-bold`.

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

Every wrapper goes through `call()` in `src/lib/ipc/transport.ts`. Inside Tauri it is
`invoke()`; in a plain browser under `pnpm ui:dev` it answers from `src/lib/ipc/fixtures/`,
so the shell can be looked at without the backend. `?fixture=none|pick|active` picks the
scenario (no saves, a choice to make, an active profile); a command with no fixture throws
`no fixture answers <command>` rather than returning something plausible. The fixtures are
imported dynamically behind `import.meta.env.DEV`: the production build carries none of
them. Their data comes from the committed payloads under `ui/fixtures/` — the real 720 items
and 642 achievements — globbed once per family, from outside `src/` so that no file of data
joins the TypeScript project. `ui/fixtures/README.md` says what each file is.
**No drawing is among them.** The 6065 sprites the design pack carried left the working tree
on 2026-09-15 and the history on 2026-09-20, with the pack and the exporter that wrote it: a
commercial game's art, on a public repository, against the third promise in `CLAUDE.md`.
Every image URL the fixtures answer is `null`, and that was already the documented fallback —
what every user sees before the game's sprites are there. It is now the only case, so the
modules that served art (`fixtures/art.ts`, `fixtures/graphArt.ts`, `kit/markArt.ts`) and the
`?art=none` switch went with them: an option with one reachable value reads like a choice and
is not one. `scripts/check-no-game-assets.mjs` fails the run if a picture is ever tracked
again. The window works the same way: `src/lib/window/appWindow.ts` is the only module that
imports `@tauri-apps/api/window`, and outside Tauri its controls do nothing.

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

**The extensions that exist** (cycle 2): `ButtonSize.Micro` (a tab's close), `IconCompact`
(the tab strip's "+"), `Row` (sidebar items), `Inline` (a wiki reference in running text —
`inline`, not `inline-flex`, or an icon sets the baseline), `Window` (window controls),
`Compact` (the search trigger), `Section` (a navbar section, full height);
`Progress` takes `size` (`Default`, `Micro` for the KPI bar) and `tone` (`Primary`, `Done`,
`Muted`);
`ButtonVariant.Nav`, `Section`, `Ref`, `Chrome`, `ChromeDanger`, `Field`. A size only sizes
and a variant only colours: cva writes size classes after variant classes, so a height inside
a variant loses. The collapsible card is a set of Card parts (`CardCollapsible`,
`CardCollapsibleTrigger`, `CardCollapsibleContent`), because a prop can't turn a `div` into
Reka's collapsible root.

**Added with sub-project 3.3a**: `BadgeVariant.Partial` — a node the graph couldn't fully
interpret: the blocked colours, a dashed edge and the lock, never the star of `Now` and never
the hatch of `Unknown`. Long lists are virtualized with `@tanstack/vue-virtual`: the options
are a `computed`, the row height is a number pinned against its spacing token by a test
(`ui/src/lib/scale/rows.test.ts`, which reads `--spacing-row-wide` out of the CSS), and each
row's offset travels as a CSS variable read by `translate-y-(--row-start)`.
The test used to be `screens/unlock/unlockLayout.test.ts` and this line still named it after it
had gone: the height is **shared by three tables** now — Unlock, Collection and Runs — so it
belongs to `lib/scale/`, not to the screen that needed it first.

**Added with sub-project 3.3b**: `BadgeVariant.Wanted` — a queue row you asked for, a square tag
in the primary colours, told apart from a step a wish dragged in. A drop sends **the row it lands
under**, never an index (`lib/plan/queueDrop.ts`); what the screen draws next is the order the
command answers with. The
IpcError sentences live in `useIpcErrorText` and `ipcErrors.*`, shared by every screen that has
to say why a command failed.

**One composable drags every list.** `composables/useDragList.ts` owns the choreography — a press
becomes a drag past a threshold, the pointer is captured only then, the items' rects are read
once, nothing moves until the release — plus the lifted copy and `Escape`, which calls a drag off
and commits nothing. The decisions it makes are pure and tested in `lib/drag/dragList.ts`. A
screen brings two things and nothing else: where its items are, and what a drop there means —
`components/shell/tabs.ts` for the strip, `lib/plan/queueDrop.ts` for the queue. A third
hand-written pointer drag inside a screen is a bug, not a variant. The lifted copy is
`components/ui/drag/DragGhost.vue`, teleported to the body and `aria-hidden`: it is a picture of
the row, not a second one. The skin is flat by decision (`assets/theme/shadow.css` sets
`--shadow-*: initial`), so it lifts with a `primary` border on an opaque sheet and never with a
shadow token. The sidebar's resize stays outside it: same choreography, no list and no drop.

### How a primitive is written

- **From the registry, already dressed.** shadcn-vue 2.8.2, style `reka-vega`. Never
  `shadcn-vue init` (it rewrites `main.css`); `pnpm dlx shadcn-vue@2.8.2 add <name> --view`
  shows the registry version, `--diff` compares it with ours.
- **Variants in `variants.ts`**, as `as const` objects keying the `cva` config; `index.ts`
  re-exports them by name. Not in `index.ts`: a component that uses a constant as a prop
  default would read it before `index.ts` has initialised it.
- **State styling with `data-[state=…]`**: Reka sets `data-state="open"`, `"checked"`,
  `"active"`, `"on"` and `data-highlighted`; the registry's `data-open:` classes need a
  stylesheet we don't import.
- **No `tw-animate-css`, no `opacity-50`, no `outline-none` on focusable elements** (the ring
  comes from `base.css`), no `shadow-*`, no radius except `rounded-input`, `rounded-cell` and
  `rounded-full`.
- **`cn()` from `@/lib/cn`**: it knows our token names. Plain `twMerge` would read `text-body`
  as a colour and drop it next to `text-foreground`.
- Shared state between parts goes through a typed `InjectionKey`, never a string key.
- **Combined states are part of the dressing.** An "on" state class is gated with
  `enabled:` (`enabled:data-[state=checked]:bg-primary`), otherwise a disabled control that
  is on keeps painting as on; the Kit page shows each primitive disabled *and* on, and
  overlays (select list, popover, tooltip, dialog) are checked open, not only closed.
- **`TooltipProvider` is required once** above any tooltip: Reka throws without it. The Kit
  page wraps itself in one; the shell must too.

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
- **Components call `useMessages()`** from `@/i18n`, never `useI18n()` directly: vue-i18n's
  own `t()` accepts any string, while `useMessages().t` only takes a key that exists in the
  Italian schema (`i18n/messages/it.ts`). `en.ts` is typed against that schema, so a missing
  English key is a compile error. `useMessages()` uses the global scope: no component needs a
  local instance.

---

## TypeScript and Vue

- **TypeScript strict**, always.
- **`pnpm typecheck` is `vue-tsc --build --force`.** `ui/tsconfig.json` is a solution file;
  `vue-tsc --noEmit` on it checks no file at all, and did so until 2026-09-10.
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

**Vitest** (`pnpm ui:test`, part of `scripts/check`) runs the frontend's logic: pure functions
beside their module (`*.test.ts`). Type-level guarantees are probes checked by
`pnpm typecheck` (`i18n/messageKey.typecheck.ts`). Presentation is checked on the Kit page
(`pnpm ui:dev`, then `#kit`). Vitest doesn't load CSS unless `test.css.include` matches it:
the theme files are listed there because `cn()` reads them.

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
| **Arbitrary pixel value in a class** | `ui/scripts/scan-conventions.mjs` |
| **Hardcoded opacity** | `ui/scripts/scan-conventions.mjs` |
| **Hardcoded duration** | `ui/scripts/scan-conventions.mjs` |
| **`invoke()` outside the IPC layer** | `ui/scripts/scan-conventions.mjs` |
| **Window API outside `src/lib/window/`** | `ui/scripts/scan-conventions.mjs` |
| **Numeric `:size` prop on an icon** | `ui/scripts/scan-conventions.mjs` |
| **Raw `<button>` / `<input>` outside `src/components/ui/`** | `ui/scripts/scan-conventions.mjs` |
| **String literal unions (`'a' \| 'b'`)** | `ui/scripts/scan-conventions.mjs` |
| **Visible strings in the template** (skipping `src/kit/` and `src/verify/`, development-only) | `ui/scripts/scan-conventions.mjs` |
| **`dark:` variant** (one theme) | `ui/scripts/scan-conventions.mjs` |
| **Literal colour in a class** | `ui/scripts/scan-conventions.mjs` |
| **Colour alpha modifier (`bg-x/50`)** | `ui/scripts/scan-conventions.mjs` |
| **`tw-animate-css` class** (not installed) | `ui/scripts/scan-conventions.mjs` |
| **Literal `variant`/`size`/`density`/`orientation` on a primitive** | `ui/scripts/scan-conventions.mjs` |
| **Glyph missing from Determination** | `ui/scripts/scan-conventions.mjs` |
| **A px token in `assets/` with no reason beside it** | `ui/scripts/scan-conventions.mjs` |
| **A media query variant, or a container size that is not ours** | `ui/scripts/scan-conventions.mjs` |
| **A screen root that is neither flowing nor filling** | `ui/scripts/scan-conventions.mjs` |
| **A width cap on a screen root** | `ui/scripts/scan-conventions.mjs` |
| **A narrow grid template with no column hidden** | `ui/scripts/scan-conventions.mjs` |

Three rows arrived on 2026-09-06 — before that, the document declared five rules and the
script checked three — six more on 2026-09-10 with the design system, and **four on
2026-09-20 with the responsive frame** (3.13a), which also gave the script something it had
not needed before: a rule whose scope is *read from the code* rather than guessed from a
path, since `src/screens/` holds screens and parts alike and only `routes.ts` knows which
is which. On the same day
the visible-string heuristic learned to skip quoted attribute values: a class such as
`has-[>svg]:grid-cols-2` used to end the tag early and leave half a class list behind as
"visible text". **This table and the script's `checks` array must have the
same rows**, and that's the only thing keeping the document from promising a check that
doesn't happen.

**The exceptions live in the script, not in the head of whoever runs it.** At the top of
`scan-conventions.mjs` there's an `EXEMPTIONS` array with file, check, and reason. It stood empty
for months; since 2026-09-20 it holds two, both against the screen-shape rule and both permanent
rather than temporary: `WelcomeScreen.vue`, a takeover drawn outside `<main>` with no page box to
fill, and `WikiScreen.vue`, which has no root of its own and picks one of four bodies that each
carry the shape. An empty list is the goal; an exception with no written reason is an untracked
violation.

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
