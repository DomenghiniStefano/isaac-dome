# Design system, cycle 1 — foundations and primitives (design)

**Date:** 2026-09-10
**Milestone:** design system (`docs/STATUS.md`, "Design system"), cycle 1 of 3
**Depends on:** the Claude Design export of 2026-09-10 (`IsaacDome design system.zip`), `DESIGN-BRIEF.md`, `docs/frontend-conventions.md`
**Status:** design agreed in conversation (scope, token naming, theme, colour rules, primitive list); pending spec review

## What this is

The first of three cycles that turn the design export into code in `ui/`:

1. **Foundations and primitives** — this document. Tokens in `@theme`, the font, motion,
   focus, scrollbar, i18n, `cn()`, and the shadcn-vue primitives the eight screens need,
   each dressed in the IsaacDome skin and shown on a development-only Kit page.
2. **App components** — what `Chrome e Stati.dc.html` draws and shadcn doesn't have: the
   tab strip, the navbar, the section sidebar, the KPI tile, the completion-matrix cell,
   the wiki inline tokens, the data states. Its own spec.
3. **Screens** — the shell and the screens of `DESIGN-BRIEF.md` §4, with Pinia, Vue Router
   and TanStack.

Nothing in this cycle shows a real screen. What it delivers is the vocabulary those screens
will be written in, built so that writing outside it is uncomfortable or impossible.

## Applicable constraints

1. **No game assets in the package.** The export's sprites (marks, portraits, paper) are
   working material and are never copied into `ui/`. The only file taken from the export is
   the Determination font, which is not a game asset (see Decision 5).
2. **Offline.** No CDN at runtime: the font is served by the app, and IBM Plex Mono, which
   the export loads from Google Fonts, leaves the app.
3. **The five frontend rules** of `docs/frontend-conventions.md` hold inside
   `src/components/ui/` too, with the one exception the conventions already grant it (raw
   `<button>`/`<input>`).
4. **i18n from day one.** A primitive that renders a string (a close button's accessible
   label) goes through vue-i18n.

## Source material, and how it is read

The export holds six pages. Their values are **transcribed into this document**; the zip
itself is not committed, because its `assets/` folder is full of the game's sprites.

| page | what it is trusted for |
|---|---|
| `Shadcn Kit.dc.html` (dark) | **the source of component values**: colours, sizes, states as drawn |
| `Tokens.dc.html` | the palette's intent and the names of roles; its `@theme` draft is not usable as is (Decision 3) |
| `Motion.dc.html` | durations and the `steps()` rule |
| `Chrome e Stati.dc.html` | focus ring, scrollbar, row densities; the rest is cycle 2 |
| `Shadcn Kit Light.dc.html` | out of scope (Decision 1) |
| `Schermate.dc.html` | which primitives the screens actually use |

Where two pages disagree, **the dark kit wins on component values** (it is what the
components were drawn with) and **`DESIGN-BRIEF.md` wins on product** (a kit example that
contradicts the brief is dropped, not implemented). Every disagreement resolved here is
listed in "Handed back to design".

## Decision 1 — one theme, dark

The app has **one theme**, the dark one ("il seminterrato"). The light variant is not
designed consistently — `Tokens.dc.html` proposes paper `#E3DBDA` on wall `#A59B9A`,
`Shadcn Kit Light.dc.html` leather `#D9C9B8` on `#9C8677` — and no screen needs it.

Consequences:

- **Values live directly in `@theme`**, not in `:root` + `.dark` + `@theme inline` as
  shadcn-vue's installer writes them. With one theme the indirection has nothing to switch.
  A second theme later is a change inside `theme/colors.css` alone — move the values to
  selectors, map them with `@theme inline` — and no component class changes.
- **`@custom-variant dark` leaves `main.css`, and `dark:` becomes a scanner violation.**
  This isn't tidiness: without the custom variant, Tailwind v4's built-in `dark:` follows
  `prefers-color-scheme`, so a `dark:` class left over from a shadcn component would switch
  on by itself on a Windows machine set to dark mode.

## Decision 2 — shadcn's role names, plus the roles shadcn doesn't have

Colour tokens keep **shadcn-vue's role names** (`background`, `card`, `popover`, `primary`,
`secondary`, `muted`, `accent`, `destructive`, `border`, `input`, `ring`) with IsaacDome
values, and add **domain roles** only where shadcn has no concept: the leather header band,
the paper sheet, the data area, alternate rows, the hairline separator, the text ramp, the
five data states.

A role has one name. Two roles may share a value today (`card` and `data` are both
`#1B120E`) — that is two names for two jobs, which can diverge, not two names for one job.

## Decision 3 — Tailwind's default scales are switched off

Every namespace we define is **reset before it is defined**:

```css
@theme {
  --color-*: initial;
  --text-*: initial;
  --font-*: initial;
  --font-weight-*: initial;
  --radius-*: initial;
  --shadow-*: initial;
}
```

With the defaults gone, `bg-red-500`, `text-sm`, `rounded-md`, `shadow-xs` and `font-bold`
**generate nothing**. The rule "never a literal colour, never an off-system size" stops
being discipline and becomes the absence of the class. `--spacing` stays on (the
conventions keep the default 4px grid), and so do the static utilities (`bg-transparent`,
`border-transparent`, `text-current`, `rounded-full`), which are not theme values — to be
confirmed on tailwindcss 4.3 during planning.

The export's own `@theme` draft can't be pasted for the same reason: `--state-done`,
`--surface-sheet`, `--selected`, `--radius`, `--row-h` and `--m-tap` sit outside any
namespace and produce no utility, silently. Every token below is renamed into its namespace.

## Decision 4 — the token catalogue

Files under `ui/src/assets/theme/`, one per family, imported at the top of `main.css`
(`docs/frontend-conventions.md`, "One place for tokens, not one file").

### Colours — `theme/colors.css`

**shadcn roles**

| token | value | role |
|---|---|---|
| `background` | `#150E0D` | window ground |
| `foreground` | `#F2ECEC` | primary text, emphasis |
| `card` / `card-foreground` | `#1B120E` / `#F2ECEC` | card body |
| `popover` / `popover-foreground` | `#1B120E` / `#F2ECEC` | popover, select, command content |
| `primary` / `primary-foreground` | `#901800` / `#F2ECEC` | default button; selected tab, toggle, checked control |
| `secondary` / `secondary-foreground` | `#3A251D` / `#F2ECEC` | secondary button, raised surface |
| `muted` / `muted-foreground` | `#241812` / `#BFB2AC` | disabled surface / helper text and icons |
| `accent` / `accent-foreground` | `#3A251D` / `#F2ECEC` | hover of ghost buttons and list items |
| `destructive` / `destructive-foreground` | `#FC0000` / `#E8C4BF` | invalid field, error alert / text on the error surface |
| `border` | `#5A3A2C` | border of sheet, card, table |
| `input` | `#8A5A46` | border of fields and controls |
| `ring` | `#00A6CC` | focus ring — and, from cycle 2, active filters; nothing else |

**Domain roles**

| token | value | role |
|---|---|---|
| `sheet` | `#2A1C17` | "il foglio": the panel a section sits on; dialog body |
| `band` / `band-foreground` | `#6C4437` / `#F2ECEC` | leather header strip of card, table, dialog, popover |
| `data` | `#1B120E` | "area dati": table body, field background |
| `row-alt` | `#20140F` | alternate row |
| `row-hover` | `#241812` | hovered row |
| `hairline` | `#2E1E17` | separator between rows |
| `primary-hover` / `primary-active` / `primary-edge` | `#A81C00` / `#6C0C00` / `#4A0A00` | default button states and border |
| `secondary-hover` / `secondary-edge` | `#492E20` / `#6C4437` | secondary button hover; border of secondary button, toggle group, kbd |
| `selection-edge` | `#FC0000` | border of a checked checkbox or switch |
| `destructive-surface` | `#2A0B08` | error alert background |
| `tooltip` | `#492E20` | tooltip background |
| `overlay` | `rgb(12 7 6 / 0.8)` | dialog backdrop — the alpha lives in the token, never in a class |
| `highlight` | `#EBD9C4` | cream: group titles, links |
| `foreground-soft` | `#D5C8C2` | body copy inside panels |
| `subtle-foreground` | `#A69690` | group labels, table footer |
| `faint-foreground` | `#8A7A72` | placeholder, disabled text, empty-state icon |
| `placeholder-stripe-a` / `-b` | `#3A251D` / `#241812` | hatch of "image not yet extracted" |

**States** — the brief's states, never reused for anything else. Each has an edge, a
foreground and a surface, because a badge needs all three.

| state | edge | foreground | surface |
|---|---|---|---|
| `state-done` | `#2CC53F` | `#8FE79B` | `#132916` |
| `state-now` | `#BE8C32` | `#E8BE6E` | `#2B2010` |
| `state-blocked` | `#6C4437` | `#BFB2AC` | `#241812` |
| `state-unknown` | `#8A7A72` | `#BFB2AC` | stripes `#2E1E17` / `#241812` |
| `state-unexpected` | `#FC0000` | `#E8C4BF` | `#2A0B08` |
| `challenge` | `#A855A8` | `#DDA8DD` | `#25102A` |

Two rules come with the palette, agreed in conversation:

- **Gold means "unlockable now" and nothing else.** `Chrome e Stati.dc.html` uses gold for
  section headings and links; here those are `highlight` (cream). The kit's "Ora" badge,
  drawn cream, becomes gold.
- **Cyan means focus** (and active filters, from cycle 2). The info alert's icon, cyan in
  the kit, becomes `highlight`.

### Typography — `theme/typography.css`

| token | size / line height | used for |
|---|---|---|
| `text-title` | 30px / 1.3 | screen title |
| `text-heading` | 21px / 1.35 | card title, empty-state title |
| `text-body` | 14px / 1.5 | paragraphs, field values |
| `text-control` | 13px / 1 | buttons, tabs, toggles |
| `text-row` | 13px / 1.35 | table cells, list and menu items |
| `text-caption` | 12px / 1.45 | field label, helper, badge, tooltip |
| `text-label` | 11px / 1.3 | table header, group label, kbd, footer |
| `text-micro` | 10px / 1.2 | kbd inside menus |

The kit's sizes win over `Tokens.dc.html`'s table (28 / 17 / 15), which no component was
drawn with. Sizes are in px because Determination is a pixel font and its crispness is
tied to whole pixels.

`--font-pixel: 'Determination', monospace`, set as the document's default family.

### Spacing — `theme/spacing.css`

Padding and gaps use the default 4px grid, with half steps (`p-2.5` = 10px) where the kit's
odd values (9px, 11px) need them. Named spacing tokens exist only for **dimensions that mean
something**:

| token | value | meaning |
|---|---|---|
| `spacing-row` | 30px | table row, normal density (the kit's default) |
| `spacing-row-compact` | 24px | full-screen Unlock |
| `spacing-row-wide` | 40px | Next steps, details |
| `spacing-control` | 34px | height of button, input, select; side of an icon button |
| `spacing-scrollbar` | 12px | scrollbar thickness |
| `spacing-sprite` | 4rem | kept: a 32px sprite doubled |
| `spacing-achievement` | 5.5rem | kept: an achievement icon at half size |

The kit's buttons measure 37px and 35px in two places and its icon button 34px: one control
height, 34px, for all of them.

### Radius — `theme/radius.css`

`radius-cell: 2px`, `radius-input: 4px`. Paper, cards, buttons and panels have **no radius**
(no class); chips use `rounded-full`.

### Motion — `theme/motion.css`

| token | value | used for |
|---|---|---|
| `--default-transition-duration` | `0ms` | a bare `transition-*` class is instant, so hover and active can't lag behind by default |
| `duration-tap` / `ease-tap` | 80ms / `steps(2)` | checkbox tick, switch thumb |
| `duration-panel` / `ease-panel` | 120ms / `steps(3)` | popover, select, tooltip, collapsible, chevron rotation |
| `duration-sheet` / `ease-sheet` | 200ms / `steps(5)` | dialog |
| `duration-loop` / `ease-frame` | 640ms / `steps(1)` | skeleton; loops are frame-by-frame keyframes |

Animations are `--animate-*` tokens composed from the tokens above, with their keyframes in
the same file: `animate-tap-in`, `animate-panel-rise`, `animate-panel-drop`,
`animate-panel-open`, `animate-sheet-rise`, `animate-skeleton`. Every translation is a
multiple of 4px; no scale, no blur, no easing curve. There are **no exit animations**:
Reka unmounts immediately when no animation is running, and `Motion.dc.html` draws
entrances only.

`prefers-reduced-motion: reduce` sets every animation and transition to 0ms and loops to a
single iteration — "loops become static, entrances instant", as the Motion page asks.

`--opacity-muted` and `--opacity-disabled` stay as they are.

### Utilities — `assets/utilities.css`

- `pixelated` — kept.
- `hatch-placeholder` — 45° stripes in `placeholder-stripe-a/b`, on the 4px grid
  (`var(--spacing)`): **an image that isn't there yet**.
- `hatch-unknown` — the same geometry in the `state-unknown` stripes: **a value we can't
  read**. Drawn with a dashed edge and a `?` wherever it stands for data, so the two gaps of
  `DESIGN-BRIEF.md` §5.6 never look alike.

## Decision 5 — typography: Determination only

Measured on `determination.ttf` (2048 units per em, 990 glyphs):

- **All ten digits share one advance (1194)**, so numbers align in columns without a second
  font. IBM Plex Mono, used by the export for numbers and annotations, leaves the app.
- **Italian is covered**: àèéìòù and capitals, `’ “ ” « » — · ×`.
- **Missing: `→ ← ↑ ↓ ⏎ ⌘ ✓`.** The kit writes "↑↓ naviga · ⏎ apri" and "The Lost → Mother";
  those would fall back to whatever system font the machine has. They are drawn with Lucide
  icons instead, and the scanner forbids the glyphs in source (Decision 13).
- **One weight.** `font-synthesis-weight: none` on the document stops the browser from
  smearing a fake bold; emphasis is colour (`foreground` against `foreground-soft`). Style
  synthesis stays on: the kit uses italic for "read, but empty".

The font ships as `ui/src/assets/fonts/determination/` with `license.txt` (CC BY 3.0) and
`readme.txt`, which its readme requires alongside any redistribution. The credit line
belongs to the About screen (cycle 3); the files travel from now.

## Decision 6 — focus, scrollbar, base layer — `assets/base.css`

- **One focus ring for the whole app**, in the base layer rather than on each component:
  `:focus-visible { outline: 2px solid var(--color-ring); outline-offset: 2px }`. Rows set
  a negative offset so the ring doesn't push their neighbours. shadcn's
  `focus-visible:ring-[3px] ring-ring/50` classes go in the dressing.
- **Scrollbar always visible, 12px**, flat thumb with an edge, track darker than the sheet —
  written with `::-webkit-scrollbar` only. WebView2 is Chromium, and Chromium ignores the
  `::-webkit-scrollbar` rules on any element that also sets `scrollbar-width` or
  `scrollbar-color`: the export sets both, which would give a thin overlay-like bar instead
  of the 12px one it draws. To be confirmed in planning.
- Document defaults: `font-family` from `--font-pixel`, `text-body`, `bg-background`,
  `text-foreground`, `font-synthesis-weight: none`.

## Decision 7 — primitives come from the CLI and are dressed in a second commit

Each primitive is added with `shadcn-vue add` and **committed untouched**, then dressed in a
separate commit: tokens instead of shadcn's classes, our animations instead of
`tw-animate-css`, no `opacity-50`, no `rounded-*`, no `dark:`, no `shadow-*`, strings through
i18n, variants as constants.

- Reka's accessibility wiring (focus trap, ARIA, keyboard) stays as its maintainers wrote
  it.
- The git history keeps a readable diff "shadcn → IsaacDome" per primitive. When shadcn-vue
  updates a component, `add --overwrite` on a branch shows exactly which lines were ours.
- **`tw-animate-css` is not installed.** Its utilities animate with continuous easing
  (`fade-in`, `zoom-in-95`), the opposite of `Motion.dc.html`. Its class names left behind
  in a component would generate nothing without warning, so the scanner forbids them.

`components.json` points `utils` at `@/lib/cn` (not a catch-all `utils.ts`), and `ui/` gains
the `@/` alias in `tsconfig.app.json` and `vite.config.ts`, which the CLI requires. The
style name to pass (`new-york` in the export, `vega` in the current CLI documentation, the
two pages of shadcn-vue's docs disagreeing) is read from the installed CLI's `--help`.

## Decision 8 — variants are constants, never strings

A variant is an `as const` object exported from the primitive's `index.ts`, and the `cva`
configuration is keyed by it:

```ts
export const ButtonVariant = {
  Default: 'default',
  Secondary: 'secondary',
  Outline: 'outline',
  Ghost: 'ghost',
  Link: 'link',
} as const
export type ButtonVariant = (typeof ButtonVariant)[keyof typeof ButtonVariant]

export const buttonVariants = cva('…', {
  variants: {
    variant: {
      [ButtonVariant.Default]: '…',
      // …
    },
  },
})
```

Consumers write `:variant="ButtonVariant.Outline"`. A literal `variant="outline"` on a
primitive is a scanner violation. The same shape covers every closed set in this cycle:

| constant | where | values |
|---|---|---|
| `ButtonVariant`, `ButtonSize` | `components/ui/button` | see catalogue |
| `BadgeVariant` | `components/ui/badge` | see catalogue |
| `AlertVariant` | `components/ui/alert` | `Default`, `Destructive` |
| `ToggleSize` | `components/ui/toggle-group` | `Default`, `Icon` |
| `TableDensity` | `components/ui/table` | `Compact`, `Normal`, `Wide` |
| `Locale` | `i18n/` | `It`, `En` |
| `ThemeNamespace` | `lib/design/` | the namespaces `cn()` must know |
| `DevRoute` | `lib/constants/` | `Kit: '#kit'` |
| `KeyName` | `lib/constants/` | `Ctrl`, `Esc`, … — key caps are data, not translations |

Anything shared through `provide`/`inject` uses a typed `InjectionKey` symbol exported from
the module that owns it, never a string key.

## Decision 9 — `cn()` knows our tokens, and learns them from the CSS

shadcn's `cn()` is `twMerge(clsx(…))` with no configuration. With our tokens that **silently
drops classes**: tailwind-merge doesn't know `text-body` is a size, reads it as a colour, and
`cn('text-body', 'text-foreground')` keeps only the second. Buttons would lose their font
size the moment a consumer passes a colour.

`lib/cn.ts` builds the merger with `extendTailwindMerge`, registering every custom value of
every namespace we define. **The lists are not written in TypeScript**: `lib/design/themeKeys.ts`
reads them from the theme files themselves (`import typography from '@/assets/theme/typography.css?raw'`),
through a pure function `themeKeys(css, namespace)`. The CSS stays the one place a token is
declared. A TypeScript copy would be a second place, and the conventions forbid exactly
that.

## Decision 10 — i18n arrives now

vue-i18n in composition mode, with `it` and `en` as TypeScript modules: `it.ts` defines the
messages, `export type MessageSchema = typeof it`, and `en.ts` is typed `MessageSchema`. A key
missing from either language, or a key misspelled in `t()`, **fails `vue-tsc`**. Alignment of
the two files is a compile error, not a review note.

The initial locale comes from a pure `resolveLocale(navigator.languages)`: the first language
we have, English otherwise. The settings preference arrives with the Settings screen.

In this cycle the messages are few — the accessible labels primitives render themselves
(`ui.close`, …). `useI18n()` is called in the root component, as the conventions require.

## Decision 11 — the Kit page

A development-only page, `src/kit/KitPage.vue` with one `src/kit/sections/<Primitive>Section.vue`
per primitive, shows **every primitive in every state side by side** — the way
`Shadcn Kit Light.dc.html` lays states out instead of hiding them behind interaction —
so it can be compared with the export page by eye.

- Mounted by `main.ts` only when `import.meta.env.DEV` and the hash is `DevRoute.Kit`,
  through a dynamic import the production build eliminates.
- `src/kit/` is declared **development-only** in the scanner: it is excused from the
  visible-string check, and only from that one, with the reason written in the script. That
  is one declared rule, not one exemption per section file.
- Sample content (item names are data) lives in `src/kit/samples.ts`.

## Primitive catalogue

Only primitives with a use in the eight screens of `DESIGN-BRIEF.md` §4. Values are the dark
kit's, expressed in the tokens above.

| primitive | parts | variants and states | where it serves |
|---|---|---|---|
| **Button** | `Button` | `ButtonVariant`: Default (`primary`, edge `primary-edge`, hover/active tokens) · Secondary (`secondary`, edge `secondary-edge`, hover `secondary-hover` + edge `input`, active `band`) · Outline (transparent, edge `input`, hover `secondary`) · Ghost (transparent edge, hover `secondary` + `secondary-edge`) · Link (`highlight`, underlined). `ButtonSize`: Default (`h-control`, `px-4`, `text-control`) · Icon (`size-control`). Disabled: `muted` surface, `faint-foreground`, edge `secondary`. No radius. | everywhere |
| **Badge** | `Badge` | `BadgeVariant`. **State** variants are pills (`rounded-full`, `text-caption`) and **carry their own icon**, so a state is never colour alone: Done ✓ `Check` · Now `Star` · Blocked `Lock` · Unknown `?` on `hatch-unknown` · Unexpected `TriangleAlert`. **Tag** variants are square (`text-control`): Tag (`data`, edge `input`) · Challenge (`challenge`). The icon per variant is an exhaustive record, not a switch with a default. | Unlock, Next steps, Plan, wiki |
| **Card** | `Card`, `CardHeader`, `CardTitle`, `CardAction`, `CardContent`, `CardFooter` | body `card`, edge `border`; header is the `band` strip; footer separated by `hairline` | Next steps, active profile |
| **Alert** | `Alert`, `AlertTitle`, `AlertDescription` | `AlertVariant`: Default (`data`, edge `border`, icon `highlight`) · Destructive (`destructive-surface`, edge `destructive`, text `destructive-foreground`). Not dismissable: diagnostics don't go away by clicking. | setup chain, `noCatalog`, `storeUnavailable` |
| **Dialog** | `Dialog`, `DialogContent`, `DialogHeader`, `DialogTitle`, `DialogFooter` | backdrop `overlay`; body `sheet`, edge `input`, `animate-sheet-rise`; header is the `band` strip with a close icon button labelled `ui.close` | the `Ctrl+K` palette |
| **Command** | `Command`, `CommandInput`, `CommandList`, `CommandGroup`, `CommandItem`, `CommandEmpty`, **`CommandFooter`** (ours) | `popover`, edge `input`; group heading `text-label` `subtle-foreground`; highlighted item `secondary`; the footer carries the count and key hints drawn with `Kbd` and icons | global search |
| **Tooltip** | `Tooltip`, `TooltipTrigger`, `TooltipContent` | `tooltip`, edge `input`, `text-caption`, `animate-panel-rise`, no arrow | KPI explanations, "blocked by 2" |
| **Popover** | `Popover`, `PopoverTrigger`, `PopoverContent`, **`PopoverTitle`** (ours) | `popover`, edge `input`, `animate-panel-rise`; the title is the `band` strip | profile indicator → change profile |
| **Tabs** | `Tabs`, `TabsList`, `TabsTrigger`, `TabsContent` | list underlined by `border`; active trigger `primary`, inactive `muted-foreground`; content `data`, edge `border` without top | settings, grouped panels |
| **Toggle group** | `ToggleGroup`, `ToggleGroupItem` | one edge `secondary-edge` around, dividers between items; on = `primary`; `ToggleSize`: Default · Icon | All / Missing / Done; sprite / paper |
| **Checkbox** | `Checkbox` | 16px, `data`, edge `input`; checked `primary` + `selection-edge`, tick `animate-tap-in`; indeterminate is a bar; disabled `muted` | Unlock facets, table selection |
| **Switch** | `Switch` | square, 34×18, thumb 12px; on `primary` + `selection-edge`, thumb `foreground`; off `data`, thumb `faint-foreground`; thumb moves with `duration-tap ease-tap` | settings toggles |
| **Select** | `Select`, `SelectTrigger`, `SelectValue`, `SelectContent`, `SelectGroup`, `SelectLabel`, `SelectItem` | trigger `h-control`, `data`, edge `input`, `rounded-input`, chevron rotates with `duration-panel ease-panel`; content `animate-panel-drop`; selected tick in `foreground` — **not** the done green the kit uses | "sort by" |
| **Input** | `Input` | `h-control`, `data`, edge `input`, `rounded-input`, `text-body`, placeholder `faint-foreground`; `aria-invalid` → edge `destructive`; disabled `muted` | search field, folder chosen by hand |
| **Field / Label** | `Label`, `Field`, `FieldLabel`, `FieldDescription`, `FieldError` | label `text-caption foreground-soft`; description `muted-foreground`; error `foreground` beside the red field edge | settings, manual folder |
| **Table** | `Table`, `TableHeader`, `TableBody`, `TableRow`, `TableHead`, `TableCell`, `TableFooter` | `data`, edge `border`; header the `band` strip, `text-label`; rows `text-row`, divided by `hairline`, alternate `row-alt`, hover `row-hover`, focus ring inset; `TableDensity` on `Table` sets `h-row-compact` / `h-row` / `h-row-wide` for every row through an injection key | Unlock, Collection, profile candidates |
| **Progress** | `Progress` | `modelValue`, `max`, **`unknown`**: a filled `primary` segment, then a `hatch-unknown` segment for what can't be read, then empty. Widths go through CSS variables bound from the template, computed by a pure `progressShares` | marks started over readable |
| **Skeleton** | `Skeleton` | `animate-skeleton` between `secondary` and `secondary-hover`; the consumer gives the size | loading (brief §10: skeleton, not spinner) |
| **Separator** | `Separator` | `border` colour, 1px | panels |
| **Collapsible** | `Collapsible`, `CollapsibleTrigger`, `CollapsibleContent` | chevron rotates 90° with `duration-panel ease-panel`; content `animate-panel-open` | collapsible card, facets drawer |
| **Kbd** | `Kbd`, `KbdGroup` | edge `secondary-edge`, `text-label`; arrow and enter keys are Lucide icons at `size-3` | `Ctrl K`, key hints |
| **Empty** | `Empty`, `EmptyMedia`, `EmptyTitle`, `EmptyDescription`, `EmptyContent` | dashed edge `secondary-edge`, `data`; icon `faint-foreground`; title `text-heading`; description `text-row muted-foreground`; actions are Outline buttons | no results; no steps without a catalogue |

Out of this cycle, because no screen uses them: Calendar, Range calendar, Pin input, Tags
input, Carousel, Chart, Stepper, Slider, Number field, Menubar, Context menu, Dropdown menu,
Drawer, Sheet, Alert dialog, Pagination (lists are virtualised), Breadcrumb, Avatar, Hover
card, Accordion, Sonner, Spinner, Sidebar (cycle 2 draws its own: shadcn's can't be resized
by dragging), Scroll area (the always-visible native bar replaces it).

## Decision 12 — what already exists moves in the same cycle

With the default scales switched off, today's classes stop generating, silently. So the
migration belongs to this cycle, not to the next:

- **`main.css`** keeps only `@import 'tailwindcss'`, the theme files, `base.css` and
  `utilities.css`. `--color-mark-*` disappear (the states replace them); `@custom-variant
  dark` goes.
- **`App.vue`** (verification page): `font-mono text-sm` → the document default;
  `text-lg font-bold` → `text-heading`; `font-bold` → `text-foreground`; `border` →
  `border border-border`; `text-mark-*` → the state foregrounds and `destructive`. Its raw
  `<button>` and `<input>` become `Button` and `Input`, and **its "primitives don't exist yet"
  exemption is deleted**, because the reason stops being true. Its visible-strings exemption
  stays, with the reason reworded: the page is replaced by the shell in cycle 3.
- **`WikiInline.vue`**: `font-bold` → `text-foreground`; `italic` stays. Its raw-button
  exemption stays: the wiki link becomes a component in cycle 2.
- **ESLint**: `vue/multi-word-component-names` off for `src/components/ui/**` only, with a
  comment. shadcn's files are named `Button.vue`, `Badge.vue`: the rule exists to avoid
  clashing with HTML elements, and a folder of primitives is exactly where single words
  belong.

## Decision 13 — the scanner learns what this cycle introduces

New checks in `ui/scripts/scan-conventions.mjs`, each added as a row of the conventions'
enforcement table in the same commit:

| check | pattern (heuristic, as the existing ones) | why |
|---|---|---|
| `dark:` variant | `\bdark:` | one theme; the built-in variant follows the OS |
| literal colour in a class | `\[#[0-9a-fA-F]{3,8}\]`, `\[(rgb|hsl|oklch)\(` | colours are tokens |
| colour alpha modifier | a colour utility followed by `/<number>` | alpha lives in a token (`overlay`), not in a class |
| `tw-animate-css` class | `\b(animate-(in|out)|fade-(in|out)|zoom-(in|out)|slide-(in|out)-from)` | the package isn't installed: the class would do nothing |
| literal variant on a primitive | `\s(variant|size|density)="[a-z]` outside `src/components/ui/` | variants are constants |
| glyph missing from Determination | `[→←↑↓⏎⌘✓]` | they fall back to a system font; use the icon |

Plus the declared development-only directory, `src/kit/`, excused from the visible-string
check alone (Decision 11).

## Testing

The frontend has its first logic, so it gets its first test runner: **Vitest**, `pnpm ui:test`
at the root, run by `scripts/check`. Test-first, with expectations from the spec:

- `themeKeys(css, namespace)` — returns the names declared in that namespace; skips the
  `--x-*: initial` reset; skips sub-properties (`--text-body--line-height`); ignores other
  namespaces; on the real `typography.css`, contains `body` and `caption`.
- `cn()` — `cn('text-body', 'text-foreground')` keeps both (size and colour); `cn('text-body',
  'text-caption')` keeps the second; `cn('h-row', 'h-control')` keeps the second;
  `cn('rounded-input', 'rounded-cell')` keeps the second; `cn('bg-primary', 'bg-secondary')`
  keeps the second.
- `resolveLocale(languages)` — `['it-IT', 'en']` → it; `['de-DE', 'en-GB']` → en; `['de']` →
  en; `[]` → en.
- `progressShares(value, unknown, max)` — shares add up to at most 100; negative inputs clamp
  to zero; value plus unknown beyond max clamps; `max = 0` gives zeros, never `NaN`.

What stays visual: the dressing of each primitive, checked on the Kit page against
`Shadcn Kit.dc.html`, and `pnpm check` green (typecheck, lint, format, scan, tests). The
production build is checked once for the absence of the Kit page's chunk.

## Files

```
ui/
  components.json
  src/
    assets/
      main.css                  imports only
      base.css                  document defaults, focus ring, scrollbar, reduced motion
      utilities.css             pixelated, hatch-placeholder, hatch-unknown
      theme/
        colors.css  typography.css  spacing.css  radius.css  motion.css  opacity.css
      fonts/determination/      determination.ttf, license.txt, readme.txt
    components/ui/<primitive>/  index.ts (parts, variant constants) + Part.vue files
    i18n/
      index.ts                  createI18n, Locale, resolveLocale
      messages/it.ts  en.ts
    kit/
      KitPage.vue  samples.ts  sections/<Primitive>Section.vue
    lib/
      cn.ts
      design/themeKeys.ts  progressShares.ts  (+ .test.ts beside each)
      constants/devRoutes.ts  keyNames.ts
```

## Documents updated by this cycle

- `docs/frontend-conventions.md` — one theme; namespaces reset; the token families and
  their files; motion and reduced motion; the font rules (missing glyphs, no synthetic
  bold); variant constants; `cn()` and why its lists come from CSS; the Kit page; the new
  scanner rows; `src/kit/` and `lib/design/` in the structure.
- `CLAUDE.md` — the "target stack, not today's" paragraph (shadcn-vue, Reka, Lucide,
  vue-i18n, Vitest installed; Pinia, Router, TanStack still to come) and `pnpm ui:test` among
  the pass-throughs.
- `docs/STATUS.md` — the design system section and the session log.
- `docs/BACKLOG.md` — a new entry for the 13 problems of the export's `design-export.md`
  (untrimmed sprites, no trim/pivot, Delirium on another sheet, guessed tiers, paired
  papers, paths with spaces, co-op sheet holes, boss portraits by position, base64 icons…):
  they belong to `crates/design-export` and to cycle 2's matrix cell, not to this cycle.

## Handed back to design

Decisions taken here on points where the export contradicts itself or the brief. They go
back to Claude Design so the next export starts from them:

1. **The `@theme` draft uses names outside Tailwind's namespaces** and would produce no
   utilities; the tokens above are its renamed equivalent.
2. **One theme.** The two light palettes (Tokens: paper; Kit Light: leather) disagree; the
   light theme is set aside.
3. **Gold is reserved to "unlockable now"**; headings and links move to cream. The kit's
   "Ora" badge was cream.
4. **Cyan is reserved to focus and active filters**; the info alert icon moves to cream. The
   wiki `edition` tag and the matrix cell's "multiplayer" square need another accent
   (cycle 2).
5. **The "unexpected" state wasn't drawn** anywhere, though the brief's `Cell` and §5.4 ask
   for it. Chosen here: the error tone with a `TriangleAlert` icon. To confirm.
6. **The Select's selected tick was the done green**, reusing a state colour for selection;
   it is `foreground` here.
7. **Determination lacks `→ ← ↑ ↓ ⏎ ⌘ ✓` and has one weight**; arrows and keys are icons,
   bold is colour. IBM Plex Mono is not used by the app.
8. **Two type scales** (Tokens 28/17/15, kit 30/21/14): the kit's is used.
9. **Three button heights** (37, 35, 34px): one, 34px.
10. **Skeleton at 800ms** outside the motion table: it uses the 640ms loop token.
11. **The scrollbar CSS sets both `scrollbar-width` and `::-webkit-scrollbar`**, which in
    Chromium cancels the second; only the webkit rules are used.
12. **Kit examples that contradict the brief** and were not implemented: "Azzera profilo"
    and "perdi lo storico locale" (there is no local history to reset), "Esporta CSV",
    "Auto-sync", a Stepper "Trova il salvataggio → Prima sincronizzazione" (§4.1: profile
    selection is not a wizard, and there is no sync), a Chart of weekly unlocks (§11: no
    charts on data we don't have), a file path inside an input (paths don't cross the IPC),
    `⌘K`/`⌘R` on a Windows app.
13. **For cycle 2, already visible:** the matrix cell has two incompatible encodings (bars in
    Tokens and Kit Light, game sprites in Chrome e Stati); the sprite encoding gives bit 2 a
    meaning ("multiplayer") that `DESIGN-BRIEF.md` §5.3 declares unconfirmed; and the Tokens
    page's note on which cells are unreadable predates 2026-09-08 (today: Mother and The
    Beast for The Forgotten and the 19, 40 cells).

## Verified during planning, before any code relies on it

Each point is checked against the official documentation or by making the tool answer, not
assumed:

1. tailwindcss 4.3: namespace resets keep the static utilities (`bg-transparent`,
   `border-transparent`, `text-current`, `rounded-full`); `--default-font-family` and
   `--default-transition-duration` are the variables preflight and transitions read;
   `@keyframes` inside `@theme`; `--transition-duration-*` and `--opacity-*` generate
   `duration-*` and `opacity-*` (the conventions say verified on 4.3.3, a documentation lookup
   of 2026-09-10 didn't list them: build and look); theme variables referenced only from plain
   CSS (`base.css`, `utilities.css`) are emitted.
2. shadcn-vue CLI: the style name, `init` on an existing Vite workspace with `utils` aliased
   to `@/lib/cn`, and what it rewrites in `main.css` (writing `components.json` by hand is the
   fallback).
3. tailwind-merge (the version shadcn-vue installs): the `extendTailwindMerge` theme keys for
   text sizes, radius, spacing, easing and animations.
4. Vite: `?raw` on a CSS file returns the source untransformed; `url()` in an imported theme
   file resolves the font.
5. WebView2/Chromium: `::-webkit-scrollbar` precedence against `scrollbar-width` and
   `scrollbar-color`.
6. eslint-plugin-vue: `multi-word-component-names` in `flat/essential`; TypeScript 6: `paths`
   without `baseUrl`.
7. Reka UI v2: the data attributes the dressing relies on (`data-state`, `data-highlighted`,
   `data-disabled`) per primitive.
8. vue-i18n: the typed-schema API of the current major version.

## Out of scope for this cycle

- The light theme.
- App components (cycle 2) and screens, shell, Pinia, Vue Router, TanStack (cycle 3).
- Primitives not in the catalogue.
- The export pack's problems (`docs/BACKLOG.md` entry above).
- Committing the export zip: it contains the game's sprites.
