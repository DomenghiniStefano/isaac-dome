# Design system, cycle 2 — app components (design)

**Date:** 2026-09-10
**Milestone:** design system (`docs/STATUS.md`, "Design system"), cycle 2 of 3
**Depends on:** cycle 1 (`2026-09-10-design-system-foundations-design.md`, merged into
`develop`), the Claude Design export of 2026-09-10 (`Chrome e Stati.dc.html` above all),
`DESIGN-BRIEF.md` §4.2, §5.3, §5.6, §8, §10
**Status:** scope, the matrix cell's encoding, bit 2, the edition token, the folder
structure and the shell section agreed in conversation; the cell's scale and the remaining
sections decided by the author on the owner's delegation ("do as much as you can, we look
at it on first launch"). Every delegated choice is marked **(delegated)** so it can be
revisited after that look.

## What this is

The pieces `Chrome e Stati.dc.html` draws and shadcn doesn't have: the window's tab strip
and controls, the navbar, the section sidebar, the KPI tile, the completion-matrix cell,
the wiki's inline tokens and blocks, the data states, and the collapsible card. Each is
shown on the development-only Kit page next to the primitives.

**Presentational only.** Every component takes what it shows through props and says what
happened through emits. No store, no router, no `invoke()`, no tab model, no persistence,
no Tauri window API. Cycle 3 wires them into the shell and the screens.

## Applicable constraints

1. **No game assets in the package.** The mark symbols reach `MarkCell` as URLs in a prop.
   On the Kit page they come from the design pack under `design-export/`, through an
   `import.meta.glob` that the production build never sees (the Kit page is behind
   `import.meta.env.DEV`); where the pack is absent the glob is empty and the page shows
   the fallback outfit.
2. **No IPC change.** `Cell`, `Inline`, `Block`, `Target`, `Dlc` are used as they are.
3. **The five frontend rules** and cycle 1's colour rules: gold means "unlockable now",
   cyan means focus (and active filters), a state's colour is never reused for anything
   else.
4. **i18n.** Every string a component renders itself goes through `useMessages()`; strings
   that belong to the consumer (a tab's label, a KPI's label) arrive as props.

## Decisions agreed in conversation

1. **Scope: presentational components** (not the real shell, not only the components
   without open questions).
2. **Structure: domain folders** under `ui/src/components/`, named for what they are:
   `shell/`, `marks/`, `kpi/`, `wiki/`, `data-state/`. Pure logic in `.ts` files beside
   the component that uses it.
3. **Matrix cell: the game's sprites, with a fallback.** Primary outfit as in
   `Chrome e Stati.dc.html` (paper and the mark's symbol; normal and hard are two different
   sprites in the game). Without sprites, the bar encoding of `Tokens.dc.html`. One
   component, two outfits, chosen by a prop; "never done" and "unreadable" look the same
   in both.
4. **Bit 2: a neutral mark with no name.** A small square in the bottom-right corner, not
   cyan; its accessible text says "third level, meaning unconfirmed". It shows the bit is
   there without claiming what it is.
5. **Edition token: a neutral tag with the name.** Square, the `secondary-edge` border,
   cream text, the edition names; distinguished by shape and text, no new colour.
6. **Shell section** as presented: one tab density, window controls and the drag region in
   the title bar, `ButtonSize`/`ButtonVariant` extensions instead of raw buttons.

## Decision 7 — the cell's scale (delegated)

Four scales were rendered with the real sprites (companion page, 2026-09-10):

| | cell | paper | symbol | grid, 34 rows |
|---|---|---|---|---|
| **A — chosen** | 40px | flat token, sampled from `paper_00.png` | 16 → 32 (2×) | 1428px |
| B | 48px | sprite at ½ | 16 → 32 | 1700px |
| C | 40px | sprite at 96 → 40 | 16 → 32 | 1428px |
| D (the export) | 38px | pre-composed at 38 | 16 → 26 (1.625×) | 1360px |

A, because it is the only one with no resampling: the symbol scales by a whole factor, the
paper isn't scaled at all, and the untrimmed paper sprite (`docs/BACKLOG.md` B10.1: the
drawing sits 2px off centre) never enters the cell. The six paper frames of the game carry
edge and blood variations the app can't read yet, so a flat paper loses nothing the app
knows. The sprite paper looked more like the game on the comparison page; if the first
launch says the flat paper is too plain, the change stays inside `MarkCell`.

`mark-paper` is `#E9DADF`, the pixel at (42, 42) of `completion_widget/paper_00.png`,
sampled through a canvas on 2026-09-10.

## Source material, and the colours that leave

`Chrome e Stati.dc.html` wins on component values, `DESIGN-BRIEF.md` on product, cycle 1's
rules on colour. The page uses 47 distinct colours; in the dark treatment 13 are outside
the palette, and most sit a few points from a token:

| export | where | becomes |
|---|---|---|
| `#93837B` | tab close, sidebar hint, search placeholder, unknown KPI bar | `faint-foreground` `#8A7A72` |
| `#DFD9D8` | sidebar items, wiki body | `foreground-soft` `#D5C8C2` |
| `#C9BDB6` | window glyphs, KPI label, the `?` | `muted-foreground` `#BFB2AC` |
| `#7A6A64` | app name, window unfocused | `faint-foreground` |
| `#5A4A44` | window glyphs, window unfocused | **`chrome-inactive-foreground`** (new) |
| `#0D0908` | tab strip | **`titlebar`** (new) |
| `#0B0807` | tab strip, window unfocused | `titlebar` — two points apart, not a state |
| `#120C0B` | navbar | **`navbar`** (new) |
| `#FC5A44` | drop marker; the Progress section's heading icon | `selection-edge`; `highlight` |
| `#E03A22` | a KPI bar | `primary` |
| `#0E7FA0` | the "multiplayer" square | removed (Decision 4) |
| `#8FD9EA` / `#35707E` | edition tag | removed (Decision 5) |
| `#160E0B` | empty matrix cell | `data` |

The light-treatment colours (`#E3DBDA`, `#857A79`, `#514A49`, `#322C2C`, `#C6B9B8`,
`#A2938A`, `#BFB2AA`, `#241016`) belong to the light theme and the "single paper" row
mode, both out of scope. Gold headings (`#BE8C32`) are `highlight`, as in cycle 1.

## Tokens added

### Colours — `theme/colors.css`

| token | value | role |
|---|---|---|
| `titlebar` | `#0D0908` | the strip that holds tabs and window controls |
| `navbar` | `#120C0B` | the navbar |
| `chrome-inactive-foreground` | `#5A4A44` | window controls and brand when the window has no focus |
| `mark-paper` | `#E9DADF` | the paper a marked cell is drawn on (Decision 7) |

### Spacing — `theme/spacing.css`

| token | value | meaning |
|---|---|---|
| `titlebar` | 30px | title bar height |
| `tab` | 24px | tab height |
| `tab-min` / `tab-max` | 34px / 150px | a tab shrinks to the minimum and stops |
| `drag-region` | 130px | the window's drag area never shrinks below this |
| `window-control` | 38px | width of each window control |
| `navbar` | 38px | navbar height |
| `brand` | 150px | the navbar's brand block |
| `search` | 240px | the search trigger's width, before it shrinks |
| `mark-cell` | 40px | a matrix cell |
| `mark-symbol` | 32px | a mark's symbol, 16px native at 2× |

The sidebar's width is **not** a token: it moves at runtime, and its bounds belong to the
function that clamps it (see `shell/`). The template binds the width as a CSS variable.

Two more, added after the Kit page review and the cleanup pass: `icon-compact` (22px, the tab
strip's "+") in `theme/spacing.css`, and a new family file, **`theme/containers.css`**, with
`--container-tab-narrow: 64px` for the squeezed tab's `@max-tab-narrow` query. A container
size isn't a spacing token, so it doesn't sit in `spacing.css`.

### Typography — `theme/typography.css`

- `text-kpi`: 26px / line height 1.
- **A new namespace, reset first**: `--tracking-*: initial`, then `--tracking-nav: 0.04em`
  (navbar sections) and `--tracking-caps: 0.07em` (uppercase sidebar headings). `cn()`
  learns it through `ThemeNamespace.Tracking` and tailwind-merge's `tracking` theme key.

## Primitive extensions

The conventions say a pattern the primitives don't cover becomes a prop on the primitive,
not hand styling. So:

**`ButtonSize`**

| size | classes (intent) | used by |
|---|---|---|
| `Micro` | 14px square, no border or padding, 8px icon | a tab's close |
| `IconCompact` | 22px square (`icon-compact` spacing token), 12px icon | the tab strip's "+" (added in the cleanup pass: it overrode `Icon` with classes) |
| `Section` | full height, `gap-1.75 px-3.25`, 14px icon | a navbar section (added in the cleanup pass: both call sites repeated the classes) |
| `Row` | auto height, full width, start-aligned, `px-2.75 py-1.75`, `text-row` | sidebar items |
| `Inline` | `inline` (not `inline-flex`: in a flex box the icon sets the baseline and lifts the label, found on the Kit page), auto height, no padding, baseline-aligned, `text-row` | wiki references in running text |
| `Window` | full height, `w-window-control`, no border | window controls |
| `Compact` | `h-7`, `px-2.25`, `text-caption` | the navbar's search trigger |

**`ButtonVariant`**

| variant | intent |
|---|---|
| `Nav` | transparent, 3px left edge; `aria-current="page"` → `primary` fill, `selection-edge` edge, `foreground` text |
| `Section` | transparent, 2px top edge, `tracking-nav`; `aria-current="page"` → `data` fill, `highlight` edge and text |
| `Ref` | cream text over a `secondary-edge` underline border; hover lights the border |
| `Chrome` | transparent, `muted-foreground` glyph, hover `secondary` |
| `ChromeDanger` | as `Chrome`, hover `primary` (the window's close) |
| `Field` | looks like a field: `data` fill, `secondary` edge, `faint-foreground` text, start-aligned |

**Card: the collapsible card is a set of parts, not a prop** (delegated). A prop can't turn
a `div` into Reka's `CollapsibleRoot`; the registry's own pattern for this is parts.
`CardCollapsible`, `CardCollapsibleTrigger` (full-width header: chevron on the left that
rotates 90° with `duration-panel ease-panel`, title, a `summary` slot on the right that
stays visible when closed), `CardCollapsibleContent` (`animate-panel-open`, the header's
12px step). Values from the page: header `muted` fill, `text-caption`; open title
`highlight`; summary `text-label subtle-foreground`.

## Components

### `shell/`

**`TabItem`** — one tab. Props: `tab: TabView`, `active`, `dragging`, `drop: DropSide |
null`. Emits `select`, `close`.

- `role="tab"`, `aria-selected`; the close is a `Button` (`Micro`, `Chrome`) labelled
  `shell.closeTab`, `tabindex="-1"` so arrow keys move between tabs, not into them.
- 24px high, `flex: 1 1 0` between `tab-min` and `tab-max`; origin icon `size-3`, label
  `text-label` truncated with an ellipsis.
- Active: `sheet` fill, `secondary` edge, 2px `primary` top edge, text `foreground`, icon
  `highlight`. Inactive: transparent, text `subtle-foreground`, icon `faint-foreground`.
  Dragging: `opacity-disabled`. Drop target: a 2px `selection-edge` edge on the side the
  tab will land (`data-drop`), which says where it goes, not where the pointer is.
- The origin icon is an exhaustive record over `TabOrigin` (`Wiki` → `Book`, `Progress` →
  `ListX`, `Settings` → `Cog`, `About` → `Info`): the brief wants the origin shown by shape.
- **Squeezed** (found on the Kit page with nine tabs: icon, name and close need about 52px,
  the minimum is 34): the tab is a size container, and below `--container-tab-narrow`
  (64px) an inactive tab hides its close and the active one its icon; overflow is clipped.
  A size container's content counts as zero width, so the tab also carries
  `contain-intrinsic-inline-size: tab-max` (the `tab-intrinsic` utility) and a
  `basis-tab-max` that only shrinks. Without it every tab collapsed to 34px even with three
  tabs open — a regression of the first squeezed-tab fix, caught by the cleanup pass.

**`TabStrip`** — the row of tabs. Props: `tabs: TabView[]`, `activeId`. Emits `select(id)`,
`close(id)`, `move(from, to)`, `add`.

- `role="tablist"`; `ArrowLeft`/`ArrowRight` emit `select` for the neighbour (no wrap).
- Reordering by pointer: a drag starts past a 4px threshold (`TabDrag.Threshold`), so a
  click stays a click; while dragging, the tab under the pointer gets `drop` from
  `dropSide`, and on release `move(from, moveIndex(…))` is emitted once. Pointer capture
  keeps the drag when the pointer leaves the strip.
- The `+` is a `Button` (`Icon`, `Chrome`) labelled `shell.newTab`, emitting `add`.

**`WindowControls`** — minimise, maximise, close: `Button` (`Window`, `Chrome` /
`ChromeDanger`) with `Minus`, `Square`, `X` at `size-2.5`, labelled `shell.minimize`,
`shell.maximize`, `shell.closeWindow`. Emits `minimize`, `toggleMaximize`, `closeWindow`.

**`TitleBar`** — `TabStrip`, the drag region (`min-w-drag-region`, `flex-1`,
`data-tauri-drag-region`), `WindowControls`. Props: the strip's props plus `focused`.
Re-emits everything. `h-titlebar`, `titlebar` fill, `hairline` bottom edge. Unfocused
(`data-focused="false"`): window glyphs `chrome-inactive-foreground`; tabs and their data
stay as they are ("no opacity on content").

**`NavBar`** — Props: `section: NavSection` (`Wiki`, `Progress`), `focused`. Emits
`update:section`, `search`, `settings`, `about`.

- `h-navbar`, `navbar` fill, `hairline` bottom edge.
- Brand: `w-brand`, `hairline` right edge, a 14px square with a 2px `primary` edge on
  `sheet` (unfocused: `border` edge, name `faint-foreground`) and the product name from
  `lib/constants/app.ts` (a name, not a message).
- Sections: two `Button`s (`Section`), icon `Book` / `ListX` at `size-3.5`, labels
  `shell.sections.wiki` / `shell.sections.progress`, `aria-current="page"` on the active one.
- A spacer, then the search trigger: `Button` (`Compact`, `Field`), `w-search`, shrinking;
  search icon, `shell.search`, and `KbdGroup` with `KeyName.Ctrl` and `KeyName.K`. Emits
  `search`: it opens the palette, it isn't the input.
- Settings and About: `Button` (`Icon`, `Chrome`), `Cog` / `Info`.

**`SectionSidebar`** — Props: `width` (`v-model:width`), `title`, `hint`. Slots: `icon`,
default (the items). Emits `update:width`.

- `data` fill, `secondary` edge; the width is bound as `--sidebar-width` and read by
  `w-(--sidebar-width)`.
- Header: icon `size-3.5` in `highlight`, title `text-caption tracking-caps uppercase
  foreground`, hint `text-label faint-foreground`.
- The resize handle: `role="separator"`, `aria-orientation="vertical"`,
  `aria-valuemin/max/now`, focusable, labelled `shell.resizeSidebar`; 5px wide
  (`w-1.25`), `col-resize`, `input` fill on hover and while dragging. Pointer drag emits
  `update:width(clampSidebarWidth(…))`; double click emits `SidebarWidth.Default`;
  `ArrowLeft`/`ArrowRight` step by `SidebarWidth.Step`. `clampSidebarWidth` rounds to a
  whole pixel (a pixel font on half pixels blurs) and turns a non-finite width into the
  default.

**`SidebarItem`** — Props: `active`. Slots: `icon`, default. A `Button` (`Row`, `Nav`), icon
`size-4`, `aria-current="page"` when active.

**`shell/tabs.ts`** — `TabOrigin`, `TabView` (`{ id: string; label: string; origin:
TabOrigin }`), `DropSide` (`Before`, `After`), `TabDrag` (`Threshold: 4`), and the pure
functions `dropSide` and `moveIndex`. **`shell/sidebarWidth.ts`** — `SidebarWidth`
(`Min: 168`, `Default: 212`, `Max: 420`, `Step: 16`) and `clampSidebarWidth`.
**`shell/navSection.ts`** — `NavSection`.

### `marks/`

**`markVisual(cell: Cell): MarkVisual`** — what a cell shows, decided once:

```ts
export const MarkTier = { Normal: 'normal', Hard: 'hard' } as const
export type MarkVisual =
  | { kind: 'empty'; third: boolean }
  | { kind: 'marked'; tier: MarkTier; third: boolean }
  | { kind: 'unknown' }
  | { kind: 'unexpected'; value: number }
```

- bit 1 set → `marked`, `Hard`, **with or without bit 0**: `2` appears 32 times on the
  reference profile, and the game draws the hard sprite for it;
- only bit 0 → `marked`, `Normal`; neither → `empty`;
- bit 2 → `third: true`, independently of the other two (`4` alone is `empty` with
  `third`: never observed, handled rather than guessed);
- a `known` cell carrying any bit above bit 2 → `unexpected` with its value: the component
  never draws a guess;
- `unknown` → `unknown`; `unexpected` → `unexpected` with the IPC's value.

**`MarkCell`** — Props: `cell: Cell`, `art: MarkArt | null` (`{ normal: string; hard:
string }`, the symbol URLs of that column), `label` (optional: the consumer's accessible
description, character × boss). `size-mark-cell`. With a label the cell is `role="img"`
and its `aria-label` is the label plus, when `third`, `marks.thirdLevel`; without one it
is decorative (`aria-hidden`), for when the text beside it already says what it is.

| visual | sprite outfit (`art` given) | fallback outfit (`art` null) |
|---|---|---|
| empty | `data` fill, `hairline` edge | same |
| marked normal | `mark-paper` fill, `art.normal` at `size-mark-symbol`, `pixelated` | `data`, `input` edge, a `primary` bar one third high |
| marked hard | `mark-paper`, `art.hard` | `data`, `input` edge, a two-thirds bar and `ChevronUp` |
| third bit | 6px square bottom-right, `background` ink | 6px square top-right, `highlight` |
| unknown | `hatch-unknown`, dashed `state-unknown` edge, `?` in `muted-foreground` | same |
| unexpected | `state-unexpected-surface`, `state-unexpected` edge, `TriangleAlert` | same |

Bar heights and the third-bit square use CSS variables bound from the template (the bar's
share), or grid classes (`size-1.5`). The cell has no tooltip of its own: the full one,
with character and boss names, arrives with the Completion screen.

Delirium's symbol comes from another sheet but has the same 16 × 16 shape: no special
case here. Its `symbolFallback` is a backend concern (cycle 3).

### `kpi/`

**`hasKpiBar(denominator): denominator is number`** — whether there is a bar at all: not
without a declared denominator (`null`, not finite, or ≤ 0), because "every KPI has a
declared denominator or no bar". The bar is the `Progress` primitive at
`ProgressSize.Micro`, its fill `ProgressTone` chosen by a record over `KpiTone` (the cleanup
pass replaced a first, hand-drawn bar and a `kpiBar` that computed the share a second time).

**`KpiTile`** — Props: `value`, `denominator` (optional), `unit` (optional text shown
instead of `/ denominator`), `label`, `tone: KpiTone` (`Progress`, `Done`, `Unknown`).
Slot `explain`: when given, the tile is a `Tooltip` trigger and the slot its content (the
explanation lives in the tooltip, not in prose).

- `sheet` fill, `border` edge, `px-3.25 pt-2.75 pb-3`.
- Value `text-kpi foreground` (`Unknown`: `muted-foreground`, same size: it's a datum,
  not a progress); `/ denominator` or the unit in `text-caption subtle-foreground`;
  label `text-label muted-foreground`.
- Bar only when `kpiBar` isn't `null`: `h-1`, `data` fill, `secondary` edge; the fill's
  width is a CSS variable; colour by tone — `Progress` `primary`, `Done` `state-done`,
  `Unknown` `faint-foreground`.

### `wiki/`

**`editionLabel(only: Dlc[]): string`** — the editions' names in `Dlc`'s declaration order,
each once, joined by ` · `; `''` for an empty list. Names from an exhaustive
`Record<Dlc, string>` in `wiki/dlcNames.ts`: Rebirth, Afterbirth, Afterbirth+, Repentance,
Repentance+ (game names stay in English, as items do).

**`WikiInline`** — moves from `components/` to `components/wiki/` and is rewritten.
Props: `inline: Inline[]`, `iconFor` (optional `(t: Target) => string | null`, supplied by
cycle 3 from the icon protocol). Emits `navigate(target)`. The verification suffix
`(item 105)` goes.

| token | render |
|---|---|
| `text` Plain | `foreground-soft` |
| `text` Bold | `foreground` — one weight: emphasis is colour |
| `text` Italic | `italic` |
| `ref` | `Button` (`Inline`, `Ref`); a 16px `pixelated` icon before the label when `iconFor` returns a URL, **no icon space when it doesn't** |
| `concept` | `subtle-foreground`, dotted `secondary-edge` underline border; not clickable |
| `edition` | a square tag (`secondary-edge` edge, `highlight` text, `text-label`, `px-1.25`) with `editionLabel(only)`, then its own inline content |

Tokens are rendered with no whitespace text nodes between them (the page's note: markup
indentation would become a space before every comma).

**`WikiBlocks`** — moves to `components/wiki/`, dressed: paragraphs `text-row
foreground-soft` with `leading` from the row token; lists with a `gap-1`; tables with the
`Table` primitive (band header); headings `text-control highlight` (level 3) and
`text-caption highlight` (deeper). `App.vue` updates its import.

### `data-state/`

Three of the page's four states; the fourth, "read, full value", is ordinary text.

| component | render | content |
|---|---|---|
| `EmptyValue` | `italic text-row faint-foreground` | slot: e.g. "not in the dataset" |
| `UnreadableValue` | a decorative `MarkCell` (no label) with `{ kind: 'unknown' }` beside the slot in `text-caption muted-foreground` | slot: e.g. "the save doesn't tell us" |
| `EmptyCategory` | dashed `border` edge, `px-3 py-2.5`, `text-caption subtle-foreground` | slot: e.g. "no Trinket pages" |

"Unreadable" is outside the denominator of every percentage; the component can't enforce
that, the screen does.

## i18n

New messages, Italian as the schema, English beside it:

| key | it | en |
|---|---|---|
| `shell.newTab` | Nuova tab | New tab |
| `shell.closeTab` | Chiudi tab | Close tab |
| `shell.minimize` | Riduci a icona | Minimize |
| `shell.maximize` | Ingrandisci | Maximize |
| `shell.closeWindow` | Chiudi finestra | Close window |
| `shell.search` | Cerca in tutto | Search everything |
| `shell.settings` | Impostazioni | Settings |
| `shell.about` | Informazioni | About |
| `shell.resizeSidebar` | Ridimensiona la barra laterale | Resize the sidebar |
| `shell.sections.wiki` | Wiki | Wiki |
| `shell.sections.progress` | Progressi | Progress |
| `marks.thirdLevel` | terzo livello, significato non confermato | third level, meaning unconfirmed |

## The Kit page

A second part below the primitives, headed "Componenti app", one section per component:

- `TitleBar` twice, focused and not, and once under pressure with nine tabs (the drag region
  stays whole); the drag is live.
- `NavBar` with the section switch live.
- `SectionSidebar` with live resizing, the Progress and Settings contents of the page.
- Four `KpiTile`s from the page (166 / 368 started, 120 cells, 3 / 34 complete, 40
  unreadable).
- `MarkCell`: every visual in both outfits, and a 4 × 5 grid with the reference profile's
  values, including an unknown pair.
- `WikiInline` with the four natures and the page's paragraph; `WikiBlocks` with a list and
  a table.
- The three data states; two collapsible cards, one open, one closed.

Sprites come from `src/kit/markArt.ts`: an eager `import.meta.glob` over the pack's
`completion_widget/*_0[02].png` and Delirium's `onlinelobby` pair, as URLs. An empty glob
gives `null` art and the fallback outfit.

## Testing

Vitest, test-first, expected values from this document:

- `markVisual` — `0` → empty; `1` → normal; `2` → hard; `3` → hard; `4` → empty + third;
  `5` → normal + third; `6` → hard + third; `7` → hard + third; `8` → unexpected 8;
  `unknown` → unknown; `unexpected` 9 → unexpected 9.
- `clampSidebarWidth` — 100 → 168; 168 → 168; 300 → 300; 500 → 420; `NaN` → 212;
  250.6 → 251.
- `dropSide(pointerX, left, width)` — left half → Before; right half → After; the exact
  middle → After.
- `moveIndex(from, target, side)` — (0, 2, After) → 2; (0, 2, Before) → 1; (3, 1, Before) →
  1; (3, 1, After) → 2; (2, 2, Before) → 2; (2, 2, After) → 2.
- `editionLabel` — `[]` → `''`; `[repentance]` → `Repentance`; `[repentancePlus,
  afterbirth]` → `Afterbirth · Repentance+`; duplicates appear once.
- `hasKpiBar` — 368 → true; `null` → false; 0 → false; `NaN` → false. The share itself is
  the `Progress` primitive's, already tested through `progressShares` (the cleanup pass
  replaced a hand-drawn bar with `Progress` at `ProgressSize.Micro`).
- `cn` — `cn('tracking-nav', 'tracking-caps')` → `tracking-caps`; `themeKeys` reads the
  real typography file's tracking names.

What stays visual, on the Kit page against `Chrome e Stati.dc.html`: every component in
every state, focused and unfocused, a drag in progress, an open collapsible card. The
production build is checked for the absence of the Kit page **and of any mark sprite**.

## Files

```
ui/src/
  assets/theme/          colors.css spacing.css typography.css containers.css (tokens above)
  assets/main.css                    imports containers.css
  components/ui/progress/            variants.ts: ProgressSize, ProgressTone (the KPI bar)
  lib/constants/aria.ts              AriaCurrent
  lib/design/themeKeys.ts (+ test)   Tracking namespace
  lib/cn.ts (+ test)                 tracking theme key
  lib/constants/app.ts               the product name
  components/ui/button/variants.ts   the new sizes and variants
  components/ui/card/                CardCollapsible, CardCollapsibleTrigger, CardCollapsibleContent
  components/shell/                  TitleBar TabStrip TabItem WindowControls NavBar
                                     SectionSidebar SidebarItem tabs.ts (+ test)
                                     sidebarWidth.ts (+ test) navSection.ts
  components/marks/                  MarkCell markVisual.ts (+ test)
  components/kpi/                    KpiTile kpiBar.ts (+ test)
  components/wiki/                   WikiInline WikiBlocks editionLabel.ts (+ test) dlcNames.ts
  components/data-state/             EmptyValue UnreadableValue EmptyCategory
  i18n/messages/                     it.ts en.ts
  kit/KitPage.vue  kit/markArt.ts  kit/sections/app/<Component>Section.vue
  App.vue                            the wiki imports
ui/scripts/scan-conventions.mjs      WikiInline's raw-button exemption goes
```

## Documents updated by this cycle

- `docs/frontend-conventions.md` — the domain folders that now exist; the `Button`
  extensions and when to add one; the tracking tokens; the collapsible card as parts.
- `docs/STATUS.md` — cycle 2 and the session log.
- `CLAUDE.md` — the State paragraph.
- `docs/BACKLOG.md` — the marks map's move from `design-export` to `ipc` for the
  Completion screen, logged where the brief already asks for it.

## Handed back to design

1. **Thirteen colours outside the palette** in the dark treatment; the table above maps
   them, three new tokens remain.
2. **Three tab densities** (26·108, 34·150, 44·208): one is used, 34·150.
3. **Navbar icon buttons 30×26**: the single control height, 34px.
4. **Icon sizes 13px and 15px** are off the 4px grid's half steps: 14px and 16px.
5. **"Multiplayer" for bit 2**, drawn cyan: a neutral unnamed square (Decision 4).
6. **A cyan edition tag**: a neutral tag (Decision 5).
7. **A 38px cell, pre-composed paper, 26px symbol**: 40px, flat paper, 32px symbol
   (Decision 7).
8. **The "single paper" row mode** and the light treatments are not implemented.
9. **The sidebar heading icon** is `#FC5A44` in one section and `#DFD9D8` in the other:
   `highlight` in both.
10. **`⌘K` and arrow glyphs** in the shortcuts: `Ctrl K` and icons (cycle 1, Decision 5).
11. **`Tokens.dc.html` reads the cell as levels 1, 2, 3**; the save stores a bitmask: the
    bars encode bit 0 and bit 1, bit 2 is the separate square.
12. **The drop marker `#FC5A44`**: `selection-edge`.

## Out of scope for this cycle

- The tab model, session restore (B6), `decorations: false` and the window API calls, the
  sidebar width's persistence.
- Serving the mark sprites: the map in `crates/design-export`'s `marks.json` moving to
  `ipc`, Delirium's fallback, the icon protocol for marks.
- The matrix as a grid (headers, portraits, groups) and the cell's tooltip: the Completion
  screen.
- The search palette's wiring, the Settings and About screens.
- The light theme.
