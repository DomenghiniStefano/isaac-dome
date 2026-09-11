# Design system, cycle 3.2 — Completion (design)

**Date:** 2026-09-11
**Milestone:** design system (`docs/STATUS.md`), cycle 3 of 3, sub-project 2 of 7
**Depends on:** sub-project 3.1 (`2026-09-11-screens-shell-profile-design.md`), cycles 1 and 2,
`docs/BACKLOG.md` B13, `DESIGN-BRIEF.md` §5.3, §5.4, §5.6, §10, `Schermate.dc.html` of the
Claude Design export (the Completion screen, lines 381–477, and its `matrix()` and `cell()`,
lines 2232–2425)
**Status:** every decision below was taken by the author on the owner's delegation ("comincia a
fare in autonomia"). Each is marked **(delegated)** so the first look can overturn it cheaply.

## What this sub-project is

The Completion screen: the character × mark matrix, and how much of it the save lets us read.
It is the screen the brief says to design first together with profile selection, because it
runs entirely on real data — and the one whose design is verified by a known profile rather
than by looking plausible: **166 started out of 368 readable · 40 unknown · 0 suspect**
(§5.4).

It closes B13 on the way: the marks map moves from `crates/design-export` into `ipc`, and the
icon protocol learns to serve **crops** — the mark symbols and the character heads are pieces of
a game sheet, not files.

## Applicable constraints

1. **The IPC change is additive, and handed on.** Three fields join `MarksMatrix`; no field
   changes shape or meaning, except `totals.started`, whose meaning is aligned with the grid
   (Decision 1) without moving a number on any save we hold. The change is recorded in
   `DESIGN-BRIEF.md`, not only committed.
2. **No game asset in the package.** Symbols and heads reach the screen as icon-protocol URLs
   read from the user's archives. The development fixtures read the design pack under
   `import.meta.env.DEV` only, like the Kit page.
3. **Degrade, never fail.** No game installed: no URLs, every cell in the fallback outfit (bars)
   and every head a placeholder. A symbol that doesn't load (a layer a patch renamed): that
   cell falls back, the rest keep their art. No counters section: every cell `unknown`, and an
   `Alert` says so above the grid.
4. **No percentage** (§5.3): counts with their denominators, and "unreadable" outside every
   denominator.
5. **This machine has no game installed** (2026-09-11: one Steam library, no
   `appmanifest_250900.acf`). The archive side — anm2 frames, sheet crops, the handler — is
   verified on synthetic anm2 XML and generated PNGs; the real sprites are looked at on the
   first launch on a machine with the game. The frontend is verified on fixtures that carry the
   reference profile's real matrix.

## Decision 1 — the contract (handed on)

```rust
pub struct CharacterRow {
    pub character: String,
    pub group: CharacterGroup,
    pub tainted: bool,             // new
    pub cells: Vec<Cell>,
    pub head_url: Option<String>,  // new
}

pub struct MarkArtView {           // new
    pub normal_url: Option<String>,
    pub hard_url: Option<String>,
}

pub struct MarksMatrix {
    pub characters: Vec<CharacterRow>,
    pub bosses: Vec<String>,
    pub art: Vec<MarkArtView>,     // new: art[i] draws bosses[i]
    pub totals: MarksTotals,
}
```

- **`tainted`** comes from `CHARACTER_KEYS`, which already carries it. The screen groups rows the
  way a player thinks of them — 17 base, 17 Tainted, as `Schermate.dc.html` does — while
  `group` stays the file's three groups, which is where the unread cells live. Deriving
  "Tainted" from a row index in TypeScript would hardcode a count.
- **`art`** is a parallel array rather than a change to `bosses`: `bosses: string[]` is what the
  design was built on. A test pins `art.len() == bosses.len()`.
- **URLs are emitted only when a catalog is given**, i.e. the game's archives are open: `art`
  for every column, `head_url` for a row whose catalog character has a head (37 of 41
  characters; the "Jacob & Esau" row takes Jacob). Without a catalog every URL is `null`.
  `marks_matrix(counters, catalog: Option<&Catalog>, icon: impl FnMut(&IconRef) -> Option<String>)`,
  the shape `unlock_view` already has.
- **`totals.started` counts a cell when bit 0 or bit 1 is set** (`bits & 3 != 0`). Today it
  counts any non-zero mask, so a cell holding only the unconfirmed bit 2 would be "started"
  while `markVisual` (cycle 2) draws it empty — a total that disagrees with its own grid. Value
  4 has never been observed (§5.4: 0, 1, 2, 3, 5, 7), so no number moves on any save; a test
  with a synthetic `4` pins the rule.

TypeScript mirror in `ui/src/lib/ipc/types.ts`: `tainted: boolean`, `headUrl: string | null`,
`art: { normalUrl: string | null; hardUrl: string | null }[]`.

## Decision 2 — two new icon references

`IconRef` gains two variants, with their paths:

| reference | path | resolves to |
|---|---|---|
| `Mark { column, tier }` | `mark/9/hard` | a frame of `completion_widget.anm2` or `onlinelobby.anm2` |
| `Head { row }` | `head/0` | `character_for(row, catalog)?.head` |

`MarkTier` (`Normal`, `Hard`) lives in `ipc`; on the wire it exists only as the path token.
`parse` rejects a column past `BOSSES` and a row past `CHARACTERS`, so the handler answers 400
without reaching the archives.

**The marks map** — domain knowledge, as §5.6 says, next to `BOSSES`:

| column | anm2 | animation | layer | how we know |
|---|---|---|---|---|
| 0 Mom's Heart | `gfx/ui/completion_widget.anm2` | any | `Heart` | layer name |
| 1 Isaac | widget | any | `Polaroid` | layer name |
| 2 Satan | widget | any | `UpsideDownCross` | layer name |
| 3 Boss Rush | widget | any | `Star` | layer name |
| 4 Blue Baby | widget | any | `Negative` | layer name |
| 5 The Lamb | widget | any | `Cross` | elimination |
| 6 Mega Satan | widget | any | `MegaSatan` | layer name |
| 7 Greed | widget | any | `Greed` | layer name |
| 8 Hush | widget | any | `Hush` | layer name |
| 9 Delirium | `gfx/ui/main menu/onlinelobby.anm2` | `Background` | `Completion_Delirium` | layer name (online lobby) |
| 10 Mother | widget | any | `Knife` | layer name |
| 11 The Beast | widget | any | `DadsNote` | layer name |

The lobby needs its animation: the same layer is drawn twice, in `Background` and in
`PlayerInfo`, at different rectangles. The widget has one animation.

**A tier is a frame index**: `Normal` → frame 0, `Hard` → frame 2. These are the rectangles the
design pack cut into `heart_00.png` / `heart_02.png` and that cycle 2 drew on the Kit page as the
two levels. Frame 0 is declared invisible in the widget's anm2 (the "not taken" state), and the
pack's cutter keeps one crop per rectangle, so frame 1 is not in the pack. **Open until the game
is on the machine:** whether frame 1 repeats frame 0's rectangle — `pnpm design:export --dump
gfx/ui/completion_widget.anm2` answers it. If it doesn't, the normal symbol is frame 1 and the
table changes in one place.

`mark_source(column, tier, frames: &MarkFrames) -> Option<SpriteRef>` is pure: `MarkFrames`
holds the parsed frames of the two anm2 files, the sheet path is the frame's own
(`Anm2Frame.sheet`) resolved against the anm2's folder the way `design-export/src/sheets.rs`
resolves it. A layer or frame that isn't there is `None`, never a neighbour.

The Delirium fallback of `marks.json` (`paper_02`, the bloodied paper) is **not** carried: the
face is the primary and the cell already has a fallback outfit (delegated).

## Decision 3 — the handler serves crops

- **`crop` moves** from `crates/design-export/src/atlas.rs` to `ipc` (`sprite_png.rs`), with
  `decode` and its three tests; `design-export` imports it from there. `ipc` gains
  `png = "0.17"`. It stays pure: bytes in, bytes out.
- **`icon_bytes`** reads a `SpriteRef` one way for every reference: the file, cropped when the
  reference carries a `rect`. `Mark` goes through `mark_source`, `Head` through the catalog;
  `Achievement` and `Item` are unchanged (whole files).
- **`MarkFramesState`** (`OnceLock`) holds the two anm2 files' frames, read once from the
  `ResourceSet`. Filled only when both read; the game being absent is never cached.
- **No decoded-sheet cache** (delegated): 24 symbols and 34 heads, one decode each, then the
  webview's cache. Revisit if the first launch is slow.

## Decision 4 — the screen (delegated)

Layout from `Schermate.dc.html`, in cycle 1 and 2 components.

- **Header:** `ScreenHeader` with the grid icon, "Completamento", and one line — "Personaggio ×
  marchio, e quanto il salvataggio lascia leggere."
- **KPIs**, four `KpiTile`s, each explanation in its tooltip:

  | value | denominator / unit | label | tone |
  |---|---|---|---|
  | started | / readable | marchi iniziati | `Progress` |
  | normal and hard both set | unit "celle" | normale + hard | `Progress` |
  | characters with every readable cell started | / characters | personaggi completi | `Done` |
  | unknown | / cells, unit "celle" | non leggibili | `Unknown` |

- **The matrix card**: a `Card` with the band title "Matrice dei marchi"; a legend row of five
  decorative `MarkCell`s — mai fatto (0), normale (1), hard (3), terzo livello (7), non leggibile
  (`unknown`) — drawn with column 0's art when there is some. `unexpected` stays out of the
  legend: it explains itself in its tooltip (the export's own choice).
- **The grid**, scrolling horizontally inside the card:
  - columns: the name column, one `mark-cell` per boss, the total column — through a
    `grid-cols-matrix` utility that reads `--matrix-columns`, bound from the template because the
    column count is data;
  - column headers: the boss name written vertically, the hard symbol under it at
    `mark-symbol`, both in a `matrix-header`-tall box;
  - **two groups**, base and Tainted (`tainted`), each opened by a band: name, first → last
    character, "N/M iniziati", and "K non leggibili" when there are any;
  - a row: the head at 32px, native (the export's 36px is a 1.125 scale that blurs pixel art),
    or a `hatch-placeholder` box of the same size; the name, truncated; twelve `MarkCell`s with
    the label "character · boss"; the row total "started/readable";
  - a footer row, "Personaggi con il marchio", with each column's "started/readable";
  - alternating rows on `row-alt`, hover on `row-hover`.
- **Totals' colour**: complete (started = readable > 0) on `state-done-foreground`, otherwise
  `subtle-foreground`, nothing readable on `faint-foreground`. The export's gold for "two or
  fewer missing" is dropped: gold means "unlockable now" and nothing else (cycle 1).
- **A cell's tooltip** (delegated): character · boss, then the state — mai fatto, normale, hard,
  normale e hard, "non leggibile: la colonna non è localizzata per questo personaggio",
  "valore N, fuori da quelli previsti" — and "terzo livello, significato non confermato" when bit
  2 is set. One `Tooltip` per cell; if 408 of them make the grid lag at first launch, a single
  readout under the grid replaces them.
- **Nothing readable** (`readable === 0`): an `Alert` above the grid, "Il salvataggio non lascia
  leggere nessun marchio"; the grid still draws every cell `unknown`.
- **Loading:** `Skeleton` blocks in the KPI row's and the card's shapes. **Error:** an `Alert`
  with the error kind's message and a retry.
- **No active profile:** 3.1's `ProgressGate` already shows the profile selection.

New tokens in `theme/spacing.css`: `matrix-name` 176px (the export's name column),
`matrix-total` 52px, `matrix-header` 118px (a vertical "Mom's Heart" and a 32px symbol). New
utilities in `utilities.css`: `grid-cols-matrix`, `writing-vertical`.

## Decision 5 — state and pure logic

- **`stores/completion.ts`** (Pinia, setup syntax): `matrix`, `status` (`LoadStatus`), `error`,
  `load()`. The screen loads on mount and whenever the active profile's id changes: a matrix
  belongs to one profile and is never shown under another. `isIpcError` moves from the profile
  store to `lib/ipc/errors.ts`, shared.
- **`lib/completion/completionView.ts`**, pure, tested first:
  - `cellStatus(cell)` — `Empty`, `Normal`, `Hard`, `Both`, `Unknown`, `Unexpected`, plus
    `third`; the tooltip's vocabulary, on the same bit rules as `markVisual`;
  - `rowTally(row)` — `{ started, readable, complete }`;
  - `columnTallies(matrix)` — the same per boss;
  - `matrixGroups(matrix)` — base and Tainted, each with its rows' indices, first and last name,
    started, readable, unknown;
  - `completionKpis(matrix)` — started, readable, both, complete characters, characters,
    unknown, cells.

## Decision 6 — fixtures and development art

- **`lib/ipc/fixtures/completion.ts`**: the reference profile's matrix (the export's
  `completion.json`, the same profile as §5.4), one twelve-character string per row, `?` for
  `unknown`. `completion` answers it when a profile is active and rejects with `noActiveProfile`
  otherwise, like `save_summary`.
- **`lib/ipc/fixtures/art.ts`** absorbs `kit/markArt.ts`: the twelve columns' symbols and the
  co-op menu heads (`coop_menu/main_NN.png`, frame = the head map of `catalog`), from the design
  pack through `import.meta.glob`. The Kit imports it from there. A clone without the pack gets
  empty globs and `null` URLs.
- **`?art=none`** on the development server answers with every URL `null`, to look at the
  fallback outfit — every user's first launch (delegated).

## Decision 7 — a picture that doesn't load

`MarkCell` drops to the fallback outfit when its symbol fails to load, instead of showing a
broken-image glyph; the failure resets when `art` changes. The row head does the same with its
placeholder (`components/marks/CharacterHead.vue`).

## i18n

New messages under `completion.*`: header line, KPI labels, units and explanations, card title,
legend labels, group names and totals, footer label, cell states, the nothing-readable alert.
Character and boss names stay data. `placeholder.completion` and the route's `routeArrives`
entry go.

## Testing

Rust, test-first:

- **`marks`** — `tainted` for rows 0, 14, 16 (false) and 17, 33 (true); `art.len() ==
  bosses.len()`; without a catalog every URL is `null`; a synthetic counter of `4` isn't
  started, `5` is; the JSON shape (`tainted`, `headUrl`, `art[].normalUrl`, `art[].hardUrl`).
- **`icon`** — `Mark` and `Head` survive the round trip; `mark/12/hard`, `mark/0/wizard`,
  `head/34`, `head/0/x` parse to nothing.
- **`mark_source`** — on a synthetic widget anm2 and lobby anm2 (via `catalog::anm2_frames`):
  each tier picks its frame's rectangle; Delirium picks `Background`, not `PlayerInfo`; a
  missing layer or frame is `None`.
- **`sprite_png`** — the three crop tests, moved.

Vitest, test-first, expectations from §5.4 and the Kit page's tiles (read off the same profile):

- **`completionKpis`** on the reference matrix — 166 started, 368 readable, 120 normal + hard,
  3 complete characters of 34, 40 unknown of 408.
- **`rowTally`** — Isaac 12/12 complete; The Forgotten 9/10; T. Magdalene 0/10.
- **`columnTallies`** — Mom's Heart 21/34; The Beast 5/14.
- **`matrixGroups`** — two groups of 17; base Isaac → Jacob & Esau with 6 unknown, Tainted
  T. Isaac → T. Jacob & Esau with 34 unknown.
- **`cellStatus`** — 0, 1, 2, 3, 4, 5, 7, `unknown`, `unexpected`.
- **`transport`** — `completion` answers with an active profile and rejects without one.

Visual checks on the development server: `?fixture=active` with art, `?fixture=active&art=none`,
`?fixture=pick` (the gate on Completion), a cell's tooltip, the unknown block in the bottom-right
corner. The production build is checked for the absence of the fixtures and the pack's images.

## Files

```
crates/ipc/Cargo.toml                 png
crates/ipc/src/marks.rs               tainted, art, head_url, started, marks_matrix signature
crates/ipc/src/mark_art.rs            MarkTier, MARK_LAYERS, MarkFrames, mark_source
crates/ipc/src/icon.rs                Mark, Head
crates/ipc/src/sprite_png.rs          decode, crop (from design-export)
crates/ipc/tests/                     marks.rs, icon.rs, mark_art.rs
crates/design-export/src/             atlas.rs loses crop; main.rs, sheets.rs, payload.rs follow
crates/app/src/lib.rs                 completion with catalog and icons; MarkFramesState; icon_bytes crops
ui/src/
  lib/ipc/types.ts errors.ts          the mirror; isIpcError
  lib/ipc/fixtures/                   completion.ts art.ts, index.ts handler, ?art=none
  lib/completion/completionView.ts (+test)
  stores/completion.ts
  screens/CompletionScreen.vue
  screens/completion/                 CompletionKpis MarksMatrixCard MatrixLegend
  components/marks/                   MarkCell (failed image) CharacterHead MarksGrid
  assets/theme/spacing.css utilities.css
  router/routes.ts routeTable.ts      the screen; no placeholder
  i18n/messages/it.ts en.ts
  kit/                                imports fixtures/art.ts
DESIGN-BRIEF.md                       §5.3, §5.6, §7: the contract change and where the map lives
```

## Out of scope for this sub-project

- **Boss portraits in the column headers.** The export's grid draws the name and the mark, not
  the portrait; Mother's 480 × 440 and The Beast's 256 × 144 don't fit a 40px column.
- **The "one sheet" treatment and the sprite toggle** of the export: they compare treatments for
  design. The product's outfit follows whether art exists.
- **A cell leading to Unlock** (3.3), and any keyboard navigation across 408 cells: the grid is a
  picture with labels.
- **Regenerating the design pack**: `pnpm design:export` needs the game. The pack's
  `contracts/types.ts` catches up at the next export; until then the brief says what changed.
